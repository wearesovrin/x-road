use crate::handlers::dto::{
    CsrDto, GenerateCsrReq, KeyDto, PossibleActionsDto, SimpleSuccessResponse, UpdateKeyReq,
};
use crate::handlers::{ApiTags, AppState};
use poem::web::Data;
use poem_openapi::{
    param::Path,
    payload::{Attachment, Json},
    OpenApi,
};

pub struct KeysApi;

#[OpenApi(tag = "ApiTags::Keys")]
impl KeysApi {
    #[oai(path = "/v1/keys", method = "get")]
    pub async fn list_keys(&self, state: Data<&AppState>) -> Json<Vec<KeyDto>> {
        let keys = state.signer.list_keys().await;
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
                possible_actions: vec!["DELETE".to_string()],
            })
            .collect();
        Json(dtos)
    }

    #[oai(path = "/v1/keys/:id", method = "get")]
    pub async fn get_key(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
    ) -> poem::Result<Json<KeyDto>> {
        let k = state.signer.get_key(&id.0).await.ok_or_else(|| {
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
            possible_actions: vec!["DELETE".to_string()],
        }))
    }

    #[oai(path = "/v1/keys/:id", method = "patch")]
    pub async fn update_key(
        &self,
        _state: Data<&AppState>,
        id: Path<String>,
        body: Json<UpdateKeyReq>,
    ) -> Json<KeyDto> {
        Json(KeyDto {
            id: id.0,
            label: Some("key-label".to_string()),
            friendly_name: Some(body.0.friendly_name),
            usage: 0,
            available: true,
            public_key: "pubkey".to_string(),
            sign_mechanism: "SHA256withRSA".to_string(),
            possible_actions: vec!["DELETE".to_string()],
        })
    }

    #[oai(path = "/v1/keys/:id", method = "delete")]
    pub async fn delete_key(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        state
            .signer
            .delete_key(&id.0)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Key {} deleted", id.0),
        }))
    }

    #[oai(path = "/v1/keys/:id/possible-actions", method = "get")]
    pub async fn get_possible_actions_for_key(
        &self,
        _state: Data<&AppState>,
        _id: Path<String>,
    ) -> Json<PossibleActionsDto> {
        Json(PossibleActionsDto {
            actions: vec!["DELETE".to_string(), "GENERATE_CSR".to_string()],
        })
    }

    #[oai(path = "/v1/keys/:id/csrs", method = "post")]
    pub async fn generate_csr(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
        body: Json<GenerateCsrReq>,
    ) -> poem::Result<Json<CsrDto>> {
        let csr_pem = state
            .signer
            .generate_csr(&id.0, &body.subject_cn)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(CsrDto {
            id: format!("csr-{}", uuid::Uuid::new_v4()),
            csr_pem,
            subject_name: body.subject_cn.clone(),
            possible_actions: vec!["DELETE".to_string()],
        }))
    }

    #[oai(path = "/v1/keys/:id/csrs/:csr_id", method = "get")]
    pub async fn download_csr(
        &self,
        _state: Data<&AppState>,
        _id: Path<String>,
        csr_id: Path<String>,
    ) -> Attachment<Vec<u8>> {
        let pem = "-----BEGIN CERTIFICATE REQUEST-----\n...\n-----END CERTIFICATE REQUEST-----";
        Attachment::new(pem.as_bytes().to_vec()).filename(format!("{}.csr", csr_id.0))
    }

    #[oai(path = "/v1/keys/:id/csrs/:csr_id", method = "delete")]
    pub async fn delete_csr(
        &self,
        _state: Data<&AppState>,
        _id: Path<String>,
        _csr_id: Path<String>,
    ) -> Json<bool> {
        Json(true)
    }

    #[oai(path = "/v1/keys/:id/csrs/:csr_id/possible-actions", method = "get")]
    pub async fn get_possible_actions_for_csr(
        &self,
        _state: Data<&AppState>,
        _id: Path<String>,
        _csr_id: Path<String>,
    ) -> Json<PossibleActionsDto> {
        Json(PossibleActionsDto {
            actions: vec!["DELETE".to_string()],
        })
    }
}
