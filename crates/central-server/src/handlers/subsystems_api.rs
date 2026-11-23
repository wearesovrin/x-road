use crate::handlers::dto::{AddSubsystemRequest, SubsystemDto, UpdateSubsystemRequest};
use crate::handlers::{ApiTags, AppState};
use poem::{http::StatusCode, web::Data};
use poem_openapi::{param::Path, payload::Json, OpenApi};

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

    #[oai(path = "/v1/subsystems", method = "post")]
    pub async fn add_subsystem(
        &self,
        state: Data<&AppState>,
        body: Json<AddSubsystemRequest>,
    ) -> poem::Result<Json<SubsystemDto>> {
        let member_id = body.0.member_id.unwrap_or(1);
        let sub = state
            .db
            .add_subsystem(member_id, &body.0.subsystem_code)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(SubsystemDto {
            id: sub.id,
            member_id: sub.member_id,
            subsystem_code: sub.subsystem_code,
        }))
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

    #[oai(path = "/v1/subsystems/:subsystem_id", method = "patch")]
    pub async fn update_subsystem(
        &self,
        state: Data<&AppState>,
        subsystem_id: Path<i64>,
        body: Json<UpdateSubsystemRequest>,
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
            subsystem_code: body.0.subsystem_code,
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

    #[oai(
        path = "/v1/subsystems/:subsystem_id/servers/:server_id",
        method = "delete"
    )]
    pub async fn unregister_subsystem_server(
        &self,
        _state: Data<&AppState>,
        _subsystem_id: Path<i64>,
        _server_id: Path<i64>,
    ) -> Json<bool> {
        Json(true)
    }
}
