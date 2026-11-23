use crate::handlers::dto::{
    AddCaRequest, AddIntermediateCaRequest, ApprovedCaDto, IntermediateCaDto, UpdateCaRequest,
};
use crate::handlers::{ApiTags, AppState};
use poem::web::Data;
use poem_openapi::{
    param::Path,
    payload::{Json, PlainText},
    OpenApi,
};

pub struct DsTlsApi;

#[OpenApi(tag = "ApiTags::DsTlsCertificationAuthorities")]
impl DsTlsApi {
    #[oai(path = "/v1/ds-tls-certification-authorities", method = "get")]
    pub async fn get_ds_tls_certification_authorities(
        &self,
        _state: Data<&AppState>,
    ) -> Json<Vec<ApprovedCaDto>> {
        Json(vec![])
    }

    #[oai(path = "/v1/ds-tls-certification-authorities", method = "post")]
    pub async fn add_ds_tls_certification_authority(
        &self,
        _state: Data<&AppState>,
        body: Json<AddCaRequest>,
    ) -> Json<ApprovedCaDto> {
        let req = body.0;
        Json(ApprovedCaDto {
            id: 1,
            name: req.name,
            authentication_only: true,
            cert_profile_info: req.cert_profile_info,
            not_before: Some("2020-01-01T00:00:00Z".to_string()),
            not_after: Some("2035-01-01T00:00:00Z".to_string()),
        })
    }

    #[oai(
        path = "/v1/ds-tls-certification-authorities/:ds_tls_certification_authority_id",
        method = "get"
    )]
    pub async fn get_ds_tls_certification_authority(
        &self,
        _state: Data<&AppState>,
        ds_tls_certification_authority_id: Path<i64>,
    ) -> Json<ApprovedCaDto> {
        Json(ApprovedCaDto {
            id: ds_tls_certification_authority_id.0,
            name: "DS TLS CA".to_string(),
            authentication_only: true,
            cert_profile_info: None,
            not_before: Some("2020-01-01T00:00:00Z".to_string()),
            not_after: Some("2035-01-01T00:00:00Z".to_string()),
        })
    }

    #[oai(
        path = "/v1/ds-tls-certification-authorities/:ds_tls_certification_authority_id",
        method = "patch"
    )]
    pub async fn update_ds_tls_certification_authority(
        &self,
        _state: Data<&AppState>,
        ds_tls_certification_authority_id: Path<i64>,
        body: Json<UpdateCaRequest>,
    ) -> Json<ApprovedCaDto> {
        Json(ApprovedCaDto {
            id: ds_tls_certification_authority_id.0,
            name: "DS TLS CA".to_string(),
            authentication_only: true,
            cert_profile_info: body.0.cert_profile_info,
            not_before: Some("2020-01-01T00:00:00Z".to_string()),
            not_after: Some("2035-01-01T00:00:00Z".to_string()),
        })
    }

    #[oai(
        path = "/v1/ds-tls-certification-authorities/:ds_tls_certification_authority_id",
        method = "delete"
    )]
    pub async fn delete_ds_tls_certification_authority(
        &self,
        _state: Data<&AppState>,
        _ds_tls_certification_authority_id: Path<i64>,
    ) -> Json<bool> {
        Json(true)
    }

    #[oai(
        path = "/v1/ds-tls-certification-authorities/:ds_tls_certification_authority_id/certificate",
        method = "get"
    )]
    pub async fn get_ds_tls_certification_authority_certificate(
        &self,
        _state: Data<&AppState>,
        _ds_tls_certification_authority_id: Path<i64>,
    ) -> PlainText<String> {
        PlainText("-----BEGIN CERTIFICATE-----\n...\n-----END CERTIFICATE-----".to_string())
    }

    #[oai(
        path = "/v1/ds-tls-certification-authorities/:ds_tls_certification_authority_id/intermediate-cas",
        method = "get"
    )]
    pub async fn get_ds_tls_certification_authority_intermediate_cas(
        &self,
        _state: Data<&AppState>,
        ds_tls_certification_authority_id: Path<i64>,
    ) -> Json<Vec<IntermediateCaDto>> {
        Json(vec![IntermediateCaDto {
            id: 1,
            ca_id: ds_tls_certification_authority_id.0,
            cert_hash: "hash".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        }])
    }

    #[oai(
        path = "/v1/ds-tls-certification-authorities/:ds_tls_certification_authority_id/intermediate-cas",
        method = "post"
    )]
    pub async fn add_ds_tls_certification_authority_intermediate_ca(
        &self,
        _state: Data<&AppState>,
        ds_tls_certification_authority_id: Path<i64>,
        _body: Json<AddIntermediateCaRequest>,
    ) -> Json<IntermediateCaDto> {
        Json(IntermediateCaDto {
            id: 1,
            ca_id: ds_tls_certification_authority_id.0,
            cert_hash: "added-hash".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    #[oai(
        path = "/v1/ds-tls-intermediate-cas/:ds_tls_intermediate_ca_id",
        method = "get"
    )]
    pub async fn get_ds_tls_intermediate_ca(
        &self,
        _state: Data<&AppState>,
        ds_tls_intermediate_ca_id: Path<i64>,
    ) -> Json<IntermediateCaDto> {
        Json(IntermediateCaDto {
            id: ds_tls_intermediate_ca_id.0,
            ca_id: 1,
            cert_hash: "hash".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    #[oai(
        path = "/v1/ds-tls-intermediate-cas/:ds_tls_intermediate_ca_id",
        method = "delete"
    )]
    pub async fn delete_ds_tls_intermediate_ca(
        &self,
        _state: Data<&AppState>,
        _ds_tls_intermediate_ca_id: Path<i64>,
    ) -> Json<bool> {
        Json(true)
    }
}
