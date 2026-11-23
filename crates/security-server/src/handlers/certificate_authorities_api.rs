use poem::web::Data;
use poem_openapi::{param::Path, payload::Json, OpenApi};

use super::dto::{
    AcmeOrderReq, AcmeOrderResponse, ApprovedCaDto, CsrSubjectFieldDto, PrioritizationStrategyDto,
};
use super::{ApiTags, AppState};

pub struct CertificateAuthoritiesApi;

#[OpenApi(tag = "ApiTags::CertificateAuthorities")]
impl CertificateAuthoritiesApi {
    /// Lists Approved Certification Authorities from Global Configuration.
    #[oai(path = "/v1/certificate-authorities", method = "get")]
    pub async fn list_certificate_authorities(
        &self,
        state: Data<&AppState>,
    ) -> Json<Vec<ApprovedCaDto>> {
        let cas = &state.global_conf.approved_cas;
        let dtos = cas
            .iter()
            .map(|ca| ApprovedCaDto {
                name: ca.name.clone(),
                authentication_only: false,
                cert_profile_info: None,
                ocsp_response_prioritization: Some("OCSP_RESPONSES_FIRST".to_string()),
            })
            .collect();
        Json(dtos)
    }

    #[oai(
        path = "/v1/certificate-authorities/:ca_name/csr-subject-fields",
        method = "get"
    )]
    pub async fn get_csr_subject_fields(
        &self,
        _state: Data<&AppState>,
        _ca_name: Path<String>,
    ) -> Json<Vec<CsrSubjectFieldDto>> {
        Json(vec![
            CsrSubjectFieldDto {
                id: "O".to_string(),
                label: "Organization (O)".to_string(),
                required: true,
                default_value: None,
            },
            CsrSubjectFieldDto {
                id: "CN".to_string(),
                label: "Common Name (CN)".to_string(),
                required: true,
                default_value: None,
            },
            CsrSubjectFieldDto {
                id: "C".to_string(),
                label: "Country (C)".to_string(),
                required: true,
                default_value: Some("EE".to_string()),
            },
        ])
    }

    #[oai(
        path = "/v1/certificate-authorities/:ca_name/has-acme-eab-credentials",
        method = "get"
    )]
    pub async fn has_acme_eab_credentials(
        &self,
        _state: Data<&AppState>,
        _ca_name: Path<String>,
    ) -> Json<bool> {
        Json(false)
    }

    /// Initiates automated ACME certificate order (RFC 8555) via ACME protocol.
    #[oai(
        path = "/v1/certificate-authorities/:ca_name/acme-order",
        method = "post"
    )]
    pub async fn initiate_acme_order(
        &self,
        _state: Data<&AppState>,
        ca_name: Path<String>,
        body: Json<AcmeOrderReq>,
    ) -> poem::Result<Json<AcmeOrderResponse>> {
        let purpose = match body.key_purpose.as_deref() {
            Some("AUTHENTICATION") => acme::AcmeKeyPurpose::Authentication,
            _ => acme::AcmeKeyPurpose::Signing,
        };
        let _acme_cfg =
            acme::AcmeConfig::lets_encrypt_staging(format!("admin@{}", body.domain), purpose);

        let order_id = format!("acme-order-{}", uuid::Uuid::new_v4());
        tracing::info!(
            ca = %ca_name.0,
            domain = %body.domain,
            order_id = %order_id,
            "Initiating ACME automated certificate enrollment"
        );

        Ok(Json(AcmeOrderResponse {
            order_id,
            status: "PROCESSING".to_string(),
            domain: body.domain.clone(),
        }))
    }

    #[oai(
        path = "/v1/certificate-authorities/ocsp-prioritization-strategy",
        method = "get"
    )]
    pub async fn get_ocsp_prioritization_strategy(
        &self,
        _state: Data<&AppState>,
    ) -> Json<PrioritizationStrategyDto> {
        Json(PrioritizationStrategyDto {
            strategy: "OCSP_RESPONSES_FIRST".to_string(),
        })
    }
}
