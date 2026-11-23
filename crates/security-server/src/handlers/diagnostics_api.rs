use crate::handlers::dto::{
    DiagnosticsGlobalConf, DiagnosticsMemoryUsageDto, DiagnosticsOcsp,
    DiagnosticsOperationalMonitoringDto, DiagnosticsStatusDto, DiagnosticsTsa,
};
use crate::handlers::{ApiTags, AppState};
use poem::web::Data;
use poem_openapi::{
    payload::{Attachment, Json},
    OpenApi,
};

pub struct DiagnosticsApi;

#[OpenApi(tag = "ApiTags::Diagnostics")]
impl DiagnosticsApi {
    #[oai(path = "/v1/diagnostics/globalconf", method = "get")]
    pub async fn get_globalconf_diagnostics(
        &self,
        state: Data<&AppState>,
    ) -> Json<DiagnosticsGlobalConf> {
        let gconf = &state.global_conf;
        Json(DiagnosticsGlobalConf {
            instance_identifier: gconf.instance_identifier.clone(),
            status: "OK".to_string(),
            members_count: gconf.members.len(),
            security_servers_count: gconf.security_servers.len(),
            approved_cas_count: gconf.approved_cas.len(),
            approved_tsps_count: gconf.approved_tsps.len(),
        })
    }

    #[oai(path = "/v1/diagnostics/global-conf-status", method = "get")]
    pub async fn get_global_conf_status(
        &self,
        _state: Data<&AppState>,
    ) -> Json<DiagnosticsStatusDto> {
        Json(DiagnosticsStatusDto {
            status: "OK".to_string(),
            description: Some("Global configuration is up to date".to_string()),
        })
    }

    #[oai(path = "/v1/diagnostics/ocsp-responders", method = "get")]
    pub async fn get_ocsp_diagnostics(
        &self,
        state: Data<&AppState>,
    ) -> Json<Vec<DiagnosticsOcsp>> {
        let mut list = Vec::new();
        for ca in &state.global_conf.approved_cas {
            list.push(DiagnosticsOcsp {
                ca_name: ca.name.clone(),
                url: format!("http://ocsp.{}.example.org", ca.name.to_lowercase()),
                status: "OK".to_string(),
                last_response: chrono::Utc::now().to_rfc3339(),
            });
        }
        Json(list)
    }

    #[oai(path = "/v1/diagnostics/timestamping-services", method = "get")]
    pub async fn get_tsa_diagnostics(
        &self,
        state: Data<&AppState>,
    ) -> Json<Vec<DiagnosticsTsa>> {
        let list = state
            .global_conf
            .approved_tsps
            .iter()
            .map(|t| DiagnosticsTsa {
                name: t.name.clone(),
                url: t.url.clone(),
                status: "OK".to_string(),
                last_response: chrono::Utc::now().to_rfc3339(),
            })
            .collect();
        Json(list)
    }

    #[oai(path = "/v1/diagnostics/addon-status", method = "get")]
    pub async fn get_addon_diagnostics(
        &self,
        _state: Data<&AppState>,
    ) -> Json<DiagnosticsStatusDto> {
        Json(DiagnosticsStatusDto {
            status: "OK".to_string(),
            description: Some("Addons healthy".to_string()),
        })
    }

    #[oai(path = "/v1/diagnostics/backup-encryption-status", method = "get")]
    pub async fn get_backup_encryption_diagnostics(
        &self,
        _state: Data<&AppState>,
    ) -> Json<DiagnosticsStatusDto> {
        Json(DiagnosticsStatusDto {
            status: "OK".to_string(),
            description: Some("Backup encryption active".to_string()),
        })
    }

    #[oai(path = "/v1/diagnostics/message-log-encryption-status", method = "get")]
    pub async fn get_messagelog_encryption_diagnostics(
        &self,
        _state: Data<&AppState>,
    ) -> Json<DiagnosticsStatusDto> {
        Json(DiagnosticsStatusDto {
            status: "OK".to_string(),
            description: Some("Message log encryption active".to_string()),
        })
    }

    #[oai(path = "/v1/diagnostics/proxy-memory-usage-status", method = "get")]
    pub async fn get_proxy_memory_usage(
        &self,
        _state: Data<&AppState>,
    ) -> Json<DiagnosticsMemoryUsageDto> {
        Json(DiagnosticsMemoryUsageDto {
            total_memory: 1024 * 1024 * 512,
            free_memory: 1024 * 1024 * 256,
            max_memory: 1024 * 1024 * 1024,
        })
    }

    #[oai(path = "/v1/diagnostics/auth-cert-req-status", method = "get")]
    pub async fn get_auth_cert_req_status(
        &self,
        _state: Data<&AppState>,
    ) -> Json<DiagnosticsStatusDto> {
        Json(DiagnosticsStatusDto {
            status: "OK".to_string(),
            description: Some("Auth cert requests healthy".to_string()),
        })
    }

    #[oai(path = "/v1/diagnostics/other-security-server-status", method = "get")]
    pub async fn get_other_security_server_status(
        &self,
        _state: Data<&AppState>,
    ) -> Json<DiagnosticsStatusDto> {
        Json(DiagnosticsStatusDto {
            status: "OK".to_string(),
            description: Some("All cluster nodes reachable".to_string()),
        })
    }

    #[oai(path = "/v1/diagnostics/operational-monitoring", method = "get")]
    pub async fn get_operational_monitoring(
        &self,
        _state: Data<&AppState>,
    ) -> Json<DiagnosticsOperationalMonitoringDto> {
        Json(DiagnosticsOperationalMonitoringDto {
            interval_seconds: 300,
            is_enabled: true,
        })
    }

    #[oai(path = "/v1/diagnostics/info/download", method = "get")]
    pub async fn download_diagnostics_report(
        &self,
        _state: Data<&AppState>,
    ) -> Attachment<Vec<u8>> {
        let report = b"Diagnostics Report: All services OK\n";
        Attachment::new(report.to_vec()).filename("diagnostics.txt")
    }
}
