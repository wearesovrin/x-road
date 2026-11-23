use crate::handlers::{ApiTags, AppState};
use crate::handlers::dto::{
    AddMemberRequest, AddSubsystemRequest, MemberDto, SecurityServerDto, SubsystemDto,
    UpdateMemberRequest,
};
use poem::{http::StatusCode, web::Data};
use poem_openapi::{OpenApi, param::Path, payload::Json};

pub struct MembersApi;

#[OpenApi(tag = "ApiTags::Members")]
impl MembersApi {
    #[oai(path = "/v1/members", method = "get")]
    pub async fn list_members(&self, state: Data<&AppState>) -> Json<Vec<MemberDto>> {
        let members = state.db.list_members().await.unwrap_or_default();
        let dtos = members
            .into_iter()
            .map(|m| MemberDto {
                id: m.id,
                instance: m.xroad_instance,
                member_class: m.member_class,
                member_code: m.member_code,
                name: m.name,
                administrative_contact: m.administrative_contact,
            })
            .collect();
        Json(dtos)
    }

    #[oai(path = "/v1/members", method = "post")]
    pub async fn add_member(
        &self,
        state: Data<&AppState>,
        body: Json<AddMemberRequest>,
    ) -> poem::Result<Json<MemberDto>> {
        let req = body.0;
        let member = state
            .db
            .add_member(
                &state.instance_id,
                &req.member_class,
                &req.member_code,
                &req.name,
                req.administrative_contact.as_deref(),
            )
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(MemberDto {
            id: member.id,
            instance: member.xroad_instance,
            member_class: member.member_class,
            member_code: member.member_code,
            name: member.name,
            administrative_contact: member.administrative_contact,
        }))
    }

    #[oai(path = "/v1/members/:member_id", method = "get")]
    pub async fn get_member(
        &self,
        state: Data<&AppState>,
        member_id: Path<i64>,
    ) -> poem::Result<Json<MemberDto>> {
        let m = state
            .db
            .get_member_by_id(member_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .ok_or_else(|| poem::Error::from_string("Member not found", StatusCode::NOT_FOUND))?;
        Ok(Json(MemberDto {
            id: m.id,
            instance: m.xroad_instance,
            member_class: m.member_class,
            member_code: m.member_code,
            name: m.name,
            administrative_contact: m.administrative_contact,
        }))
    }

    #[oai(path = "/v1/members/:member_id", method = "put")]
    pub async fn update_member(
        &self,
        state: Data<&AppState>,
        member_id: Path<i64>,
        body: Json<UpdateMemberRequest>,
    ) -> poem::Result<Json<MemberDto>> {
        let m = state
            .db
            .update_member_name(member_id.0, &body.0.name)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;
        Ok(Json(MemberDto {
            id: m.id,
            instance: m.xroad_instance,
            member_class: m.member_class,
            member_code: m.member_code,
            name: m.name,
            administrative_contact: m.administrative_contact,
        }))
    }

    #[oai(path = "/v1/members/:member_id", method = "delete")]
    pub async fn delete_member(
        &self,
        state: Data<&AppState>,
        member_id: Path<i64>,
    ) -> poem::Result<Json<bool>> {
        let ok = state.db.delete_member(member_id.0).await.map_err(|e| {
            poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
        })?;
        Ok(Json(ok))
    }

    #[oai(path = "/v1/members/:member_id/subsystems", method = "get")]
    pub async fn list_member_subsystems(
        &self,
        state: Data<&AppState>,
        member_id: Path<i64>,
    ) -> Json<Vec<SubsystemDto>> {
        let subs = state
            .db
            .list_subsystems(member_id.0)
            .await
            .unwrap_or_default();
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

    #[oai(path = "/v1/members/:member_id/subsystems", method = "post")]
    pub async fn add_member_subsystem(
        &self,
        state: Data<&AppState>,
        member_id: Path<i64>,
        body: Json<AddSubsystemRequest>,
    ) -> poem::Result<Json<SubsystemDto>> {
        let sub = state
            .db
            .add_subsystem(member_id.0, &body.0.subsystem_code)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(SubsystemDto {
            id: sub.id,
            member_id: sub.member_id,
            subsystem_code: sub.subsystem_code,
        }))
    }

    #[oai(path = "/v1/members/:member_id/security-servers", method = "get")]
    pub async fn list_member_security_servers(
        &self,
        state: Data<&AppState>,
        member_id: Path<i64>,
    ) -> Json<Vec<SecurityServerDto>> {
        let servers = state
            .db
            .list_security_servers()
            .await
            .unwrap_or_default()
            .into_iter()
            .filter(|s| s.owner_id == member_id.0)
            .map(|s| SecurityServerDto {
                id: s.id,
                owner_id: s.owner_id,
                server_code: s.server_code,
                address: s.address,
            })
            .collect();
        Json(servers)
    }
}
