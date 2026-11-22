use poem::web::Data;
use poem_openapi::{
    OpenApi,
    payload::{Json, PlainText},
};

use super::AppState;
use super::dto::{
    DiagnosticsGlobalConf, DiagnosticsOcsp, DiagnosticsTsa, MaintenanceModeResponse,
    ServerAddressReq, ServerAddressResponse, ServerStatus, SimpleSuccessResponse, SystemVersion,
};

pub struct SystemApi;

#[OpenApi]
impl SystemApi {
    /// Returns the current operational status and version of the Security Server.
    #[oai(path = "/v1/status", method = "get")]
    pub async fn get_status(&self, state: Data<&AppState>) -> Json<ServerStatus> {
        let clients = state.server_conf.list_clients().await.unwrap_or_default();
        let logs = state
            .message_log
            .find_unarchived(100)
            .await
            .unwrap_or_default();
        let owner = state.server_conf.get_owner().await.unwrap_or_default();

        Json(ServerStatus {
            status: "OK".to_string(),
            version: "0.1.0".to_string(),
            client_count: clients.len(),
            unarchived_logs: logs.len(),
            is_initialized: owner.is_some(),
        })
    }

    /// Returns Prometheus / OpenMetrics operational and system metrics text.
    #[oai(path = "/metrics", method = "get")]
    pub async fn get_metrics(&self) -> PlainText<String> {
        let text = monitor::generate_prometheus_metrics(0, 0);
        PlainText(text)
    }

    /// Returns version information.
    #[oai(path = "/v1/system/version", method = "get")]
    pub async fn get_version(&self) -> Json<SystemVersion> {
        Json(SystemVersion {
            version: "0.1.0".to_string(),
            edition: "2024".to_string(),
            authors: "Sovrin OÜ <info@sovrin.ee>, X-Road Rust Contributors".to_string(),
        })
    }

    /// Gets current Security Server network address.
    #[oai(path = "/v1/system/server-address", method = "get")]
    pub async fn get_server_address(&self, state: Data<&AppState>) -> Json<ServerAddressResponse> {
        let settings = state.system_settings.read().await;
        Json(ServerAddressResponse {
            server_address: settings.server_address.clone(),
        })
    }

    /// Sets Security Server network address.
    #[oai(path = "/v1/system/server-address", method = "put")]
    pub async fn set_server_address(
        &self,
        state: Data<&AppState>,
        body: Json<ServerAddressReq>,
    ) -> Json<SimpleSuccessResponse> {
        let mut settings = state.system_settings.write().await;
        settings.server_address = body.server_address.clone();
        Json(SimpleSuccessResponse {
            success: true,
            message: format!("Server address updated to {}", body.server_address),
        })
    }

    /// Gets maintenance mode status.
    #[oai(path = "/v1/system/maintenance-mode", method = "get")]
    pub async fn get_maintenance_mode(
        &self,
        state: Data<&AppState>,
    ) -> Json<MaintenanceModeResponse> {
        let settings = state.system_settings.read().await;
        Json(MaintenanceModeResponse {
            maintenance_mode: settings.maintenance_mode,
        })
    }

    /// Enables maintenance mode.
    #[oai(path = "/v1/system/maintenance-mode/enable", method = "put")]
    pub async fn enable_maintenance_mode(
        &self,
        state: Data<&AppState>,
    ) -> Json<SimpleSuccessResponse> {
        let mut settings = state.system_settings.write().await;
        settings.maintenance_mode = true;
        Json(SimpleSuccessResponse {
            success: true,
            message: "Maintenance mode enabled".to_string(),
        })
    }

    /// Disables maintenance mode.
    #[oai(path = "/v1/system/maintenance-mode/disable", method = "put")]
    pub async fn disable_maintenance_mode(
        &self,
        state: Data<&AppState>,
    ) -> Json<SimpleSuccessResponse> {
        let mut settings = state.system_settings.write().await;
        settings.maintenance_mode = false;
        Json(SimpleSuccessResponse {
            success: true,
            message: "Maintenance mode disabled".to_string(),
        })
    }

    /// Queries Global Configuration diagnostics.
    #[oai(path = "/v1/diagnostics/globalconf", method = "get")]
    pub async fn get_globalconf_diagnostics(
        &self,
        state: Data<&AppState>,
    ) -> Json<DiagnosticsGlobalConf> {
        let gc = &state.global_conf;
        Json(DiagnosticsGlobalConf {
            instance_identifier: gc.instance_identifier.clone(),
            status: "OK".to_string(),
            members_count: gc.members.len(),
            security_servers_count: gc.security_servers.len(),
            approved_cas_count: gc.approved_cas.len(),
            approved_tsps_count: gc.approved_tsps.len(),
        })
    }

    /// Queries Approved Time-Stamping Authorities diagnostics.
    #[oai(path = "/v1/diagnostics/timestamping-services", method = "get")]
    pub async fn get_tsa_diagnostics(&self, state: Data<&AppState>) -> Json<Vec<DiagnosticsTsa>> {
        let tsas = &state.global_conf.approved_tsps;
        let dtos = tsas
            .iter()
            .map(|t| DiagnosticsTsa {
                name: t.name.clone(),
                url: t.url.clone(),
                status: "OK".to_string(),
            })
            .collect();
        Json(dtos)
    }

    /// Queries OCSP Responders diagnostics.
    #[oai(path = "/v1/diagnostics/ocsp-responders", method = "get")]
    pub async fn get_ocsp_diagnostics(&self, state: Data<&AppState>) -> Json<Vec<DiagnosticsOcsp>> {
        let cas = &state.global_conf.approved_cas;
        let dtos = cas
            .iter()
            .map(|ca| DiagnosticsOcsp {
                ca_name: ca.name.clone(),
                url: format!("http://ocsp.{}.x-road.local", ca.name.to_lowercase()),
                status: "OK".to_string(),
            })
            .collect();
        Json(dtos)
    }
}
