use identifiers::ClientId;
use poem::web::Data;
use poem_openapi::{OpenApi, param::Path, payload::Json};

use super::AppState;
use super::dto::{AddGroupMemberReq, AddLocalGroupReq, LocalGroupDto, SimpleSuccessResponse};

pub struct LocalGroupsApi;

#[OpenApi]
impl LocalGroupsApi {
    /// Lists local groups configured for a client.
    #[oai(path = "/v1/clients/:id/local-groups", method = "get")]
    pub async fn list_client_local_groups(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
    ) -> poem::Result<Json<Vec<LocalGroupDto>>> {
        let client_id: ClientId = id.0.parse().map_err(poem::error::BadRequest)?;
        let groups = state
            .server_conf
            .list_local_groups(&client_id)
            .await
            .map_err(poem::error::InternalServerError)?;

        let mut list = Vec::new();
        for g in groups {
            let members = state
                .server_conf
                .list_group_members(g.id)
                .await
                .unwrap_or_default();
            list.push(LocalGroupDto {
                id: g.id,
                group_code: g.group_code,
                description: g.description,
                updated_at: g.updated.to_rfc3339(),
                member_count: members.len(),
            });
        }
        Ok(Json(list))
    }

    /// Creates a local group for a client.
    #[oai(path = "/v1/clients/:id/local-groups", method = "post")]
    pub async fn create_local_group(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
        body: Json<AddLocalGroupReq>,
    ) -> poem::Result<Json<LocalGroupDto>> {
        let client_id: ClientId = id.0.parse().map_err(poem::error::BadRequest)?;
        let g = state
            .server_conf
            .create_local_group(&client_id, &body.group_code, &body.description)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(LocalGroupDto {
            id: g.id,
            group_code: g.group_code,
            description: g.description,
            updated_at: g.updated.to_rfc3339(),
            member_count: 0,
        }))
    }

    /// Gets a local group by ID.
    #[oai(path = "/v1/local-groups/:id", method = "get")]
    pub async fn get_local_group(
        &self,
        state: Data<&AppState>,
        id: Path<i64>,
    ) -> poem::Result<Json<LocalGroupDto>> {
        let g = state
            .server_conf
            .get_local_group(id.0)
            .await
            .map_err(poem::error::InternalServerError)?
            .ok_or_else(|| {
                poem::Error::from_string("Local group not found", poem::http::StatusCode::NOT_FOUND)
            })?;

        let members = state
            .server_conf
            .list_group_members(g.id)
            .await
            .unwrap_or_default();
        Ok(Json(LocalGroupDto {
            id: g.id,
            group_code: g.group_code,
            description: g.description,
            updated_at: g.updated.to_rfc3339(),
            member_count: members.len(),
        }))
    }

    /// Deletes a local group by ID.
    #[oai(path = "/v1/local-groups/:id", method = "delete")]
    pub async fn delete_local_group(
        &self,
        state: Data<&AppState>,
        id: Path<i64>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        state
            .server_conf
            .delete_local_group(id.0)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Local group {} deleted", id.0),
        }))
    }

    /// Lists member clients of a local group.
    #[oai(path = "/v1/local-groups/:id/members", method = "get")]
    pub async fn list_local_group_members(
        &self,
        state: Data<&AppState>,
        id: Path<i64>,
    ) -> poem::Result<Json<Vec<String>>> {
        let members = state
            .server_conf
            .list_group_members(id.0)
            .await
            .unwrap_or_default();
        Ok(Json(members))
    }

    /// Adds a member client to a local group.
    #[oai(path = "/v1/local-groups/:id/members", method = "post")]
    pub async fn add_local_group_member(
        &self,
        state: Data<&AppState>,
        id: Path<i64>,
        body: Json<AddGroupMemberReq>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        let client_id: ClientId = body.member_id.parse().map_err(poem::error::BadRequest)?;
        state
            .server_conf
            .add_group_member(id.0, &client_id)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Member {} added to local group {}", body.member_id, id.0),
        }))
    }

    /// Removes a member client from a local group.
    #[oai(path = "/v1/local-groups/:id/members/delete", method = "post")]
    pub async fn remove_local_group_member(
        &self,
        state: Data<&AppState>,
        id: Path<i64>,
        body: Json<AddGroupMemberReq>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        let client_id: ClientId = body.member_id.parse().map_err(poem::error::BadRequest)?;
        state
            .server_conf
            .remove_group_member(id.0, &client_id)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!(
                "Member {} removed from local group {}",
                body.member_id, id.0
            ),
        }))
    }
}
