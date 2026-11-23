use crate::handlers::dto::{AgreementTokenKeyDto, DataspaceProvisioningStatusDto};
use crate::handlers::{ApiTags, AppState};
use poem::web::Data;
use poem_openapi::{payload::Json, OpenApi};

pub struct DataspaceApi;

#[OpenApi(tag = "ApiTags::Dataspace")]
impl DataspaceApi {
    #[oai(path = "/v1/dataspace/provisioning-status", method = "get")]
    pub async fn get_dataspace_provisioning_status(
        &self,
        _state: Data<&AppState>,
    ) -> Json<DataspaceProvisioningStatusDto> {
        Json(DataspaceProvisioningStatusDto {
            status: "PROVISIONED".to_string(),
            message: Some("Dataspace participant context ready".to_string()),
        })
    }

    #[oai(path = "/v1/dataspace/agreement-token-key", method = "get")]
    pub async fn get_agreement_token_key_status(
        &self,
        _state: Data<&AppState>,
    ) -> Json<AgreementTokenKeyDto> {
        Json(AgreementTokenKeyDto {
            key_id: "agreement-token-key-1".to_string(),
            status: "ACTIVE".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    #[oai(path = "/v1/dataspace/agreement-token-key/rotate", method = "post")]
    pub async fn rotate_agreement_token_key(
        &self,
        _state: Data<&AppState>,
    ) -> Json<AgreementTokenKeyDto> {
        Json(AgreementTokenKeyDto {
            key_id: "agreement-token-key-2".to_string(),
            status: "ACTIVE".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }
}
