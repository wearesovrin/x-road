use crate::handlers::{ApiTags, AppState};
use crate::handlers::dto::{AddMemberClassRequest, MemberClassDto};
use poem::{http::StatusCode, web::Data};
use poem_openapi::{OpenApi, param::Path, payload::Json};

pub struct MemberClassesApi;

#[OpenApi(tag = "ApiTags::MemberClasses")]
impl MemberClassesApi {
    #[oai(path = "/v1/member-classes", method = "get")]
    pub async fn list_member_classes(&self, state: Data<&AppState>) -> Json<Vec<MemberClassDto>> {
        let classes = state.db.list_member_classes().await.unwrap_or_default();
        Json(
            classes
                .into_iter()
                .map(|c| MemberClassDto {
                    code: c.code,
                    description: c.description,
                })
                .collect(),
        )
    }

    #[oai(path = "/v1/member-classes", method = "post")]
    pub async fn add_member_class(
        &self,
        state: Data<&AppState>,
        body: Json<AddMemberClassRequest>,
    ) -> poem::Result<Json<MemberClassDto>> {
        let mc = state
            .db
            .add_member_class(&body.0.code, &body.0.description)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;
        Ok(Json(MemberClassDto {
            code: mc.code,
            description: mc.description,
        }))
    }

    #[oai(path = "/v1/member-classes/:code", method = "get")]
    pub async fn get_member_class(
        &self,
        state: Data<&AppState>,
        code: Path<String>,
    ) -> poem::Result<Json<MemberClassDto>> {
        let mc = state
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
            code: mc.code,
            description: mc.description,
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
