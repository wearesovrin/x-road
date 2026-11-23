use poem::web::Data;
use poem_openapi::{
    payload::{Attachment, Json, PlainText},
    OpenApi,
};

use super::dto::{
    AuthProviderTypeDto, CertificateDetailsDto, MailNotificationStatusDto,
    MaintenanceModeResponse, NodeTypeDto, PrioritizationStrategyDto, SendTestMailReq,
    ServerAddressReq, ServerAddressResponse, ServerStatus, SimpleSuccessResponse, SystemVersion,
    TimestampingServiceConfigDto,
};
use super::{ApiTags, AppState};

pub struct SystemApi;

#[OpenApi(tag = "ApiTags::System")]
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

    #[oai(path = "/v1/system/node-type", method = "get")]
    pub async fn get_node_type(&self) -> Json<NodeTypeDto> {
        Json(NodeTypeDto {
            node_type: "STANDALONE".to_string(),
        })
    }

    #[oai(path = "/v1/system/auth-provider-type", method = "get")]
    pub async fn get_auth_provider_type(&self) -> Json<AuthProviderTypeDto> {
        Json(AuthProviderTypeDto {
            auth_provider_type: "INTERNAL".to_string(),
        })
    }

    #[oai(path = "/v1/system/anchor", method = "get")]
    pub async fn get_anchor(&self, _state: Data<&AppState>) -> Attachment<Vec<u8>> {
        let xml = "<anchor><instance>CS</instance></anchor>";
        Attachment::new(xml.as_bytes().to_vec()).filename("configuration_anchor.xml")
    }

    #[oai(path = "/v1/system/anchor", method = "post")]
    pub async fn upload_initial_anchor(&self, _state: Data<&AppState>) -> Json<bool> {
        Json(true)
    }

    #[oai(path = "/v1/system/anchor", method = "put")]
    pub async fn replace_anchor(&self, _state: Data<&AppState>) -> Json<bool> {
        Json(true)
    }

    #[oai(path = "/v1/system/anchor/previews", method = "post")]
    pub async fn preview_anchor(&self, _state: Data<&AppState>) -> Json<bool> {
        Json(true)
    }

    #[oai(path = "/v1/system/anchor/download", method = "get")]
    pub async fn download_anchor(&self, _state: Data<&AppState>) -> Attachment<Vec<u8>> {
        let xml = "<anchor><instance>CS</instance></anchor>";
        Attachment::new(xml.as_bytes().to_vec()).filename("configuration_anchor.xml")
    }

    #[oai(path = "/v1/system/certificate", method = "get")]
    pub async fn get_system_certificate(
        &self,
        _state: Data<&AppState>,
    ) -> Json<CertificateDetailsDto> {
        Json(CertificateDetailsDto {
            hash: "system-cert-hash".to_string(),
            issuer_distinguished_name: "CN=Security Server Internal CA".to_string(),
            subject_distinguished_name: "CN=Security Server Internal TLS".to_string(),
            not_before: "2020-01-01T00:00:00Z".to_string(),
            not_after: "2030-01-01T00:00:00Z".to_string(),
        })
    }

    #[oai(path = "/v1/system/certificate", method = "post")]
    pub async fn generate_system_certificate(
        &self,
        _state: Data<&AppState>,
    ) -> Json<CertificateDetailsDto> {
        Json(CertificateDetailsDto {
            hash: "new-system-cert-hash".to_string(),
            issuer_distinguished_name: "CN=Security Server Internal CA".to_string(),
            subject_distinguished_name: "CN=Security Server Internal TLS".to_string(),
            not_before: "2020-01-01T00:00:00Z".to_string(),
            not_after: "2030-01-01T00:00:00Z".to_string(),
        })
    }

    #[oai(path = "/v1/system/certificate/export", method = "get")]
    pub async fn download_system_certificate(
        &self,
        _state: Data<&AppState>,
    ) -> Attachment<Vec<u8>> {
        let cert = "-----BEGIN CERTIFICATE-----\n...\n-----END CERTIFICATE-----";
        Attachment::new(cert.as_bytes().to_vec()).filename("internal.crt")
    }

    #[oai(path = "/v1/system/certificate/csr", method = "post")]
    pub async fn generate_system_certificate_request(
        &self,
        _state: Data<&AppState>,
    ) -> PlainText<String> {
        PlainText("-----BEGIN CERTIFICATE REQUEST-----\n...\n-----END CERTIFICATE REQUEST-----".to_string())
    }

    #[oai(path = "/v1/system/certificate/import", method = "post")]
    pub async fn import_system_certificate(
        &self,
        _state: Data<&AppState>,
    ) -> Json<CertificateDetailsDto> {
        Json(CertificateDetailsDto {
            hash: "imported-system-cert-hash".to_string(),
            issuer_distinguished_name: "CN=Internal CA".to_string(),
            subject_distinguished_name: "CN=Internal TLS".to_string(),
            not_before: "2020-01-01T00:00:00Z".to_string(),
            not_after: "2030-01-01T00:00:00Z".to_string(),
        })
    }

    #[oai(path = "/v1/system/external-rpc-ca", method = "get")]
    pub async fn get_external_rpc_ca(
        &self,
        _state: Data<&AppState>,
    ) -> Json<CertificateDetailsDto> {
        Json(CertificateDetailsDto {
            hash: "rpc-ca-hash".to_string(),
            issuer_distinguished_name: "CN=External RPC CA".to_string(),
            subject_distinguished_name: "CN=External RPC".to_string(),
            not_before: "2020-01-01T00:00:00Z".to_string(),
            not_after: "2030-01-01T00:00:00Z".to_string(),
        })
    }

    #[oai(path = "/v1/system/external-rpc-ca/download", method = "get")]
    pub async fn download_external_rpc_ca(
        &self,
        _state: Data<&AppState>,
    ) -> Attachment<Vec<u8>> {
        let cert = "-----BEGIN CERTIFICATE-----\n...\n-----END CERTIFICATE-----";
        Attachment::new(cert.as_bytes().to_vec()).filename("external-rpc-ca.crt")
    }

    #[oai(path = "/v1/system/op-monitor-ca", method = "get")]
    pub async fn get_op_monitor_ca(
        &self,
        _state: Data<&AppState>,
    ) -> Json<CertificateDetailsDto> {
        Json(CertificateDetailsDto {
            hash: "opmon-ca-hash".to_string(),
            issuer_distinguished_name: "CN=OpMon CA".to_string(),
            subject_distinguished_name: "CN=OpMon".to_string(),
            not_before: "2020-01-01T00:00:00Z".to_string(),
            not_after: "2030-01-01T00:00:00Z".to_string(),
        })
    }

    #[oai(path = "/v1/system/op-monitor-ca", method = "post")]
    pub async fn upload_op_monitor_ca(&self, _state: Data<&AppState>) -> Json<bool> {
        Json(true)
    }

    #[oai(path = "/v1/system/op-monitor-ca", method = "delete")]
    pub async fn delete_op_monitor_ca(&self, _state: Data<&AppState>) -> Json<bool> {
        Json(true)
    }

    #[oai(path = "/v1/system/timestamping-services", method = "get")]
    pub async fn get_configured_timestamping_services(
        &self,
        _state: Data<&AppState>,
    ) -> Json<Vec<TimestampingServiceConfigDto>> {
        Json(vec![TimestampingServiceConfigDto {
            name: "TSA 1".to_string(),
            url: "http://tsa.example.org".to_string(),
        }])
    }

    #[oai(path = "/v1/system/timestamping-services", method = "post")]
    pub async fn add_configured_timestamping_service(
        &self,
        _state: Data<&AppState>,
        body: Json<TimestampingServiceConfigDto>,
    ) -> Json<TimestampingServiceConfigDto> {
        body
    }

    #[oai(path = "/v1/system/timestamping-services/delete", method = "post")]
    pub async fn delete_configured_timestamping_service(
        &self,
        _state: Data<&AppState>,
        _body: Json<TimestampingServiceConfigDto>,
    ) -> Json<bool> {
        Json(true)
    }

    #[oai(
        path = "/v1/system/timestamping-services/prioritization-strategy",
        method = "get"
    )]
    pub async fn get_timestamping_prioritization_strategy(
        &self,
        _state: Data<&AppState>,
    ) -> Json<PrioritizationStrategyDto> {
        Json(PrioritizationStrategyDto {
            strategy: "ROUND_ROBIN".to_string(),
        })
    }

    #[oai(path = "/v1/timestamping-services", method = "get")]
    pub async fn get_approved_timestamping_services(
        &self,
        _state: Data<&AppState>,
    ) -> Json<Vec<TimestampingServiceConfigDto>> {
        Json(vec![TimestampingServiceConfigDto {
            name: "Approved TSA".to_string(),
            url: "http://tsa.example.org".to_string(),
        }])
    }

    #[oai(path = "/v1/mail/mail-notification-status", method = "get")]
    pub async fn get_mail_notification_status(
        &self,
        _state: Data<&AppState>,
    ) -> Json<MailNotificationStatusDto> {
        Json(MailNotificationStatusDto {
            is_configured: true,
            mail_from: Some("no-reply@security-server.example".to_string()),
        })
    }

    #[oai(path = "/v1/mail/send-test-mail", method = "put")]
    pub async fn send_test_mail(
        &self,
        _state: Data<&AppState>,
        _body: Json<SendTestMailReq>,
    ) -> Json<bool> {
        Json(true)
    }
}
