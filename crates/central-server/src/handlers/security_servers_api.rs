use crate::handlers::{ApiTags, AppState};
use crate::handlers::dto::{
    AddAuthCertRequest, AddServerClientRequest, LinkClientRequest, RegisterSecurityServerRequest,
    SecurityServerAuthCertDto, SecurityServerDto,
};
use poem::{http::StatusCode, web::Data};
use poem_openapi::{OpenApi, param::Path, payload::Json};

pub struct SecurityServersApi;

#[OpenApi(tag = "ApiTags::SecurityServers")]
impl SecurityServersApi {
    #[oai(path = "/v1/security-servers", method = "get")]
    pub async fn list_security_servers(
        &self,
        state: Data<&AppState>,
    ) -> Json<Vec<SecurityServerDto>> {
        let servers = state.db.list_security_servers().await.unwrap_or_default();
        let dtos = servers
            .into_iter()
            .map(|s| SecurityServerDto {
                id: s.id,
                owner_id: s.owner_id,
                server_code: s.server_code,
                address: s.address,
            })
            .collect();
        Json(dtos)
    }

    #[oai(path = "/v1/security-servers", method = "post")]
    pub async fn register_security_server(
        &self,
        state: Data<&AppState>,
        body: Json<RegisterSecurityServerRequest>,
    ) -> poem::Result<Json<SecurityServerDto>> {
        let req = body.0;
        let server = state
            .db
            .register_security_server(req.owner_id, &req.server_code, &req.address)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(SecurityServerDto {
            id: server.id,
            owner_id: server.owner_id,
            server_code: server.server_code,
            address: server.address,
        }))
    }

    #[oai(path = "/v1/security-servers/:server_id", method = "get")]
    pub async fn get_security_server(
        &self,
        state: Data<&AppState>,
        server_id: Path<i64>,
    ) -> poem::Result<Json<SecurityServerDto>> {
        let s = state
            .db
            .get_security_server(server_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .ok_or_else(|| {
                poem::Error::from_string("Security Server not found", StatusCode::NOT_FOUND)
            })?;
        Ok(Json(SecurityServerDto {
            id: s.id,
            owner_id: s.owner_id,
            server_code: s.server_code,
            address: s.address,
        }))
    }

    #[oai(path = "/v1/security-servers/:server_id", method = "delete")]
    pub async fn delete_security_server(
        &self,
        state: Data<&AppState>,
        server_id: Path<i64>,
    ) -> poem::Result<Json<bool>> {
        let ok = state
            .db
            .delete_security_server(server_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        Ok(Json(ok))
    }

    #[oai(path = "/v1/security-servers/:server_id/clients", method = "get")]
    pub async fn list_server_clients(
        &self,
        state: Data<&AppState>,
        server_id: Path<i64>,
    ) -> poem::Result<Json<Vec<String>>> {
        let clients = state
            .db
            .list_security_server_clients(server_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        Ok(Json(clients))
    }

    #[oai(path = "/v1/security-servers/:server_id/clients", method = "post")]
    pub async fn add_server_client(
        &self,
        state: Data<&AppState>,
        server_id: Path<i64>,
        body: Json<AddServerClientRequest>,
    ) -> poem::Result<Json<bool>> {
        let client_id: identifiers::ClientId = body.0.client_id_str.parse().map_err(|e| {
            poem::Error::from_string(format!("Invalid ClientId: {e}"), StatusCode::BAD_REQUEST)
        })?;

        state
            .db
            .add_security_server_client(server_id.0, &client_id)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(true))
    }

    #[oai(
        path = "/v1/security-servers/:server_id/clients/:client_id",
        method = "delete"
    )]
    pub async fn remove_server_client(
        &self,
        state: Data<&AppState>,
        server_id: Path<i64>,
        client_id: Path<String>,
    ) -> poem::Result<Json<bool>> {
        let ok = state
            .db
            .remove_security_server_client(server_id.0, &client_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        Ok(Json(ok))
    }

    #[oai(path = "/v1/security-servers/client", method = "post")]
    pub async fn link_client(
        &self,
        state: Data<&AppState>,
        body: Json<LinkClientRequest>,
    ) -> poem::Result<Json<bool>> {
        let req = body.0;
        let client_id: identifiers::ClientId = req.client_id_str.parse().map_err(|e| {
            poem::Error::from_string(format!("Invalid ClientId: {e}"), StatusCode::BAD_REQUEST)
        })?;

        state
            .db
            .add_security_server_client(req.security_server_id, &client_id)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(true))
    }

    #[oai(
        path = "/v1/security-servers/:server_id/authentication-certificates",
        method = "get"
    )]
    pub async fn list_server_auth_certs(
        &self,
        state: Data<&AppState>,
        server_id: Path<i64>,
    ) -> poem::Result<Json<Vec<SecurityServerAuthCertDto>>> {
        let certs = state
            .db
            .list_security_server_auth_certs(server_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;

        Ok(Json(
            certs
                .into_iter()
                .map(|c| SecurityServerAuthCertDto {
                    id: c.id,
                    security_server_id: c.security_server_id,
                    cert_hash: c.cert_hash,
                    created_at: c.created_at.to_rfc3339(),
                })
                .collect(),
        ))
    }

    #[oai(
        path = "/v1/security-servers/:server_id/authentication-certificates",
        method = "post"
    )]
    pub async fn add_server_auth_cert(
        &self,
        state: Data<&AppState>,
        server_id: Path<i64>,
        body: Json<AddAuthCertRequest>,
    ) -> poem::Result<Json<SecurityServerAuthCertDto>> {
        let cert = state
            .db
            .add_security_server_auth_cert(server_id.0, &body.0.cert_pem)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(SecurityServerAuthCertDto {
            id: cert.id,
            security_server_id: cert.security_server_id,
            cert_hash: cert.cert_hash,
            created_at: cert.created_at.to_rfc3339(),
        }))
    }

    #[oai(
        path = "/v1/security-servers/:server_id/authentication-certificates/:cert_id",
        method = "delete"
    )]
    pub async fn delete_server_auth_cert(
        &self,
        state: Data<&AppState>,
        _server_id: Path<i64>,
        cert_id: Path<i64>,
    ) -> poem::Result<Json<bool>> {
        let ok = state
            .db
            .delete_security_server_auth_cert(cert_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        Ok(Json(ok))
    }
}
