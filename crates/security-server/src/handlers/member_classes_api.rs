use crate::handlers::dto::{
    MemberClassDto, MemberNameDto, SecurityServerDto, XRoadInstanceDto,
};
use crate::handlers::{ApiTags, AppState};
use poem::web::Data;
use poem_openapi::{param::Path, payload::Json, OpenApi};

pub struct MemberClassesApi;

#[OpenApi(tag = "ApiTags::MemberClasses")]
impl MemberClassesApi {
    #[oai(path = "/v1/member-classes", method = "get")]
    pub async fn get_member_classes(
        &self,
        _state: Data<&AppState>,
    ) -> Json<Vec<MemberClassDto>> {
        Json(vec![
            MemberClassDto {
                code: "GOV".to_string(),
                description: "Governmental organizations".to_string(),
            },
            MemberClassDto {
                code: "COM".to_string(),
                description: "Commercial organizations".to_string(),
            },
        ])
    }

    #[oai(path = "/v1/member-classes/:id", method = "get")]
    pub async fn get_member_classes_for_instance(
        &self,
        _state: Data<&AppState>,
        _id: Path<String>,
    ) -> Json<Vec<MemberClassDto>> {
        Json(vec![
            MemberClassDto {
                code: "GOV".to_string(),
                description: "Governmental organizations".to_string(),
            },
            MemberClassDto {
                code: "COM".to_string(),
                description: "Commercial organizations".to_string(),
            },
        ])
    }

    #[oai(path = "/v1/member-names", method = "get")]
    pub async fn find_member_name(&self, _state: Data<&AppState>) -> Json<MemberNameDto> {
        Json(MemberNameDto {
            member_name: "Example Organization".to_string(),
        })
    }

    #[oai(path = "/v1/language/:code", method = "put")]
    pub async fn set_language(
        &self,
        _state: Data<&AppState>,
        _code: Path<String>,
    ) -> Json<bool> {
        Json(true)
    }

    #[oai(path = "/v1/xroad-instances", method = "get")]
    pub async fn get_xroad_instances(
        &self,
        state: Data<&AppState>,
    ) -> Json<Vec<XRoadInstanceDto>> {
        let inst = &state.global_conf.instance_identifier;
        Json(vec![XRoadInstanceDto {
            instance_identifier: inst.clone(),
        }])
    }

    #[oai(path = "/v1/security-servers", method = "get")]
    pub async fn get_security_servers(
        &self,
        state: Data<&AppState>,
    ) -> Json<Vec<SecurityServerDto>> {
        let servers = &state.global_conf.security_servers;
        let dtos = servers
            .iter()
            .map(|(srv_id, address)| SecurityServerDto {
                id: srv_id.server_code.clone(),
                server_code: srv_id.server_code.clone(),
                server_address: address.clone(),
                owner_client_id: srv_id.owner().to_string(),
            })
            .collect();
        Json(dtos)
    }

    #[oai(path = "/v1/security-servers/:id", method = "get")]
    pub async fn get_security_server(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
    ) -> poem::Result<Json<SecurityServerDto>> {
        let servers = &state.global_conf.security_servers;
        let s = servers
            .iter()
            .find(|(srv_id, _)| srv_id.server_code == id.0)
            .ok_or_else(|| {
                poem::Error::from_string("Security Server not found", poem::http::StatusCode::NOT_FOUND)
            })?;
        Ok(Json(SecurityServerDto {
            id: s.0.server_code.clone(),
            server_code: s.0.server_code.clone(),
            server_address: s.1.clone(),
            owner_client_id: s.0.owner().to_string(),
        }))
    }
}
