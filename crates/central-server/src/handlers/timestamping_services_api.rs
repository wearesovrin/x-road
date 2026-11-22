use crate::handlers::dto::{AddTsaRequest, ApprovedTsaDto};
use crate::services::CentralServerDb;
use poem::http::StatusCode;
use poem_openapi::{OpenApi, param::Path, payload::Json};

pub struct TimestampingServicesApi {
    pub db: CentralServerDb,
}

#[OpenApi]
impl TimestampingServicesApi {
    #[oai(path = "/v1/timestamping-services", method = "get")]
    pub async fn list_timestamping_services(&self) -> Json<Vec<ApprovedTsaDto>> {
        self.list_tsas().await
    }

    #[oai(path = "/v1/timestamping-services", method = "post")]
    pub async fn add_timestamping_service(
        &self,
        body: Json<AddTsaRequest>,
    ) -> poem::Result<Json<ApprovedTsaDto>> {
        self.add_tsa(body).await
    }

    #[oai(path = "/v1/timestamping-services/:id", method = "get")]
    pub async fn get_timestamping_service(
        &self,
        id: Path<i64>,
    ) -> poem::Result<Json<ApprovedTsaDto>> {
        let tsa = self
            .db
            .get_approved_tsa(id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .ok_or_else(|| poem::Error::from_string("TSA not found", StatusCode::NOT_FOUND))?;
        Ok(Json(ApprovedTsaDto {
            id: tsa.id,
            name: tsa.name,
            url: tsa.url,
        }))
    }

    #[oai(path = "/v1/timestamping-services/:id", method = "delete")]
    pub async fn delete_timestamping_service(&self, id: Path<i64>) -> poem::Result<Json<bool>> {
        let ok = self.db.delete_approved_tsa(id.0).await.map_err(|e| {
            poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
        })?;
        Ok(Json(ok))
    }

    #[oai(path = "/v1/tsas", method = "get")]
    pub async fn list_tsas(&self) -> Json<Vec<ApprovedTsaDto>> {
        let tsas = self.db.list_approved_tsas().await.unwrap_or_default();
        let dtos = tsas
            .into_iter()
            .map(|t| ApprovedTsaDto {
                id: t.id,
                name: t.name,
                url: t.url,
            })
            .collect();
        Json(dtos)
    }

    #[oai(path = "/v1/tsas", method = "post")]
    pub async fn add_tsa(&self, body: Json<AddTsaRequest>) -> poem::Result<Json<ApprovedTsaDto>> {
        let req = body.0;
        let tsa = self
            .db
            .add_approved_tsa(&req.name, &req.url, req.cert_pem.as_bytes())
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(ApprovedTsaDto {
            id: tsa.id,
            name: tsa.name,
            url: tsa.url,
        }))
    }
}
