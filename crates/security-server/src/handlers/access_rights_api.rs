use identifiers::{ClientId, ServiceId};
use poem::web::Data;
use poem_openapi::{OpenApi, payload::Json};

use super::AppState;
use super::dto::{AccessRightReq, SimpleSuccessResponse};

pub struct AccessRightsApi;

#[OpenApi]
impl AccessRightsApi {
    /// Grants access for a client to a service.
    #[oai(path = "/v1/access-rights", method = "post")]
    pub async fn grant_access(
        &self,
        state: Data<&AppState>,
        req: Json<AccessRightReq>,
    ) -> Json<SimpleSuccessResponse> {
        let service_id: ServiceId = match req.service_id.parse() {
            Ok(s) => s,
            Err(e) => {
                return Json(SimpleSuccessResponse {
                    success: false,
                    message: format!("Invalid service_id: {e}"),
                });
            }
        };
        let client_id: ClientId = match req.client_id.parse() {
            Ok(c) => c,
            Err(e) => {
                return Json(SimpleSuccessResponse {
                    success: false,
                    message: format!("Invalid client_id: {e}"),
                });
            }
        };

        match state
            .server_conf
            .grant_access(&service_id, &client_id)
            .await
        {
            Ok(_) => Json(SimpleSuccessResponse {
                success: true,
                message: format!("Access granted for {client_id} to {service_id}"),
            }),
            Err(e) => Json(SimpleSuccessResponse {
                success: false,
                message: e.to_string(),
            }),
        }
    }

    /// Revokes access for a client to a service.
    #[oai(path = "/v1/access-rights", method = "delete")]
    pub async fn revoke_access(
        &self,
        state: Data<&AppState>,
        req: Json<AccessRightReq>,
    ) -> Json<SimpleSuccessResponse> {
        let service_id: ServiceId = match req.service_id.parse() {
            Ok(s) => s,
            Err(e) => {
                return Json(SimpleSuccessResponse {
                    success: false,
                    message: format!("Invalid service_id: {e}"),
                });
            }
        };
        let client_id: ClientId = match req.client_id.parse() {
            Ok(c) => c,
            Err(e) => {
                return Json(SimpleSuccessResponse {
                    success: false,
                    message: format!("Invalid client_id: {e}"),
                });
            }
        };

        match state
            .server_conf
            .revoke_access(&service_id, &client_id)
            .await
        {
            Ok(_) => Json(SimpleSuccessResponse {
                success: true,
                message: format!("Access revoked for {client_id} to {service_id}"),
            }),
            Err(e) => Json(SimpleSuccessResponse {
                success: false,
                message: e.to_string(),
            }),
        }
    }
}
