use base64::Engine;
use poem_openapi::{OpenApi, param::Path, payload::Json};

use super::AdminApiState;
use super::dto::{
    CsrDto, GenerateCsrReq, GenerateKeyReq, ImportCertReq, KeyDto, SimpleSuccessResponse, TokenDto,
    TokenLoginReq, TokenPinReq,
};

pub struct TokensApi {
    pub state: AdminApiState,
}

#[OpenApi]
impl TokensApi {
    /// Lists cryptographic tokens (software and hardware tokens).
    #[oai(path = "/v1/tokens", method = "get")]
    pub async fn list_tokens(&self) -> Json<Vec<TokenDto>> {
        let tokens = self.state.signer.list_tokens().await;
        let dtos = tokens
            .into_iter()
            .map(|t| TokenDto {
                id: t.id,
                name: t.name,
                token_type: t.token_type,
                logged_in: t.logged_in,
                has_pin: t.has_pin,
            })
            .collect();
        Json(dtos)
    }

    /// Logs into a token with PIN.
    #[oai(path = "/v1/tokens/:id/login", method = "put")]
    pub async fn login_token(
        &self,
        id: Path<String>,
        body: Json<TokenLoginReq>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        match self.state.signer.login_token(&id.0, &body.pin).await {
            Ok(true) => Ok(Json(SimpleSuccessResponse {
                success: true,
                message: format!("Token {} logged in", id.0),
            })),
            Ok(false) => Err(poem::Error::from_string(
                "Invalid PIN",
                poem::http::StatusCode::UNAUTHORIZED,
            )),
            Err(e) => Err(poem::error::InternalServerError(e)),
        }
    }

    /// Logs out of a token.
    #[oai(path = "/v1/tokens/:id/logout", method = "put")]
    pub async fn logout_token(
        &self,
        id: Path<String>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        self.state
            .signer
            .logout_token(&id.0)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Token {} logged out", id.0),
        }))
    }

    /// Updates PIN for a token.
    #[oai(path = "/v1/tokens/:id/pin", method = "put")]
    pub async fn update_token_pin(
        &self,
        id: Path<String>,
        body: Json<TokenPinReq>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        self.state
            .signer
            .set_token_pin(&id.0, body.old_pin.as_deref(), &body.new_pin)
            .await
            .map_err(poem::error::BadRequest)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Token {} PIN updated", id.0),
        }))
    }

    /// Generates a new cryptographic key on a token.
    #[oai(path = "/v1/tokens/:id/keys", method = "post")]
    pub async fn generate_token_key(
        &self,
        id: Path<String>,
        body: Json<GenerateKeyReq>,
    ) -> poem::Result<Json<KeyDto>> {
        let usage = proto::KeyUsageInfo::Signing;
        let key_info = self
            .state
            .signer
            .generate_software_key(&id.0, &body.label, usage)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(KeyDto {
            id: key_info.id,
            label: key_info.label,
            friendly_name: key_info.friendly_name,
            usage: key_info.usage,
            available: key_info.available,
            public_key: key_info.public_key,
            sign_mechanism: key_info.sign_mechanism_name,
        }))
    }

    /// Lists all managed cryptographic keys.
    #[oai(path = "/v1/keys", method = "get")]
    pub async fn list_keys(&self) -> Json<Vec<KeyDto>> {
        let keys = self.state.signer.list_keys().await;
        let dtos = keys
            .into_iter()
            .map(|k| KeyDto {
                id: k.id,
                label: k.label,
                friendly_name: k.friendly_name,
                usage: k.usage,
                available: k.available,
                public_key: k.public_key,
                sign_mechanism: k.sign_mechanism_name,
            })
            .collect();
        Json(dtos)
    }

    /// Gets key details by ID.
    #[oai(path = "/v1/keys/:id", method = "get")]
    pub async fn get_key(&self, id: Path<String>) -> poem::Result<Json<KeyDto>> {
        let k = self.state.signer.get_key(&id.0).await.ok_or_else(|| {
            poem::Error::from_string("Key not found", poem::http::StatusCode::NOT_FOUND)
        })?;

        Ok(Json(KeyDto {
            id: k.id,
            label: k.label,
            friendly_name: k.friendly_name,
            usage: k.usage,
            available: k.available,
            public_key: k.public_key,
            sign_mechanism: k.sign_mechanism_name,
        }))
    }

    /// Deletes a key by ID.
    #[oai(path = "/v1/keys/:id", method = "delete")]
    pub async fn delete_key(&self, id: Path<String>) -> poem::Result<Json<SimpleSuccessResponse>> {
        self.state
            .signer
            .delete_key(&id.0)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Key {} deleted", id.0),
        }))
    }

    /// Generates a CSR for a key.
    #[oai(path = "/v1/keys/:id/csrs", method = "post")]
    pub async fn generate_csr(
        &self,
        id: Path<String>,
        body: Json<GenerateCsrReq>,
    ) -> poem::Result<Json<CsrDto>> {
        let csr_pem = self
            .state
            .signer
            .generate_csr(&id.0, &body.subject_cn)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(CsrDto {
            id: format!("csr-{}", uuid::Uuid::new_v4()),
            csr_pem,
            subject_name: body.subject_cn.clone(),
        }))
    }

    /// Imports a certificate for a key.
    #[oai(path = "/v1/token-certificates", method = "post")]
    pub async fn import_certificate(
        &self,
        body: Json<ImportCertReq>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        let der = base64::engine::general_purpose::STANDARD
            .decode(&body.cert_der_base64)
            .map_err(poem::error::BadRequest)?;

        self.state
            .signer
            .add_certificate(&body.key_id, der)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: "Certificate successfully imported".to_string(),
        }))
    }
}
