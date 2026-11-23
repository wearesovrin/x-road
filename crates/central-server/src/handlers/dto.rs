use poem_openapi::Object;

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct InitializationStatusDto {
    pub is_initialized: bool,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct InitialRegistrationDto {
    pub central_server_address: String,
    pub software_token_pin: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct MemberClassDto {
    pub code: String,
    pub description: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct AddMemberClassRequest {
    pub code: String,
    pub description: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct UpdateMemberClassRequest {
    pub description: String,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct CentralServerInfo {
    pub instance_identifier: String,
    pub status: String,
    pub member_count: usize,
    pub security_server_count: usize,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct MemberDto {
    pub id: i64,
    pub instance: String,
    pub member_class: String,
    pub member_code: String,
    pub name: String,
    pub administrative_contact: Option<String>,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct AddMemberRequest {
    pub member_class: String,
    pub member_code: String,
    pub name: String,
    pub administrative_contact: Option<String>,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct UpdateMemberRequest {
    pub name: String,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct SubsystemDto {
    pub id: i64,
    pub member_id: i64,
    pub subsystem_code: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct AddSubsystemRequest {
    pub member_id: Option<i64>,
    pub subsystem_code: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct UpdateSubsystemRequest {
    pub subsystem_code: String,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct ClientDto {
    pub client_id: String,
    pub member_name: String,
    pub member_class: String,
    pub member_code: String,
    pub subsystem_code: Option<String>,
    pub status: String,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct SecurityServerDto {
    pub id: i64,
    pub owner_id: i64,
    pub server_code: String,
    pub address: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct RegisterSecurityServerRequest {
    pub owner_id: i64,
    pub server_code: String,
    pub address: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct UpdateSecurityServerRequest {
    pub address: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct LinkClientRequest {
    pub security_server_id: i64,
    pub client_id_str: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct AddServerClientRequest {
    pub client_id_str: String,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct SecurityServerAuthCertDto {
    pub id: i64,
    pub security_server_id: i64,
    pub cert_hash: String,
    pub created_at: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct AddAuthCertRequest {
    pub cert_pem: String,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct ApprovedCaDto {
    pub id: i64,
    pub name: String,
    pub authentication_only: bool,
    pub cert_profile_info: Option<String>,
    pub not_before: Option<String>,
    pub not_after: Option<String>,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct AddCaRequest {
    pub name: String,
    pub top_ca_cert_pem: String,
    pub authentication_only: bool,
    pub cert_profile_info: Option<String>,
    pub acme_server_directory_url: Option<String>,
    pub acme_server_ip_address: Option<String>,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct UpdateCaRequest {
    pub cert_profile_info: Option<String>,
    pub acme_server_directory_url: Option<String>,
    pub acme_server_ip_address: Option<String>,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct IntermediateCaDto {
    pub id: i64,
    pub ca_id: i64,
    pub cert_hash: String,
    pub created_at: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct AddIntermediateCaRequest {
    pub cert_pem: String,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct OcspResponderDto {
    pub id: i64,
    pub url: String,
    pub has_certificate: bool,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct AddOcspResponderRequest {
    pub url: String,
    pub cert_pem: Option<String>,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct UpdateOcspResponderRequest {
    pub url: String,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct CertificateDetailsDto {
    pub hash: String,
    pub issuer_distinguished_name: String,
    pub subject_distinguished_name: String,
    pub not_before: String,
    pub not_after: String,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct ApprovedTsaDto {
    pub id: i64,
    pub name: String,
    pub url: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct AddTsaRequest {
    pub name: String,
    pub url: String,
    pub cert_pem: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct UpdateTsaRequest {
    pub url: String,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct GlobalGroupDto {
    pub group_code: String,
    pub description: String,
    pub member_count: usize,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct CreateGlobalGroupRequest {
    pub group_code: String,
    pub description: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct UpdateGlobalGroupRequest {
    pub description: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct AddGroupMemberRequest {
    pub client_id_str: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct AddGroupMembersRequest {
    pub items: Vec<String>,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct GroupMembersFilterModelDto {
    pub group_code: String,
    pub members: Vec<String>,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct ConfigurationAnchorDto {
    pub instance_identifier: String,
    pub generated_at: String,
    pub hash: String,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct ConfigurationPartDto {
    pub content_identifier: String,
    pub file_name: String,
    pub version: i32,
    pub updated_at: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct UploadConfigurationPartRequest {
    pub content_identifier: String,
    pub content_base64: String,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct SigningKeyDto {
    pub id: String,
    pub key_hash: String,
    pub label: String,
    pub active: bool,
    pub available: bool,
    pub created_at: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct AddSigningKeyRequest {
    pub label: Option<String>,
    pub token_id: Option<String>,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct TokenDto {
    pub id: String,
    pub name: String,
    pub serial_number: Option<String>,
    pub status: String,
    pub logged_in: bool,
    pub active: bool,
    pub available: bool,
    pub possible_actions: Vec<String>,
    pub keys: Vec<SigningKeyDto>,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct TokenPasswordRequest {
    pub password: String,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct TrustedAnchorDto {
    pub hash: String,
    pub instance_identifier: String,
    pub generated_at: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct UploadTrustedAnchorRequest {
    pub anchor_xml: String,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct ManagementServicesConfigurationDto {
    pub service_provider_id: Option<String>,
    pub service_provider_name: Option<String>,
    pub security_server_id: Option<String>,
    pub security_server_address: Option<String>,
    pub services_address: Option<String>,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct UpdateManagementServicesConfigurationRequest {
    pub security_server_id: Option<String>,
    pub services_address: Option<String>,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct RegisterServiceProviderRequest {
    pub service_provider_id: String,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct IssuerServiceStateDto {
    pub host: Option<String>,
    pub bound: bool,
    pub host_set: bool,
    pub certificate_ready: bool,
    pub run_in_progress: bool,
    pub did_to_bind: Option<String>,
    pub bound_did: Option<String>,
    pub can_sign: bool,
    pub status: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct IssuerServiceHostRequest {
    pub host: String,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct HighAvailabilityClusterStatusDto {
    pub status: String,
    pub nodes: Vec<String>,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct DidCheckResultDto {
    pub valid: bool,
    pub message: Option<String>,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct DidCheckRequestDto {
    pub did: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct UpdateMemberDidRequest {
    pub did: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ServerAddressDto {
    pub server_address: String,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct SystemVersionDto {
    pub version: String,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct ManagementRequestDto {
    pub id: i64,
    pub request_type: String,
    pub origin: String,
    pub security_server_id: Option<i64>,
    pub client_id_str: Option<String>,
    pub status: String,
    pub comments: Option<String>,
    pub created_at: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct CreateManagementRequestDto {
    pub request_type: String,
    pub origin: String,
    pub security_server_id: Option<i64>,
    pub client_id_str: Option<String>,
    pub auth_cert_pem: Option<String>,
    pub comments: Option<String>,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct BackupDto {
    pub filename: String,
    pub created_at: String,
    pub size_bytes: u64,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct UploadBackupRequest {
    pub filename: String,
    pub content_base64: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct CreateApiKeyReq {
    pub roles: Vec<String>,
    pub label: Option<String>,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct UpdateApiKeyReq {
    pub roles: Vec<String>,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct ApiKeyDto {
    pub id: i64,
    pub key_id: String,
    pub key: Option<String>,
    pub label: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct ValidateApiKeyReq {
    pub key: String,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct ValidateApiKeyResp {
    pub valid: bool,
    pub roles: Vec<String>,
}
