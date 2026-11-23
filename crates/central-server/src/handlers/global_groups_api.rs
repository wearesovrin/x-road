use crate::handlers::dto::{
    AddGroupMemberRequest, AddGroupMembersRequest, CreateGlobalGroupRequest, GlobalGroupDto,
    GroupMembersFilterModelDto, UpdateGlobalGroupRequest,
};
use crate::handlers::{ApiTags, AppState};
use poem::{http::StatusCode, web::Data};
use poem_openapi::{param::Path, payload::Json, OpenApi};

pub struct GlobalGroupsApi;

#[OpenApi(tag = "ApiTags::GlobalGroups")]
impl GlobalGroupsApi {
    #[oai(path = "/v1/global-groups", method = "get")]
    pub async fn list_global_groups(&self, state: Data<&AppState>) -> Json<Vec<GlobalGroupDto>> {
        let groups = state.db.list_global_groups().await.unwrap_or_default();
        let mut dtos = Vec::new();
        for g in groups {
            let members = state
                .db
                .list_global_group_members(&g.group_code)
                .await
                .unwrap_or_default();
            dtos.push(GlobalGroupDto {
                group_code: g.group_code,
                description: g.description,
                member_count: members.len(),
            });
        }
        Json(dtos)
    }

    #[oai(path = "/v1/global-groups", method = "post")]
    pub async fn create_global_group(
        &self,
        state: Data<&AppState>,
        body: Json<CreateGlobalGroupRequest>,
    ) -> poem::Result<Json<GlobalGroupDto>> {
        let req = body.0;
        let g = state
            .db
            .create_global_group(&req.group_code, &req.description)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(GlobalGroupDto {
            group_code: g.group_code,
            description: g.description,
            member_count: 0,
        }))
    }

    #[oai(path = "/v1/global-groups/:group_code", method = "get")]
    pub async fn get_global_group(
        &self,
        state: Data<&AppState>,
        group_code: Path<String>,
    ) -> poem::Result<Json<GlobalGroupDto>> {
        let g = state
            .db
            .get_global_group(&group_code.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .ok_or_else(|| {
                poem::Error::from_string("Global group not found", StatusCode::NOT_FOUND)
            })?;

        let members = state
            .db
            .list_global_group_members(&g.group_code)
            .await
            .unwrap_or_default();

        Ok(Json(GlobalGroupDto {
            group_code: g.group_code,
            description: g.description,
            member_count: members.len(),
        }))
    }

    #[oai(path = "/v1/global-groups/:group_code", method = "patch")]
    pub async fn update_global_group_description(
        &self,
        state: Data<&AppState>,
        group_code: Path<String>,
        body: Json<UpdateGlobalGroupRequest>,
    ) -> poem::Result<Json<GlobalGroupDto>> {
        let g = state
            .db
            .get_global_group(&group_code.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .ok_or_else(|| {
                poem::Error::from_string("Global group not found", StatusCode::NOT_FOUND)
            })?;

        let members = state
            .db
            .list_global_group_members(&g.group_code)
            .await
            .unwrap_or_default();

        Ok(Json(GlobalGroupDto {
            group_code: g.group_code,
            description: body.0.description,
            member_count: members.len(),
        }))
    }

    #[oai(path = "/v1/global-groups/:group_code", method = "delete")]
    pub async fn delete_global_group(
        &self,
        state: Data<&AppState>,
        group_code: Path<String>,
    ) -> poem::Result<Json<bool>> {
        let ok = state
            .db
            .delete_global_group(&group_code.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        Ok(Json(ok))
    }

    #[oai(path = "/v1/global-groups/:group_code/members", method = "get")]
    pub async fn list_global_group_members(
        &self,
        state: Data<&AppState>,
        group_code: Path<String>,
    ) -> poem::Result<Json<Vec<String>>> {
        let members = state
            .db
            .list_global_group_members(&group_code.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        Ok(Json(members))
    }

    #[oai(path = "/v1/global-groups/:group_code/members", method = "post")]
    pub async fn add_global_group_member(
        &self,
        state: Data<&AppState>,
        group_code: Path<String>,
        body: Json<AddGroupMemberRequest>,
    ) -> poem::Result<Json<bool>> {
        state
            .db
            .add_to_global_group(&group_code.0, &body.0.client_id_str)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;
        Ok(Json(true))
    }

    #[oai(path = "/v1/global-groups/:group_code/members/add", method = "post")]
    pub async fn add_global_group_members(
        &self,
        state: Data<&AppState>,
        group_code: Path<String>,
        body: Json<AddGroupMembersRequest>,
    ) -> poem::Result<Json<bool>> {
        for item in body.0.items {
            let _ = state.db.add_to_global_group(&group_code.0, &item).await;
        }
        Ok(Json(true))
    }

    #[oai(
        path = "/v1/global-groups/:group_code/members/filter-model",
        method = "get"
    )]
    pub async fn get_group_members_filter_model(
        &self,
        state: Data<&AppState>,
        group_code: Path<String>,
    ) -> poem::Result<Json<GroupMembersFilterModelDto>> {
        let members = state
            .db
            .list_global_group_members(&group_code.0)
            .await
            .unwrap_or_default();
        Ok(Json(GroupMembersFilterModelDto {
            group_code: group_code.0,
            members,
        }))
    }

    #[oai(
        path = "/v1/global-groups/:group_code/members/:client_id",
        method = "delete"
    )]
    pub async fn remove_global_group_member(
        &self,
        state: Data<&AppState>,
        group_code: Path<String>,
        client_id: Path<String>,
    ) -> poem::Result<Json<bool>> {
        let ok = state
            .db
            .remove_from_global_group(&group_code.0, &client_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        Ok(Json(ok))
    }
}
