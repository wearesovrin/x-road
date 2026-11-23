use crate::handlers::{ApiTags, AppState};
use crate::handlers::dto::SubsystemDto;
use poem::{http::StatusCode, web::Data};
use poem_openapi::{OpenApi, param::Path, payload::Json};

pub struct SubsystemsApi;

#[OpenApi(tag = "ApiTags::Subsystems")]
impl SubsystemsApi {
    #[oai(path = "/v1/subsystems", method = "get")]
    pub async fn list_all_subsystems(&self, state: Data<&AppState>) -> Json<Vec<SubsystemDto>> {
        let subs = state.db.list_all_subsystems().await.unwrap_or_default();
        Json(
            subs.into_iter()
                .map(|s| SubsystemDto {
                    id: s.id,
                    member_id: s.member_id,
                    subsystem_code: s.subsystem_code,
                })
                .collect(),
        )
    }

    #[oai(path = "/v1/subsystems/:subsystem_id", method = "get")]
    pub async fn get_subsystem(
        &self,
        state: Data<&AppState>,
        subsystem_id: Path<i64>,
    ) -> poem::Result<Json<SubsystemDto>> {
        let sub = state
            .db
            .get_subsystem(subsystem_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .ok_or_else(|| {
                poem::Error::from_string("Subsystem not found", StatusCode::NOT_FOUND)
            })?;
        Ok(Json(SubsystemDto {
            id: sub.id,
            member_id: sub.member_id,
            subsystem_code: sub.subsystem_code,
        }))
    }

    #[oai(path = "/v1/subsystems/:subsystem_id", method = "delete")]
    pub async fn delete_subsystem(
        &self,
        state: Data<&AppState>,
        subsystem_id: Path<i64>,
    ) -> poem::Result<Json<bool>> {
        let ok = state
            .db
            .delete_subsystem(subsystem_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        Ok(Json(ok))
    }
}
