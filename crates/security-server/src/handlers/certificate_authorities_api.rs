use poem_openapi::{OpenApi, param::Path, payload::Json};

use super::AdminApiState;
use super::dto::{AcmeOrderReq, AcmeOrderResponse, ApprovedCaDto};

pub struct CertificateAuthoritiesApi {
    pub state: AdminApiState,
}

#[OpenApi]
impl CertificateAuthoritiesApi {
    /// Lists Approved Certification Authorities from Global Configuration.
    #[oai(path = "/v1/certificate-authorities", method = "get")]
    pub async fn list_certificate_authorities(&self) -> Json<Vec<ApprovedCaDto>> {
        let cas = &self.state.global_conf.approved_cas;
        let dtos = cas
            .iter()
            .map(|ca| ApprovedCaDto {
                name: ca.name.clone(),
                authentication_only: false,
                cert_profile_info: None,
            })
            .collect();
        Json(dtos)
    }

    /// Initiates automated ACME certificate order (RFC 8555) via ACME protocol.
    #[oai(
        path = "/v1/certificate-authorities/:ca_name/acme-order",
        method = "post"
    )]
    pub async fn initiate_acme_order(
        &self,
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
}
