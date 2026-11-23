use crate::handlers::dto::{
    CertificateDetailsDto, ManagementServicesConfigurationDto, RegisterServiceProviderRequest,
    UpdateManagementServicesConfigurationRequest,
};
use crate::handlers::{ApiTags, AppState};
use poem::web::Data;
use poem_openapi::{
    payload::{Attachment, Json, PlainText},
    OpenApi,
};

pub struct ManagementServicesApi;

#[OpenApi(tag = "ApiTags::ManagementServicesConfiguration")]
impl ManagementServicesApi {
    #[oai(path = "/v1/management-services-configuration", method = "get")]
    pub async fn get_management_services_configuration(
        &self,
        _state: Data<&AppState>,
    ) -> Json<ManagementServicesConfigurationDto> {
        Json(ManagementServicesConfigurationDto {
            service_provider_id: Some("CS/GOV/1234567-8/MANAGEMENT".to_string()),
            service_provider_name: Some("Management Service Provider".to_string()),
            security_server_id: Some("CS/GOV/1234567-8/SS1".to_string()),
            security_server_address: Some("https://management.central-server.example".to_string()),
            services_address: Some("https://management.central-server.example".to_string()),
        })
    }

    #[oai(path = "/v1/management-services-configuration", method = "patch")]
    pub async fn update_management_services_configuration(
        &self,
        _state: Data<&AppState>,
        body: Json<UpdateManagementServicesConfigurationRequest>,
    ) -> Json<ManagementServicesConfigurationDto> {
        Json(ManagementServicesConfigurationDto {
            service_provider_id: Some("CS/GOV/1234567-8/MANAGEMENT".to_string()),
            service_provider_name: Some("Management Service Provider".to_string()),
            security_server_id: body.0.security_server_id,
            security_server_address: body.0.services_address.clone(),
            services_address: body.0.services_address,
        })
    }

    #[oai(
        path = "/v1/management-services-configuration/register-provider",
        method = "post"
    )]
    pub async fn register_service_provider(
        &self,
        _state: Data<&AppState>,
        body: Json<RegisterServiceProviderRequest>,
    ) -> Json<ManagementServicesConfigurationDto> {
        Json(ManagementServicesConfigurationDto {
            service_provider_id: Some(body.0.service_provider_id),
            service_provider_name: Some("Registered Service Provider".to_string()),
            security_server_id: None,
            security_server_address: None,
            services_address: None,
        })
    }

    #[oai(
        path = "/v1/management-services-configuration/download-certificate",
        method = "get"
    )]
    pub async fn download_certificate(&self, _state: Data<&AppState>) -> Attachment<Vec<u8>> {
        let cert = "-----BEGIN CERTIFICATE-----\n...\n-----END CERTIFICATE-----";
        Attachment::new(cert.as_bytes().to_vec()).filename("management-service-cert.pem")
    }

    #[oai(
        path = "/v1/management-services-configuration/certificate",
        method = "get"
    )]
    pub async fn get_certificate(
        &self,
        _state: Data<&AppState>,
    ) -> Json<CertificateDetailsDto> {
        Json(CertificateDetailsDto {
            hash: "mgmt-cert-hash".to_string(),
            issuer_distinguished_name: "CN=Management CA".to_string(),
            subject_distinguished_name: "CN=Central Server Management".to_string(),
            not_before: "2020-01-01T00:00:00Z".to_string(),
            not_after: "2030-01-01T00:00:00Z".to_string(),
        })
    }

    #[oai(
        path = "/v1/management-services-configuration/certificate",
        method = "post"
    )]
    pub async fn generate_key_and_certificate(
        &self,
        _state: Data<&AppState>,
    ) -> Json<CertificateDetailsDto> {
        Json(CertificateDetailsDto {
            hash: "new-mgmt-cert-hash".to_string(),
            issuer_distinguished_name: "CN=Management CA".to_string(),
            subject_distinguished_name: "CN=Central Server Management".to_string(),
            not_before: "2020-01-01T00:00:00Z".to_string(),
            not_after: "2030-01-01T00:00:00Z".to_string(),
        })
    }

    #[oai(
        path = "/v1/management-services-configuration/generate-csr",
        method = "post"
    )]
    pub async fn generate_certificate_request(&self, _state: Data<&AppState>) -> PlainText<String> {
        PlainText("-----BEGIN CERTIFICATE REQUEST-----\n...\n-----END CERTIFICATE REQUEST-----".to_string())
    }

    #[oai(
        path = "/v1/management-services-configuration/upload-certificate",
        method = "post"
    )]
    pub async fn upload_certificate(
        &self,
        _state: Data<&AppState>,
    ) -> Json<CertificateDetailsDto> {
        Json(CertificateDetailsDto {
            hash: "uploaded-mgmt-cert-hash".to_string(),
            issuer_distinguished_name: "CN=Management CA".to_string(),
            subject_distinguished_name: "CN=Central Server Management".to_string(),
            not_before: "2020-01-01T00:00:00Z".to_string(),
            not_after: "2030-01-01T00:00:00Z".to_string(),
        })
    }
}
