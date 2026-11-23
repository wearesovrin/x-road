use crate::handlers::dto::{
    CentralServerInfo, HighAvailabilityClusterStatusDto, ServerAddressDto, SystemVersionDto,
};
use crate::handlers::{ApiTags, AppState};
use poem::{http::StatusCode, web::Data};
use poem_openapi::{payload::Json, OpenApi};

pub struct SystemApi;

#[OpenApi(tag = "ApiTags::System")]
impl SystemApi {
    #[oai(path = "/v1/system/server-address", method = "get")]
    pub async fn get_server_address(
        &self,
        state: Data<&AppState>,
    ) -> poem::Result<Json<ServerAddressDto>> {
        let addr = state
            .db
            .get_server_address()
            .await
            .unwrap_or_else(|_| "127.0.0.1:4000".to_string());
        Ok(Json(ServerAddressDto {
            server_address: addr,
        }))
    }

    #[oai(path = "/v1/system/server-address", method = "put")]
    pub async fn set_server_address(
        &self,
        state: Data<&AppState>,
        body: Json<ServerAddressDto>,
    ) -> poem::Result<Json<ServerAddressDto>> {
        state
            .db
            .set_server_address(&body.0.server_address)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;
        Ok(body)
    }

    #[oai(path = "/v1/status", method = "get")]
    pub async fn get_status_compat(&self, state: Data<&AppState>) -> Json<CentralServerInfo> {
        self.get_system_status(state).await
    }

    #[oai(path = "/v1/system/status", method = "get")]
    pub async fn get_system_status(&self, state: Data<&AppState>) -> Json<CentralServerInfo> {
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
    pub async fn get_version(&self) -> Json<SystemVersionDto> {
        Json(SystemVersionDto {
            version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }

    #[oai(path = "/v1/system/high-availability-cluster/status", method = "get")]
    pub async fn get_ha_cluster_status(&self) -> Json<HighAvailabilityClusterStatusDto> {
        Json(HighAvailabilityClusterStatusDto {
            status: "OK".to_string(),
            nodes: vec!["node-1 (leader)".to_string()],
        })
    }
}
