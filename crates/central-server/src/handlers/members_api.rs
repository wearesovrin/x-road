use crate::handlers::dto::{
    AddMemberRequest, AddSubsystemRequest, MemberDto, SecurityServerDto, SubsystemDto,
    UpdateMemberRequest,
};
use crate::services::CentralServerDb;
use poem::http::StatusCode;
use poem_openapi::{OpenApi, param::Path, payload::Json};

pub struct MembersApi {
    pub instance_id: String,
    pub db: CentralServerDb,
}

#[OpenApi]
impl MembersApi {
    #[oai(path = "/v1/members", method = "get")]
    pub async fn list_members(&self) -> Json<Vec<MemberDto>> {
        let members = self.db.list_members().await.unwrap_or_default();
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
    pub async fn add_member(&self, body: Json<AddMemberRequest>) -> poem::Result<Json<MemberDto>> {
        let req = body.0;
        let member = self
            .db
            .add_member(
                &self.instance_id,
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
    pub async fn get_member(&self, member_id: Path<i64>) -> poem::Result<Json<MemberDto>> {
        let m = self
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
        member_id: Path<i64>,
        body: Json<UpdateMemberRequest>,
    ) -> poem::Result<Json<MemberDto>> {
        let m = self
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
    pub async fn delete_member(&self, member_id: Path<i64>) -> poem::Result<Json<bool>> {
        let ok = self.db.delete_member(member_id.0).await.map_err(|e| {
            poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
        })?;
        Ok(Json(ok))
    }

    #[oai(path = "/v1/members/:member_id/subsystems", method = "get")]
    pub async fn list_subsystems(
        &self,
        member_id: Path<i64>,
    ) -> poem::Result<Json<Vec<SubsystemDto>>> {
        let subs = self.db.list_subsystems(member_id.0).await.map_err(|e| {
            poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
        })?;

        let dtos = subs
            .into_iter()
            .map(|s| SubsystemDto {
                id: s.id,
                member_id: s.member_id,
                subsystem_code: s.subsystem_code,
            })
            .collect();
        Ok(Json(dtos))
    }

    #[oai(path = "/v1/members/:member_id/subsystems", method = "post")]
    pub async fn add_subsystem(
        &self,
        member_id: Path<i64>,
        body: Json<AddSubsystemRequest>,
    ) -> poem::Result<Json<SubsystemDto>> {
        let sub = self
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

    #[oai(path = "/v1/members/:member_id/global-groups", method = "get")]
    pub async fn list_member_global_groups(
        &self,
        member_id: Path<i64>,
    ) -> poem::Result<Json<Vec<String>>> {
        let groups = self
            .db
            .list_member_global_groups(member_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        Ok(Json(groups))
    }

    #[oai(path = "/v1/members/:member_id/owned-servers", method = "get")]
    pub async fn list_member_owned_servers(
        &self,
        member_id: Path<i64>,
    ) -> poem::Result<Json<Vec<SecurityServerDto>>> {
        let servers = self
            .db
            .list_member_owned_servers(member_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        Ok(Json(
            servers
                .into_iter()
                .map(|s| SecurityServerDto {
                    id: s.id,
                    owner_id: s.owner_id,
                    server_code: s.server_code,
                    address: s.address,
                })
                .collect(),
        ))
    }
}
