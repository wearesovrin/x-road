use crate::handlers::dto::{CreateManagementRequestDto, ManagementRequestDto};
use crate::services::CentralServerDb;
use poem::http::StatusCode;
use poem_openapi::{OpenApi, param::Path, payload::Json};

pub struct ManagementRequestsApi {
    pub db: CentralServerDb,
}

#[OpenApi]
impl ManagementRequestsApi {
    #[oai(path = "/v1/management-requests", method = "get")]
    pub async fn list_management_requests(&self) -> Json<Vec<ManagementRequestDto>> {
        let reqs = self.db.list_management_requests().await.unwrap_or_default();
        Json(
            reqs.into_iter()
                .map(|r| ManagementRequestDto {
                    id: r.id,
                    request_type: r.request_type,
                    origin: r.origin,
                    security_server_id: r.security_server_id,
                    client_id_str: r.client_id_str,
                    status: r.status,
                    comments: r.comments,
                    created_at: r.created_at.to_rfc3339(),
                })
                .collect(),
        )
    }

    #[oai(path = "/v1/management-requests", method = "post")]
    pub async fn create_management_request(
        &self,
        body: Json<CreateManagementRequestDto>,
    ) -> poem::Result<Json<ManagementRequestDto>> {
        let req = body.0;
        let created = self
            .db
            .create_management_request(
                &req.request_type,
                &req.origin,
                req.security_server_id,
                req.client_id_str.as_deref(),
                req.auth_cert_pem.as_deref(),
                req.comments.as_deref(),
            )
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(ManagementRequestDto {
            id: created.id,
            request_type: created.request_type,
            origin: created.origin,
            security_server_id: created.security_server_id,
            client_id_str: created.client_id_str,
            status: created.status,
            comments: created.comments,
            created_at: created.created_at.to_rfc3339(),
        }))
    }

    #[oai(path = "/v1/management-requests/:id", method = "get")]
    pub async fn get_management_request(
        &self,
        id: Path<i64>,
    ) -> poem::Result<Json<ManagementRequestDto>> {
        let r = self
            .db
            .get_management_request(id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .ok_or_else(|| {
                poem::Error::from_string("Management request not found", StatusCode::NOT_FOUND)
            })?;

        Ok(Json(ManagementRequestDto {
            id: r.id,
            request_type: r.request_type,
            origin: r.origin,
            security_server_id: r.security_server_id,
            client_id_str: r.client_id_str,
            status: r.status,
            comments: r.comments,
            created_at: r.created_at.to_rfc3339(),
        }))
    }

    #[oai(path = "/v1/management-requests/:id/approval", method = "put")]
    pub async fn approve_management_request(
        &self,
        id: Path<i64>,
    ) -> poem::Result<Json<ManagementRequestDto>> {
        let r = self
            .db
            .approve_management_request(id.0)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(ManagementRequestDto {
            id: r.id,
            request_type: r.request_type,
            origin: r.origin,
            security_server_id: r.security_server_id,
            client_id_str: r.client_id_str,
            status: r.status,
            comments: r.comments,
            created_at: r.created_at.to_rfc3339(),
        }))
    }

    #[oai(path = "/v1/management-requests/:id/decline", method = "post")]
    pub async fn decline_management_request(
        &self,
        id: Path<i64>,
    ) -> poem::Result<Json<ManagementRequestDto>> {
        let r = self
            .db
            .decline_management_request(id.0)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(ManagementRequestDto {
            id: r.id,
            request_type: r.request_type,
            origin: r.origin,
            security_server_id: r.security_server_id,
            client_id_str: r.client_id_str,
            status: r.status,
            comments: r.comments,
            created_at: r.created_at.to_rfc3339(),
        }))
    }
}
