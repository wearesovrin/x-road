use poem_openapi::Object;

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ServerStatus {
    pub status: String,
    pub version: String,
    pub client_count: usize,
    pub unarchived_logs: usize,
    pub is_initialized: bool,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct InitializationStatus {
    pub is_initialized: bool,
    pub has_software_token_pin: bool,
    pub owner_client_id: Option<String>,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct InitializeServerReq {
    pub instance: String,
    pub member_class: String,
    pub member_code: String,
    pub subsystem_code: Option<String>,
    pub software_token_pin: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ClientDto {
    pub id: String,
    pub instance: String,
    pub member_class: String,
    pub member_code: String,
    pub subsystem_code: Option<String>,
    pub status: String,
    pub is_owner: bool,
    pub is_auth: bool,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct RegisterClientReq {
    pub instance: String,
    pub member_class: String,
    pub member_code: String,
    pub subsystem_code: Option<String>,
    pub is_authentication: Option<bool>,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ServiceDescriptionDto {
    pub id: i64,
    pub client_id: String,
    pub url: String,
    pub service_type: String,
    pub disabled: bool,
    pub disabled_notice: Option<String>,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AddServiceDescriptionReq {
    pub url: String,
    pub service_type: Option<String>,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ServiceDto {
    pub id: i64,
    pub service_code: String,
    pub service_version: Option<String>,
    pub title: Option<String>,
    pub url: String,
    pub timeout: i32,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct UpdateServiceReq {
    pub url: String,
    pub timeout: i32,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct RegisterServiceReq {
    pub client_id: String,
    pub service_code: String,
    pub service_version: Option<String>,
    pub url: String,
    pub service_type: Option<String>,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AccessRightReq {
    pub service_id: String,
    pub client_id: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ServiceClientDto {
    pub subject_id: String,
    pub service_id: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct EndpointDto {
    pub id: i64,
    pub service_code: String,
    pub method: String,
    pub path: String,
    pub generated: bool,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AddEndpointReq {
    pub method: String,
    pub path: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct LocalGroupDto {
    pub id: i64,
    pub group_code: String,
    pub description: String,
    pub updated_at: String,
    pub member_count: usize,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AddLocalGroupReq {
    pub group_code: String,
    pub description: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AddGroupMemberReq {
    pub member_id: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TokenDto {
    pub id: String,
    pub name: String,
    pub token_type: String,
    pub logged_in: bool,
    pub has_pin: bool,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TokenLoginReq {
    pub pin: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TokenPinReq {
    pub old_pin: Option<String>,
    pub new_pin: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct GenerateKeyReq {
    pub label: String,
    pub usage: Option<String>,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct KeyDto {
    pub id: String,
    pub label: Option<String>,
    pub friendly_name: Option<String>,
    pub usage: i32,
    pub available: bool,
    pub public_key: String,
    pub sign_mechanism: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct GenerateCsrReq {
    pub subject_cn: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct CsrDto {
    pub id: String,
    pub csr_pem: String,
    pub subject_name: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ImportCertReq {
    pub key_id: String,
    pub cert_der_base64: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct CertDto {
    pub id: String,
    pub active: bool,
    pub status: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ApprovedCaDto {
    pub name: String,
    pub authentication_only: bool,
    pub cert_profile_info: Option<String>,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AcmeOrderReq {
    pub domain: String,
    pub key_purpose: Option<String>,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AcmeOrderResponse {
    pub order_id: String,
    pub status: String,
    pub domain: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SystemVersion {
    pub version: String,
    pub edition: String,
    pub authors: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ServerAddressReq {
    pub server_address: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ServerAddressResponse {
    pub server_address: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct MaintenanceModeResponse {
    pub maintenance_mode: bool,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct BackupDto {
    pub filename: String,
    pub created_at: String,
    pub size_bytes: usize,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct DiagnosticsGlobalConf {
    pub instance_identifier: String,
    pub status: String,
    pub members_count: usize,
    pub security_servers_count: usize,
    pub approved_cas_count: usize,
    pub approved_tsps_count: usize,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct DiagnosticsTsa {
    pub name: String,
    pub url: String,
    pub status: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct DiagnosticsOcsp {
    pub ca_name: String,
    pub url: String,
    pub status: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct CreateApiKeyReq {
    pub key: Option<String>,
    pub label: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct UpdateApiKeyReq {
    pub roles: Vec<String>,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ApiKeyDto {
    pub id: i64,
    pub key_id: String,
    pub key: Option<String>,
    pub label: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ApiKeyResponse {
    pub id: i64,
    pub key_id: String,
    pub key: Option<String>,
    pub label: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ValidateApiKeyReq {
    pub key: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ValidateApiKeyResponse {
    pub valid: bool,
    pub roles: Vec<String>,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct StoreCertReq {
    pub client_id: String,
    pub cert_der_base64: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SimpleSuccessResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct LogRecordResponse {
    pub id: i64,
    pub query_id: String,
    pub client_id: Option<String>,
    pub service_id: Option<String>,
    pub archived: bool,
    pub timestamp: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ArchiveResponse {
    pub success: bool,
    pub records_archived: usize,
    pub file_path: Option<String>,
    pub file_name: Option<String>,
    pub last_digest: Option<String>,
    pub message: String,
}
