use crate::handlers::dto::{AddCaRequest, ApprovedCaDto};
use crate::services::CentralServerDb;
use poem::http::StatusCode;
use poem_openapi::{
    OpenApi,
    param::Path,
    payload::{Json, PlainText},
};

pub struct CertificationServicesApi {
    pub db: CentralServerDb,
}

#[OpenApi]
impl CertificationServicesApi {
    #[oai(path = "/v1/certification-services", method = "get")]
    pub async fn list_certification_services(&self) -> Json<Vec<ApprovedCaDto>> {
        self.list_cas().await
    }

    #[oai(path = "/v1/certification-services", method = "post")]
    pub async fn add_certification_service(
        &self,
        body: Json<AddCaRequest>,
    ) -> poem::Result<Json<ApprovedCaDto>> {
        self.add_ca(body).await
    }

    #[oai(path = "/v1/certification-services/:id", method = "get")]
    pub async fn get_certification_service(
        &self,
        id: Path<i64>,
    ) -> poem::Result<Json<ApprovedCaDto>> {
        let ca = self
            .db
            .get_approved_ca(id.0)
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
        }))
    }

    #[oai(path = "/v1/certification-services/:id", method = "delete")]
    pub async fn delete_certification_service(&self, id: Path<i64>) -> poem::Result<Json<bool>> {
        let ok = self.db.delete_approved_ca(id.0).await.map_err(|e| {
            poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
        })?;
        Ok(Json(ok))
    }

    #[oai(path = "/v1/certification-services/:id/certificate", method = "get")]
    pub async fn download_ca_certificate(&self, id: Path<i64>) -> poem::Result<PlainText<String>> {
        let ca = self
            .db
            .get_approved_ca(id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .ok_or_else(|| poem::Error::from_string("CA not found", StatusCode::NOT_FOUND))?;
        Ok(PlainText(
            String::from_utf8_lossy(&ca.top_ca_cert).to_string(),
        ))
    }

    #[oai(path = "/v1/cas", method = "get")]
    pub async fn list_cas(&self) -> Json<Vec<ApprovedCaDto>> {
        let cas = self.db.list_approved_cas().await.unwrap_or_default();
        let dtos = cas
            .into_iter()
            .map(|ca| ApprovedCaDto {
                id: ca.id,
                name: ca.name,
                authentication_only: ca.authentication_only,
                cert_profile_info: ca.cert_profile_info,
            })
            .collect();
        Json(dtos)
    }

    #[oai(path = "/v1/cas", method = "post")]
    pub async fn add_ca(&self, body: Json<AddCaRequest>) -> poem::Result<Json<ApprovedCaDto>> {
        let req = body.0;
        let ca = self
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
        }))
    }
}
