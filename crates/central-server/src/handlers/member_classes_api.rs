use crate::handlers::dto::{AddMemberClassRequest, MemberClassDto, UpdateMemberClassRequest};
use crate::handlers::{ApiTags, AppState};
use poem::{http::StatusCode, web::Data};
use poem_openapi::{param::Path, payload::Json, OpenApi};

pub struct MemberClassesApi;

#[OpenApi(tag = "ApiTags::MemberClasses")]
impl MemberClassesApi {
    #[oai(path = "/v1/member-classes", method = "get")]
    pub async fn list_member_classes(&self, state: Data<&AppState>) -> Json<Vec<MemberClassDto>> {
        let classes = state.db.list_member_classes().await.unwrap_or_default();
        let dtos = classes
            .into_iter()
            .map(|c| MemberClassDto {
                code: c.code,
                description: c.description,
            })
            .collect();
        Json(dtos)
    }

    #[oai(path = "/v1/member-classes", method = "post")]
    pub async fn add_member_class(
        &self,
        state: Data<&AppState>,
        body: Json<AddMemberClassRequest>,
    ) -> poem::Result<Json<MemberClassDto>> {
        let req = body.0;
        let created = state
            .db
            .add_member_class(&req.code, &req.description)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(MemberClassDto {
            code: created.code,
            description: created.description,
        }))
    }

    #[oai(path = "/v1/member-classes/:code", method = "get")]
    pub async fn get_member_class(
        &self,
        state: Data<&AppState>,
        code: Path<String>,
    ) -> poem::Result<Json<MemberClassDto>> {
        let c = state
            .db
            .get_member_class(&code.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .ok_or_else(|| {
                poem::Error::from_string("Member class not found", StatusCode::NOT_FOUND)
            })?;
        Ok(Json(MemberClassDto {
            code: c.code,
            description: c.description,
        }))
    }

    #[oai(path = "/v1/member-classes/:code", method = "patch")]
    pub async fn update_member_class(
        &self,
        state: Data<&AppState>,
        code: Path<String>,
        body: Json<UpdateMemberClassRequest>,
    ) -> poem::Result<Json<MemberClassDto>> {
        let _ = state.db.delete_member_class(&code.0).await;
        let created = state
            .db
            .add_member_class(&code.0, &body.0.description)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;
        Ok(Json(MemberClassDto {
            code: created.code,
            description: created.description,
        }))
    }

    #[oai(path = "/v1/member-classes/:code", method = "delete")]
    pub async fn delete_member_class(
        &self,
        state: Data<&AppState>,
        code: Path<String>,
    ) -> poem::Result<Json<bool>> {
        let ok = state.db.delete_member_class(&code.0).await.map_err(|e| {
            poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
        })?;
        Ok(Json(ok))
    }
}
