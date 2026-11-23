use crate::handlers::dto::{AddTsaRequest, ApprovedTsaDto, UpdateTsaRequest};
use crate::handlers::{ApiTags, AppState};
use poem::{http::StatusCode, web::Data};
use poem_openapi::{param::Path, payload::Json, OpenApi};

pub struct TimestampingServicesApi;

#[OpenApi(tag = "ApiTags::TimestampingServices")]
impl TimestampingServicesApi {
    #[oai(path = "/v1/timestamping-services", method = "get")]
    pub async fn list_timestamping_services(
        &self,
        state: Data<&AppState>,
    ) -> Json<Vec<ApprovedTsaDto>> {
        self.list_tsas(state).await
    }

    #[oai(path = "/v1/timestamping-services", method = "post")]
    pub async fn add_timestamping_service(
        &self,
        state: Data<&AppState>,
        body: Json<AddTsaRequest>,
    ) -> poem::Result<Json<ApprovedTsaDto>> {
        self.add_tsa(state, body).await
    }

    #[oai(
        path = "/v1/timestamping-services/:timestamping_service_id",
        method = "get"
    )]
    pub async fn get_timestamping_service(
        &self,
        state: Data<&AppState>,
        timestamping_service_id: Path<i64>,
    ) -> poem::Result<Json<ApprovedTsaDto>> {
        let tsa = state
            .db
            .get_approved_tsa(timestamping_service_id.0)
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

    #[oai(
        path = "/v1/timestamping-services/:timestamping_service_id",
        method = "patch"
    )]
    pub async fn update_timestamping_service(
        &self,
        state: Data<&AppState>,
        timestamping_service_id: Path<i64>,
        body: Json<UpdateTsaRequest>,
    ) -> poem::Result<Json<ApprovedTsaDto>> {
        let tsa = state
            .db
            .get_approved_tsa(timestamping_service_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .ok_or_else(|| poem::Error::from_string("TSA not found", StatusCode::NOT_FOUND))?;
        Ok(Json(ApprovedTsaDto {
            id: tsa.id,
            name: tsa.name,
            url: body.0.url,
        }))
    }

    #[oai(
        path = "/v1/timestamping-services/:timestamping_service_id",
        method = "delete"
    )]
    pub async fn delete_timestamping_service(
        &self,
        state: Data<&AppState>,
        timestamping_service_id: Path<i64>,
    ) -> poem::Result<Json<bool>> {
        let ok = state
            .db
            .delete_approved_tsa(timestamping_service_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        Ok(Json(ok))
    }

    #[oai(path = "/v1/tsas", method = "get")]
    pub async fn list_tsas(&self, state: Data<&AppState>) -> Json<Vec<ApprovedTsaDto>> {
        let tsas = state.db.list_approved_tsas().await.unwrap_or_default();
        let dtos = tsas
            .into_iter()
            .map(|tsa| ApprovedTsaDto {
                id: tsa.id,
                name: tsa.name,
                url: tsa.url,
            })
            .collect();
        Json(dtos)
    }

    #[oai(path = "/v1/tsas", method = "post")]
    pub async fn add_tsa(
        &self,
        state: Data<&AppState>,
        body: Json<AddTsaRequest>,
    ) -> poem::Result<Json<ApprovedTsaDto>> {
        let req = body.0;
        let tsa = state
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
