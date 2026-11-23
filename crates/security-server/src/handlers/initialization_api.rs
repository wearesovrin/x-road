use identifiers::ClientId;
use poem::web::Data;
use poem_openapi::{OpenApi, payload::Json};

use super::{ApiTags, AppState};
use super::dto::{InitializationStatus, InitializeServerReq, SimpleSuccessResponse};

pub struct InitializationApi;

#[OpenApi(tag = "ApiTags::Initialization")]
impl InitializationApi {
    /// Checks if the Security Server has been initialized.
    #[oai(path = "/v1/initialization/status", method = "get")]
    pub async fn get_initialization_status(
        &self,
        state: Data<&AppState>,
    ) -> Json<InitializationStatus> {
        let owner = state.server_conf.get_owner().await.unwrap_or_default();
        let token = state.signer.get_token("0").await;

        Json(InitializationStatus {
            is_initialized: owner.is_some(),
            has_software_token_pin: token.as_ref().map(|t| t.has_pin).unwrap_or(false),
            owner_client_id: owner.map(|o| {
                if let Some(ref sub) = o.subsystem_code {
                    format!(
                        "{}/{}/{}/{}",
                        o.xroad_instance, o.member_class, o.member_code, sub
                    )
                } else {
                    format!("{}/{}/{}", o.xroad_instance, o.member_class, o.member_code)
                }
            }),
        })
    }

    /// Initializes the Security Server with an owner client and initial PIN.
    #[oai(path = "/v1/initialization", method = "post")]
    pub async fn initialize_server(
        &self,
        state: Data<&AppState>,
        req: Json<InitializeServerReq>,
    ) -> Json<SimpleSuccessResponse> {
        let r = req.0;
        let client_id = if let Some(ref sub) = r.subsystem_code {
            ClientId::subsystem_unchecked(&r.instance, &r.member_class, &r.member_code, sub)
        } else {
            ClientId::member_unchecked(&r.instance, &r.member_class, &r.member_code)
        };

        if let Err(e) = state.server_conf.add_client(&client_id, true).await {
            return Json(SimpleSuccessResponse {
                success: false,
                message: format!("Failed to register owner client: {e}"),
            });
        }
        let _ = state.server_conf.make_owner(&client_id).await;
        let _ = state
            .signer
            .set_token_pin("0", None, &r.software_token_pin)
            .await;

        Json(SimpleSuccessResponse {
            success: true,
            message: format!("Security Server successfully initialized for owner {client_id}"),
        })
    }
}
