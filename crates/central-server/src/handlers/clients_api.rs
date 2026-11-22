use crate::handlers::dto::ClientDto;
use crate::services::CentralServerDb;
use poem::http::StatusCode;
use poem_openapi::{OpenApi, payload::Json};

pub struct ClientsApi {
    pub db: CentralServerDb,
}

#[OpenApi]
impl ClientsApi {
    #[oai(path = "/v1/clients", method = "get")]
    pub async fn list_all_clients(&self) -> poem::Result<Json<Vec<ClientDto>>> {
        let clients = self.db.list_all_clients().await.map_err(|e| {
            poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
        })?;
        Ok(Json(
            clients
                .into_iter()
                .map(|c| ClientDto {
                    client_id: c.client_id,
                    member_name: c.member_name,
                    member_class: c.member_class,
                    member_code: c.member_code,
                    subsystem_code: c.subsystem_code,
                    status: c.status,
                })
                .collect(),
        ))
    }
}
