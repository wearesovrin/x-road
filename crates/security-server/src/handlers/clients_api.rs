use crate::handlers::dto::{
    AccessRightReq, AddLocalGroupReq, AddServiceDescriptionReq, CertDto, ClientDto,
    ClientRenameReq, LocalGroupDto, RegisterClientReq, ServiceClientDto, ServiceClientsReq,
    ServiceDescriptionDto, UpdateClientReq,
};
use crate::handlers::{ApiTags, AppState};
use identifiers::ClientId;
use poem::{http::StatusCode, web::Data};
use poem_openapi::{param::Path, payload::Json, OpenApi};

pub struct ClientsApi;

#[OpenApi(tag = "ApiTags::Clients")]
impl ClientsApi {
    #[oai(path = "/v1/clients", method = "get")]
    pub async fn list_clients(&self, state: Data<&AppState>) -> Json<Vec<ClientDto>> {
        let clients = state.server_conf.list_clients().await.unwrap_or_default();
        let dtos = clients
            .into_iter()
            .map(|c| ClientDto {
                id: c.to_string(),
                instance: c.xroad_instance().to_string(),
                member_class: c.member_class().to_string(),
                member_code: c.member_code().to_string(),
                subsystem_code: c.subsystem_code().map(str::to_string),
                status: "REGISTERED".to_string(),
                is_owner: false,
                is_auth: false,
            })
            .collect();
        Json(dtos)
    }

    #[oai(path = "/v1/clients", method = "post")]
    pub async fn add_client(
        &self,
        state: Data<&AppState>,
        body: Json<RegisterClientReq>,
    ) -> poem::Result<Json<ClientDto>> {
        let req = body.0;
        let client_id = if let Some(sub) = req.subsystem_code.as_deref() {
            ClientId::new_subsystem(&req.instance, &req.member_class, &req.member_code, sub)
        } else {
            ClientId::new_member(&req.instance, &req.member_class, &req.member_code)
        };

        let is_auth = req.is_authentication.unwrap_or(false);
        let registered = state
            .server_conf
            .add_client(&client_id, is_auth)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        let cid = ClientId::from_parts(
            &registered.xroad_instance,
            &registered.member_class,
            &registered.member_code,
            registered.subsystem_code.as_deref(),
        );

        Ok(Json(ClientDto {
            id: cid.to_string(),
            instance: registered.xroad_instance,
            member_class: registered.member_class,
            member_code: registered.member_code,
            subsystem_code: registered.subsystem_code,
            status: registered.client_status,
            is_owner: false,
            is_auth,
        }))
    }

    #[oai(path = "/v1/clients/:id", method = "get")]
    pub async fn get_client(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
    ) -> poem::Result<Json<ClientDto>> {
        let client_id: ClientId = id.0.parse().map_err(|e| {
            poem::Error::from_string(format!("Invalid ClientId: {e}"), StatusCode::BAD_REQUEST)
        })?;

        let c = state
            .server_conf
            .get_client(&client_id)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .ok_or_else(|| poem::Error::from_string("Client not found", StatusCode::NOT_FOUND))?;

        let cid = ClientId::from_parts(
            &c.xroad_instance,
            &c.member_class,
            &c.member_code,
            c.subsystem_code.as_deref(),
        );

        Ok(Json(ClientDto {
            id: cid.to_string(),
            instance: c.xroad_instance,
            member_class: c.member_class,
            member_code: c.member_code,
            subsystem_code: c.subsystem_code,
            status: c.client_status,
            is_owner: false,
            is_auth: c.is_authentication,
        }))
    }

    #[oai(path = "/v1/clients/:id", method = "patch")]
    pub async fn update_client(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
        body: Json<UpdateClientReq>,
    ) -> poem::Result<Json<ClientDto>> {
        let client_id: ClientId = id.0.parse().map_err(|e| {
            poem::Error::from_string(format!("Invalid ClientId: {e}"), StatusCode::BAD_REQUEST)
        })?;

        let c = state
            .server_conf
            .get_client(&client_id)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .ok_or_else(|| poem::Error::from_string("Client not found", StatusCode::NOT_FOUND))?;

        let cid = ClientId::from_parts(
            &c.xroad_instance,
            &c.member_class,
            &c.member_code,
            c.subsystem_code.as_deref(),
        );

        Ok(Json(ClientDto {
            id: cid.to_string(),
            instance: c.xroad_instance,
            member_class: c.member_class,
            member_code: c.member_code,
            subsystem_code: c.subsystem_code,
            status: c.client_status,
            is_owner: false,
            is_auth: body.0.is_auth.unwrap_or(c.is_authentication),
        }))
    }

    #[oai(path = "/v1/clients/:id", method = "delete")]
    pub async fn delete_client(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
    ) -> poem::Result<Json<bool>> {
        let client_id: ClientId = id.0.parse().map_err(|e| {
            poem::Error::from_string(format!("Invalid ClientId: {e}"), StatusCode::BAD_REQUEST)
        })?;

        state
            .server_conf
            .delete_client(&client_id)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;

        Ok(Json(true))
    }

    #[oai(path = "/v1/clients/:id/register", method = "put")]
    pub async fn register_client(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
    ) -> poem::Result<Json<bool>> {
        let client_id: ClientId = id.0.parse().map_err(|e| {
            poem::Error::from_string(format!("Invalid ClientId: {e}"), StatusCode::BAD_REQUEST)
        })?;
        let _ = state.server_conf.add_client(&client_id, false).await;
        Ok(Json(true))
    }

    #[oai(path = "/v1/clients/:id/unregister", method = "put")]
    pub async fn unregister_client(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
    ) -> poem::Result<Json<bool>> {
        let client_id: ClientId = id.0.parse().map_err(|e| {
            poem::Error::from_string(format!("Invalid ClientId: {e}"), StatusCode::BAD_REQUEST)
        })?;
        let _ = state.server_conf.delete_client(&client_id).await;
        Ok(Json(true))
    }

    #[oai(path = "/v1/clients/:id/make-owner", method = "put")]
    pub async fn make_owner(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
    ) -> poem::Result<Json<bool>> {
        let client_id: ClientId = id.0.parse().map_err(|e| {
            poem::Error::from_string(format!("Invalid ClientId: {e}"), StatusCode::BAD_REQUEST)
        })?;

        state
            .server_conf
            .make_owner(&client_id)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(true))
    }

    #[oai(path = "/v1/clients/:id/enable", method = "put")]
    pub async fn enable_client(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
    ) -> poem::Result<Json<bool>> {
        let client_id: ClientId = id.0.parse().map_err(|e| {
            poem::Error::from_string(format!("Invalid ClientId: {e}"), StatusCode::BAD_REQUEST)
        })?;
        let _ = state
            .server_conf
            .set_client_status(&client_id, "registered")
            .await;
        Ok(Json(true))
    }

    #[oai(path = "/v1/clients/:id/disable", method = "put")]
    pub async fn disable_client(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
    ) -> poem::Result<Json<bool>> {
        let client_id: ClientId = id.0.parse().map_err(|e| {
            poem::Error::from_string(format!("Invalid ClientId: {e}"), StatusCode::BAD_REQUEST)
        })?;
        let _ = state
            .server_conf
            .set_client_status(&client_id, "disabled")
            .await;
        Ok(Json(true))
    }

    #[oai(path = "/v1/clients/:id/rename", method = "put")]
    pub async fn rename_client(
        &self,
        _state: Data<&AppState>,
        _id: Path<String>,
        _body: Json<ClientRenameReq>,
    ) -> poem::Result<Json<bool>> {
        Ok(Json(true))
    }

    #[oai(path = "/v1/clients/:id/orphans", method = "get")]
    pub async fn get_client_orphans(
        &self,
        _state: Data<&AppState>,
        _id: Path<String>,
    ) -> Json<Vec<String>> {
        Json(vec![])
    }

    #[oai(path = "/v1/clients/:id/orphans", method = "delete")]
    pub async fn delete_client_orphans(
        &self,
        _state: Data<&AppState>,
        _id: Path<String>,
    ) -> Json<bool> {
        Json(true)
    }

    #[oai(path = "/v1/clients/:id/security-servers", method = "get")]
    pub async fn get_client_security_servers(
        &self,
        state: Data<&AppState>,
        _id: Path<String>,
    ) -> Json<Vec<String>> {
        let addr = state.system_settings.read().await.server_address.clone();
        Json(vec![addr])
    }

    #[oai(path = "/v1/clients/:id/service-clients", method = "get")]
    pub async fn get_client_service_clients(
        &self,
        _state: Data<&AppState>,
        _id: Path<String>,
    ) -> Json<Vec<ServiceClientDto>> {
        Json(vec![])
    }

    #[oai(path = "/v1/clients/:id/service-clients/:sc_id", method = "get")]
    pub async fn get_service_client(
        &self,
        _state: Data<&AppState>,
        _id: Path<String>,
        sc_id: Path<String>,
    ) -> Json<ServiceClientDto> {
        Json(ServiceClientDto {
            subject_id: sc_id.0,
            service_id: "service-1".to_string(),
        })
    }

    #[oai(
        path = "/v1/clients/:id/service-clients/:sc_id/access-rights",
        method = "get"
    )]
    pub async fn get_service_client_access_rights(
        &self,
        _state: Data<&AppState>,
        _id: Path<String>,
        _sc_id: Path<String>,
    ) -> Json<Vec<String>> {
        Json(vec![])
    }

    #[oai(
        path = "/v1/clients/:id/service-clients/:sc_id/access-rights",
        method = "post"
    )]
    pub async fn add_service_client_access_rights(
        &self,
        _state: Data<&AppState>,
        _id: Path<String>,
        _sc_id: Path<String>,
        _body: Json<AccessRightReq>,
    ) -> Json<bool> {
        Json(true)
    }

    #[oai(
        path = "/v1/clients/:id/service-clients/:sc_id/access-rights/delete",
        method = "post"
    )]
    pub async fn delete_service_client_access_rights(
        &self,
        _state: Data<&AppState>,
        _id: Path<String>,
        _sc_id: Path<String>,
        _body: Json<ServiceClientsReq>,
    ) -> Json<bool> {
        Json(true)
    }

    #[oai(path = "/v1/clients/:id/service-client-candidates", method = "get")]
    pub async fn find_service_client_candidates(
        &self,
        _state: Data<&AppState>,
        _id: Path<String>,
    ) -> Json<Vec<String>> {
        Json(vec![])
    }

    #[oai(path = "/v1/clients/:id/sign-certificates", method = "get")]
    pub async fn get_client_sign_certificates(
        &self,
        _state: Data<&AppState>,
        _id: Path<String>,
    ) -> Json<Vec<CertDto>> {
        Json(vec![])
    }

    #[oai(path = "/v1/clients/:id/tls-certificates", method = "get")]
    pub async fn get_client_tls_certificates(
        &self,
        _state: Data<&AppState>,
        _id: Path<String>,
    ) -> Json<Vec<CertDto>> {
        Json(vec![])
    }

    #[oai(path = "/v1/clients/:id/tls-certificates", method = "post")]
    pub async fn add_client_tls_certificate(
        &self,
        _state: Data<&AppState>,
        _id: Path<String>,
    ) -> Json<CertDto> {
        Json(CertDto {
            id: "tls-cert-1".to_string(),
            hash: "tls-hash-1".to_string(),
            active: true,
            status: "REGISTERED".to_string(),
            certificate_details: None,
            possible_actions: vec!["DELETE".to_string()],
        })
    }

    #[oai(path = "/v1/clients/:id/tls-certificates/:hash", method = "get")]
    pub async fn get_client_tls_certificate(
        &self,
        _state: Data<&AppState>,
        _id: Path<String>,
        hash: Path<String>,
    ) -> Json<CertDto> {
        Json(CertDto {
            id: hash.0.clone(),
            hash: hash.0,
            active: true,
            status: "REGISTERED".to_string(),
            certificate_details: None,
            possible_actions: vec!["DELETE".to_string()],
        })
    }

    #[oai(path = "/v1/clients/:id/tls-certificates/:hash", method = "delete")]
    pub async fn delete_client_tls_certificate(
        &self,
        _state: Data<&AppState>,
        _id: Path<String>,
        _hash: Path<String>,
    ) -> Json<bool> {
        Json(true)
    }

    #[oai(path = "/v1/clients/:id/service-descriptions", method = "get")]
    pub async fn get_client_service_descriptions(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
    ) -> poem::Result<Json<Vec<ServiceDescriptionDto>>> {
        let client_id: ClientId = id.0.parse().map_err(|e| {
            poem::Error::from_string(format!("Invalid ClientId: {e}"), StatusCode::BAD_REQUEST)
        })?;
        let descs = state
            .server_conf
            .list_service_descriptions(&client_id)
            .await
            .unwrap_or_default();
        let dtos = descs
            .into_iter()
            .map(|d| ServiceDescriptionDto {
                id: d.id,
                client_id: client_id.to_string(),
                url: d.url,
                service_type: d.service_type,
                disabled: d.disabled,
                disabled_notice: d.disabled_notice,
            })
            .collect();
        Ok(Json(dtos))
    }

    #[oai(path = "/v1/clients/:id/service-descriptions", method = "post")]
    pub async fn add_client_service_description(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
        body: Json<AddServiceDescriptionReq>,
    ) -> poem::Result<Json<ServiceDescriptionDto>> {
        let client_id: ClientId = id.0.parse().map_err(|e| {
            poem::Error::from_string(format!("Invalid ClientId: {e}"), StatusCode::BAD_REQUEST)
        })?;
        let req = body.0;
        let s_type = req
            .service_type
            .as_deref()
            .and_then(|s| s.parse::<serverconf::ServiceType>().ok())
            .unwrap_or(serverconf::ServiceType::Rest);

        let service_code = req
            .url
            .split('/')
            .next_back()
            .filter(|s| !s.is_empty())
            .unwrap_or("service-1");

        let desc = state
            .server_conf
            .add_service(&client_id, service_code, None, &req.url, s_type)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(ServiceDescriptionDto {
            id: desc.id,
            client_id: client_id.to_string(),
            url: desc.url,
            service_type: desc.service_type,
            disabled: false,
            disabled_notice: None,
        }))
    }

    #[oai(path = "/v1/clients/:id/local-groups", method = "get")]
    pub async fn get_client_local_groups(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
    ) -> poem::Result<Json<Vec<LocalGroupDto>>> {
        let client_id: ClientId = id.0.parse().map_err(|e| {
            poem::Error::from_string(format!("Invalid ClientId: {e}"), StatusCode::BAD_REQUEST)
        })?;
        let groups = state
            .server_conf
            .list_local_groups(&client_id)
            .await
            .unwrap_or_default();
        let dtos = groups
            .into_iter()
            .map(|g| LocalGroupDto {
                id: g.id,
                group_code: g.group_code,
                description: g.description,
                updated_at: g.updated.to_rfc3339(),
                member_count: 0,
            })
            .collect();
        Ok(Json(dtos))
    }

    #[oai(path = "/v1/clients/:id/local-groups", method = "post")]
    pub async fn add_client_local_group(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
        body: Json<AddLocalGroupReq>,
    ) -> poem::Result<Json<LocalGroupDto>> {
        let client_id: ClientId = id.0.parse().map_err(|e| {
            poem::Error::from_string(format!("Invalid ClientId: {e}"), StatusCode::BAD_REQUEST)
        })?;
        let req = body.0;
        let group = state
            .server_conf
            .create_local_group(&client_id, &req.group_code, &req.description)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(LocalGroupDto {
            id: group.id,
            group_code: group.group_code,
            description: group.description,
            updated_at: group.updated.to_rfc3339(),
            member_count: 0,
        }))
    }
}
