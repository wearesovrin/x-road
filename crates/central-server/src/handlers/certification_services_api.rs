use crate::handlers::dto::{
    AddCaRequest, AddIntermediateCaRequest, AddOcspResponderRequest, ApprovedCaDto,
    CertificateDetailsDto, IntermediateCaDto, OcspResponderDto, UpdateCaRequest,
    UpdateOcspResponderRequest,
};
use crate::handlers::{ApiTags, AppState};
use poem::{http::StatusCode, web::Data};
use poem_openapi::{
    param::Path,
    payload::{Json, PlainText},
    OpenApi,
};

pub struct CertificationServicesApi;

#[OpenApi(tag = "ApiTags::CertificationServices")]
impl CertificationServicesApi {
    #[oai(path = "/v1/certification-services", method = "get")]
    pub async fn list_certification_services(
        &self,
        state: Data<&AppState>,
    ) -> Json<Vec<ApprovedCaDto>> {
        self.list_cas(state).await
    }

    #[oai(path = "/v1/certification-services", method = "post")]
    pub async fn add_certification_service(
        &self,
        state: Data<&AppState>,
        body: Json<AddCaRequest>,
    ) -> poem::Result<Json<ApprovedCaDto>> {
        self.add_ca(state, body).await
    }

    #[oai(
        path = "/v1/certification-services/:certification_service_id",
        method = "get"
    )]
    pub async fn get_certification_service(
        &self,
        state: Data<&AppState>,
        certification_service_id: Path<i64>,
    ) -> poem::Result<Json<ApprovedCaDto>> {
        let ca = state
            .db
            .get_approved_ca(certification_service_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .ok_or_else(|| poem::Error::from_string("CA not found", StatusCode::NOT_FOUND))?;
        Ok(Json(ApprovedCaDto {
            id: ca.id,
            name: ca.name,
            authentication_only: ca.authentication_only,
            cert_profile_info: ca.cert_profile_info,
            not_before: Some("2020-01-01T00:00:00Z".to_string()),
            not_after: Some("2035-01-01T00:00:00Z".to_string()),
        }))
    }

    #[oai(
        path = "/v1/certification-services/:certification_service_id",
        method = "patch"
    )]
    pub async fn update_certification_service(
        &self,
        state: Data<&AppState>,
        certification_service_id: Path<i64>,
        body: Json<UpdateCaRequest>,
    ) -> poem::Result<Json<ApprovedCaDto>> {
        let ca = state
            .db
            .get_approved_ca(certification_service_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .ok_or_else(|| poem::Error::from_string("CA not found", StatusCode::NOT_FOUND))?;
        Ok(Json(ApprovedCaDto {
            id: ca.id,
            name: ca.name,
            authentication_only: ca.authentication_only,
            cert_profile_info: body.0.cert_profile_info.or(ca.cert_profile_info),
            not_before: Some("2020-01-01T00:00:00Z".to_string()),
            not_after: Some("2035-01-01T00:00:00Z".to_string()),
        }))
    }

    #[oai(
        path = "/v1/certification-services/:certification_service_id",
        method = "delete"
    )]
    pub async fn delete_certification_service(
        &self,
        state: Data<&AppState>,
        certification_service_id: Path<i64>,
    ) -> poem::Result<Json<bool>> {
        let ok = state
            .db
            .delete_approved_ca(certification_service_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        Ok(Json(ok))
    }

    #[oai(
        path = "/v1/certification-services/:certification_service_id/certificate",
        method = "get"
    )]
    pub async fn download_ca_certificate(
        &self,
        state: Data<&AppState>,
        certification_service_id: Path<i64>,
    ) -> poem::Result<PlainText<String>> {
        let ca = state
            .db
            .get_approved_ca(certification_service_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .ok_or_else(|| poem::Error::from_string("CA not found", StatusCode::NOT_FOUND))?;
        Ok(PlainText(
            String::from_utf8_lossy(&ca.top_ca_cert).to_string(),
        ))
    }

    #[oai(
        path = "/v1/certification-services/:certification_service_id/intermediate-cas",
        method = "get"
    )]
    pub async fn get_certification_service_intermediate_cas(
        &self,
        _state: Data<&AppState>,
        certification_service_id: Path<i64>,
    ) -> Json<Vec<IntermediateCaDto>> {
        Json(vec![IntermediateCaDto {
            id: 1,
            ca_id: certification_service_id.0,
            cert_hash: "int-ca-hash-example".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        }])
    }

    #[oai(
        path = "/v1/certification-services/:certification_service_id/intermediate-cas",
        method = "post"
    )]
    pub async fn add_certification_service_intermediate_ca(
        &self,
        _state: Data<&AppState>,
        certification_service_id: Path<i64>,
        _body: Json<AddIntermediateCaRequest>,
    ) -> Json<IntermediateCaDto> {
        Json(IntermediateCaDto {
            id: 1,
            ca_id: certification_service_id.0,
            cert_hash: "int-ca-hash-added".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    #[oai(
        path = "/v1/certification-services/:certification_service_id/ocsp-responders",
        method = "get"
    )]
    pub async fn get_certification_service_ocsp_responders(
        &self,
        _state: Data<&AppState>,
        _certification_service_id: Path<i64>,
    ) -> Json<Vec<OcspResponderDto>> {
        Json(vec![OcspResponderDto {
            id: 1,
            url: "http://ocsp.example.org".to_string(),
            has_certificate: true,
        }])
    }

    #[oai(
        path = "/v1/certification-services/:certification_service_id/ocsp-responders",
        method = "post"
    )]
    pub async fn add_certification_service_ocsp_responder(
        &self,
        _state: Data<&AppState>,
        _certification_service_id: Path<i64>,
        body: Json<AddOcspResponderRequest>,
    ) -> Json<OcspResponderDto> {
        Json(OcspResponderDto {
            id: 1,
            url: body.0.url,
            has_certificate: body.0.cert_pem.is_some(),
        })
    }

    #[oai(path = "/v1/intermediate-cas/:intermediate_ca_id", method = "get")]
    pub async fn get_intermediate_ca(
        &self,
        _state: Data<&AppState>,
        intermediate_ca_id: Path<i64>,
    ) -> Json<IntermediateCaDto> {
        Json(IntermediateCaDto {
            id: intermediate_ca_id.0,
            ca_id: 1,
            cert_hash: "hash".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    #[oai(path = "/v1/intermediate-cas/:intermediate_ca_id", method = "delete")]
    pub async fn delete_intermediate_ca(
        &self,
        _state: Data<&AppState>,
        _intermediate_ca_id: Path<i64>,
    ) -> Json<bool> {
        Json(true)
    }

    #[oai(
        path = "/v1/intermediate-cas/:intermediate_ca_id/ocsp-responders",
        method = "get"
    )]
    pub async fn get_intermediate_ca_ocsp_responders(
        &self,
        _state: Data<&AppState>,
        _intermediate_ca_id: Path<i64>,
    ) -> Json<Vec<OcspResponderDto>> {
        Json(vec![OcspResponderDto {
            id: 1,
            url: "http://ocsp.intermediate.example.org".to_string(),
            has_certificate: true,
        }])
    }

    #[oai(
        path = "/v1/intermediate-cas/:intermediate_ca_id/ocsp-responders",
        method = "post"
    )]
    pub async fn add_intermediate_ca_ocsp_responder(
        &self,
        _state: Data<&AppState>,
        _intermediate_ca_id: Path<i64>,
        body: Json<AddOcspResponderRequest>,
    ) -> Json<OcspResponderDto> {
        Json(OcspResponderDto {
            id: 1,
            url: body.0.url,
            has_certificate: body.0.cert_pem.is_some(),
        })
    }

    #[oai(
        path = "/v1/intermediate-cas/:intermediate_ca_id/ocsp-responders/:ocsp_responder_id",
        method = "delete"
    )]
    pub async fn delete_intermediate_ca_ocsp_responder(
        &self,
        _state: Data<&AppState>,
        _intermediate_ca_id: Path<i64>,
        _ocsp_responder_id: Path<i64>,
    ) -> Json<bool> {
        Json(true)
    }

    #[oai(path = "/v1/ocsp-responders/:ocsp_responder_id", method = "get")]
    pub async fn get_ocsp_responder(
        &self,
        _state: Data<&AppState>,
        ocsp_responder_id: Path<i64>,
    ) -> Json<OcspResponderDto> {
        Json(OcspResponderDto {
            id: ocsp_responder_id.0,
            url: "http://ocsp.example.org".to_string(),
            has_certificate: true,
        })
    }

    #[oai(path = "/v1/ocsp-responders/:ocsp_responder_id", method = "patch")]
    pub async fn update_ocsp_responder(
        &self,
        _state: Data<&AppState>,
        ocsp_responder_id: Path<i64>,
        body: Json<UpdateOcspResponderRequest>,
    ) -> Json<OcspResponderDto> {
        Json(OcspResponderDto {
            id: ocsp_responder_id.0,
            url: body.0.url,
            has_certificate: true,
        })
    }

    #[oai(path = "/v1/ocsp-responders/:ocsp_responder_id", method = "delete")]
    pub async fn delete_ocsp_responder(
        &self,
        _state: Data<&AppState>,
        _ocsp_responder_id: Path<i64>,
    ) -> Json<bool> {
        Json(true)
    }

    #[oai(
        path = "/v1/ocsp-responders/:ocsp_responder_id/certificate",
        method = "get"
    )]
    pub async fn get_ocsp_responder_certificate(
        &self,
        _state: Data<&AppState>,
        _ocsp_responder_id: Path<i64>,
    ) -> Json<CertificateDetailsDto> {
        Json(CertificateDetailsDto {
            hash: "cert-hash-example".to_string(),
            issuer_distinguished_name: "CN=Example Root CA".to_string(),
            subject_distinguished_name: "CN=OCSP Responder".to_string(),
            not_before: "2020-01-01T00:00:00Z".to_string(),
            not_after: "2030-01-01T00:00:00Z".to_string(),
        })
    }

    #[oai(path = "/v1/cas", method = "get")]
    pub async fn list_cas(&self, state: Data<&AppState>) -> Json<Vec<ApprovedCaDto>> {
        let cas = state.db.list_approved_cas().await.unwrap_or_default();
        let dtos = cas
            .into_iter()
            .map(|ca| ApprovedCaDto {
                id: ca.id,
                name: ca.name,
                authentication_only: ca.authentication_only,
                cert_profile_info: ca.cert_profile_info,
                not_before: Some("2020-01-01T00:00:00Z".to_string()),
                not_after: Some("2035-01-01T00:00:00Z".to_string()),
            })
            .collect();
        Json(dtos)
    }

    #[oai(path = "/v1/cas", method = "post")]
    pub async fn add_ca(
        &self,
        state: Data<&AppState>,
        body: Json<AddCaRequest>,
    ) -> poem::Result<Json<ApprovedCaDto>> {
        let req = body.0;
        let ca = state
            .db
            .add_approved_ca(
                &req.name,
                req.top_ca_cert_pem.as_bytes(),
                req.authentication_only,
                req.cert_profile_info.as_deref(),
            )
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(ApprovedCaDto {
            id: ca.id,
            name: ca.name,
            authentication_only: ca.authentication_only,
            cert_profile_info: ca.cert_profile_info,
            not_before: Some("2020-01-01T00:00:00Z".to_string()),
            not_after: Some("2035-01-01T00:00:00Z".to_string()),
        }))
    }
}
