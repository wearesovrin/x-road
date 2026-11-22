use crate::handlers::AppState;
use crate::handlers::dto::{CentralServerInfo, ServerAddressDto, SystemVersionDto};
use poem::{http::StatusCode, web::Data};
use poem_openapi::{OpenApi, payload::Json};

pub struct SystemApi;

#[OpenApi]
impl SystemApi {
    #[oai(path = "/v1/system/server-address", method = "get")]
    pub async fn get_server_address(
        &self,
        state: Data<&AppState>,
    ) -> poem::Result<Json<ServerAddressDto>> {
        let addr = state.db.get_server_address().await.map_err(|e| {
            poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
        })?;
        Ok(Json(ServerAddressDto {
            server_address: addr,
        }))
    }

    #[oai(path = "/v1/system/server-address", method = "put")]
    pub async fn update_server_address(
        &self,
        state: Data<&AppState>,
        body: Json<ServerAddressDto>,
    ) -> poem::Result<Json<ServerAddressDto>> {
        state
            .db
            .set_server_address(&body.0.server_address)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;
        Ok(Json(ServerAddressDto {
            server_address: body.0.server_address,
        }))
    }

    #[oai(path = "/v1/system/status", method = "get")]
    pub async fn system_status(&self, state: Data<&AppState>) -> Json<CentralServerInfo> {
        self.status(state).await
    }

    #[oai(path = "/v1/status", method = "get")]
    pub async fn status(&self, state: Data<&AppState>) -> Json<CentralServerInfo> {
        let members = state.db.list_members().await.unwrap_or_default();
        let servers = state.db.list_security_servers().await.unwrap_or_default();

        Json(CentralServerInfo {
            instance_identifier: state.instance_id.clone(),
            status: "READY".to_string(),
            member_count: members.len(),
            security_server_count: servers.len(),
        })
    }

    #[oai(path = "/v1/system/version", method = "get")]
    pub async fn system_version(&self) -> Json<SystemVersionDto> {
        Json(SystemVersionDto {
            version: "7.5.0-rust".to_string(),
        })
    }
}
