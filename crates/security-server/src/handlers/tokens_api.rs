use crate::handlers::dto::{
    GenerateKeyReq, KeyDto, KeyWithCsrsReq, SimpleSuccessResponse, TokenDto, TokenLoginReq,
    TokenPinReq, UpdateTokenReq,
};
use crate::handlers::{ApiTags, AppState};
use poem::web::Data;
use poem_openapi::{param::Path, payload::Json, OpenApi};

pub struct TokensApi;

#[OpenApi(tag = "ApiTags::Tokens")]
impl TokensApi {
    #[oai(path = "/v1/tokens", method = "get")]
    pub async fn list_tokens(&self, state: Data<&AppState>) -> Json<Vec<TokenDto>> {
        let tokens = state.signer.list_tokens().await;
        let dtos = tokens
            .into_iter()
            .map(|t| TokenDto {
                id: t.id,
                name: t.name,
                token_type: t.token_type.to_string(),
                logged_in: t.logged_in,
                has_pin: t.has_pin,
                possible_actions: vec!["LOGIN".to_string(), "LOGOUT".to_string()],
            })
            .collect();
        Json(dtos)
    }

    #[oai(path = "/v1/tokens/:id", method = "get")]
    pub async fn get_token(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
    ) -> poem::Result<Json<TokenDto>> {
        let tokens = state.signer.list_tokens().await;
        let t = tokens
            .into_iter()
            .find(|tok| tok.id == id.0)
            .ok_or_else(|| {
                poem::Error::from_string("Token not found", poem::http::StatusCode::NOT_FOUND)
            })?;
        Ok(Json(TokenDto {
            id: t.id,
            name: t.name,
            token_type: t.token_type.to_string(),
            logged_in: t.logged_in,
            has_pin: t.has_pin,
            possible_actions: vec!["LOGIN".to_string(), "LOGOUT".to_string()],
        }))
    }

    #[oai(path = "/v1/tokens/:id", method = "delete")]
    pub async fn delete_token(
        &self,
        _state: Data<&AppState>,
        _id: Path<String>,
    ) -> Json<bool> {
        Json(true)
    }

    #[oai(path = "/v1/tokens/:id", method = "patch")]
    pub async fn update_token(
        &self,
        _state: Data<&AppState>,
        id: Path<String>,
        body: Json<UpdateTokenReq>,
    ) -> Json<TokenDto> {
        Json(TokenDto {
            id: id.0,
            name: body.0.name,
            token_type: "SoftwareToken".to_string(),
            logged_in: true,
            has_pin: true,
            possible_actions: vec!["LOGOUT".to_string()],
        })
    }

    #[oai(path = "/v1/tokens/:id/login", method = "put")]
    pub async fn login_token(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
        body: Json<TokenLoginReq>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        match state.signer.login_token(&id.0, &body.pin).await {
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

    #[oai(path = "/v1/tokens/:id/logout", method = "put")]
    pub async fn logout_token(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        state
            .signer
            .logout_token(&id.0)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Token {} logged out", id.0),
        }))
    }

    #[oai(path = "/v1/tokens/:id/pin", method = "put")]
    pub async fn update_token_pin(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
        body: Json<TokenPinReq>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        state
            .signer
            .set_token_pin(&id.0, body.old_pin.as_deref(), &body.new_pin)
            .await
            .map_err(poem::error::BadRequest)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Token {} PIN updated", id.0),
        }))
    }

    #[oai(path = "/v1/tokens/:id/keys", method = "post")]
    pub async fn generate_token_key(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
        body: Json<GenerateKeyReq>,
    ) -> poem::Result<Json<KeyDto>> {
        let usage = proto::KeyUsageInfo::Signing;
        let key_info = state
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
            possible_actions: vec!["DELETE".to_string()],
        }))
    }

    #[oai(path = "/v1/tokens/:id/keys-with-csrs", method = "post")]
    pub async fn add_key_and_csr(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
        body: Json<KeyWithCsrsReq>,
    ) -> poem::Result<Json<KeyDto>> {
        let usage = proto::KeyUsageInfo::Signing;
        let key_info = state
            .signer
            .generate_software_key(&id.0, &body.0.label, usage)
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
            possible_actions: vec!["DELETE".to_string()],
        }))
    }
}
