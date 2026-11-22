use poem::web::Data;
use poem_openapi::{OpenApi, param::Path, payload::Json};
use sha2::{Digest, Sha256};

use super::AppState;
use super::dto::{
    ApiKeyDto, ApiKeyResponse, CreateApiKeyReq, UpdateApiKeyReq, ValidateApiKeyReq,
    ValidateApiKeyResponse,
};

pub struct ApiKeysApi;

#[OpenApi]
impl ApiKeysApi {
    /// Creates an API key with assigned roles. Returns plaintext key token.
    #[oai(path = "/v1/api-keys", method = "post")]
    pub async fn create_api_key(
        &self,
        state: Data<&AppState>,
        req: Json<CreateApiKeyReq>,
    ) -> Json<ApiKeyResponse> {
        let plaintext_key = req
            .key
            .clone()
            .filter(|k| !k.is_empty())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let encoded_key = format!("{:x}", Sha256::digest(plaintext_key.as_bytes()));
        let roles: Vec<&str> = req.roles.iter().map(String::as_str).collect();

        let key_id = state
            .server_conf
            .create_api_key(&encoded_key, req.label.as_deref(), &roles)
            .await
            .unwrap_or_default();

        let all_keys = state.server_conf.list_api_keys().await.unwrap_or_default();
        let db_id = all_keys
            .iter()
            .find(|(k, _)| k.key_id == key_id)
            .map(|(k, _)| k.id)
            .unwrap_or(0);

        Json(ApiKeyResponse {
            id: db_id,
            key_id,
            key: Some(plaintext_key),
            label: req.label.clone(),
            roles: req.roles.clone(),
        })
    }

    /// Lists all registered API keys.
    #[oai(path = "/v1/api-keys", method = "get")]
    pub async fn list_api_keys(&self, state: Data<&AppState>) -> Json<Vec<ApiKeyDto>> {
        let keys = state.server_conf.list_api_keys().await.unwrap_or_default();
        let list = keys
            .into_iter()
            .map(|(k, roles)| ApiKeyDto {
                id: k.id,
                key_id: k.key_id,
                key: None,
                label: k.label,
                roles,
            })
            .collect();
        Json(list)
    }

    /// Gets an API key and its assigned roles by database ID.
    #[oai(path = "/v1/api-keys/:id", method = "get")]
    pub async fn get_api_key(
        &self,
        state: Data<&AppState>,
        id: Path<i64>,
    ) -> poem::Result<Json<ApiKeyDto>> {
        match state.server_conf.get_api_key(id.0).await {
            Ok(Some((k, roles))) => Ok(Json(ApiKeyDto {
                id: k.id,
                key_id: k.key_id,
                key: None,
                label: k.label,
                roles,
            })),
            _ => Err(poem::error::NotFoundError.into()),
        }
    }

    /// Updates the assigned roles of an API key.
    #[oai(path = "/v1/api-keys/:id", method = "put")]
    pub async fn update_api_key(
        &self,
        state: Data<&AppState>,
        id: Path<i64>,
        req: Json<UpdateApiKeyReq>,
    ) -> poem::Result<Json<ApiKeyDto>> {
        let role_strs: Vec<&str> = req.roles.iter().map(String::as_str).collect();
        match state
            .server_conf
            .update_api_key_roles(id.0, &role_strs)
            .await
        {
            Ok(()) => match state.server_conf.get_api_key(id.0).await {
                Ok(Some((k, roles))) => Ok(Json(ApiKeyDto {
                    id: k.id,
                    key_id: k.key_id,
                    key: None,
                    label: k.label,
                    roles,
                })),
                _ => Err(poem::error::NotFoundError.into()),
            },
            Err(e) => Err(poem::error::BadRequest(e).into()),
        }
    }

    /// Revokes (deletes) an API key by database ID.
    #[oai(path = "/v1/api-keys/:id", method = "delete")]
    pub async fn revoke_api_key(&self, state: Data<&AppState>, id: Path<i64>) -> Json<bool> {
        let _ = state.server_conf.revoke_api_key(id.0).await;
        Json(true)
    }

    /// Validates an API key and returns assigned roles.
    #[oai(path = "/v1/api-keys/validate", method = "post")]
    pub async fn validate_api_key(
        &self,
        state: Data<&AppState>,
        req: Json<ValidateApiKeyReq>,
    ) -> Json<ValidateApiKeyResponse> {
        match state.server_conf.validate_api_key(&req.key).await {
            Ok(Some(roles)) => Json(ValidateApiKeyResponse { valid: true, roles }),
            _ => Json(ValidateApiKeyResponse {
                valid: false,
                roles: vec![],
            }),
        }
    }
}
