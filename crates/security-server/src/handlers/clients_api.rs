use base64::Engine;
use identifiers::ClientId;
use poem_openapi::{OpenApi, param::Path, payload::Json};

use super::AdminApiState;
use super::dto::{ClientDto, RegisterClientReq, SimpleSuccessResponse, StoreCertReq};

pub struct ClientsApi {
    pub state: AdminApiState,
}

#[OpenApi]
impl ClientsApi {
    /// Lists all registered clients.
    #[oai(path = "/v1/clients", method = "get")]
    pub async fn list_clients(&self) -> Json<Vec<ClientDto>> {
        let clients = self
            .state
            .server_conf
            .list_clients()
            .await
            .unwrap_or_default();
        let mut list = Vec::new();
        for c in clients {
            if let Ok(Some(m)) = self.state.server_conf.get_client(&c).await {
                list.push(ClientDto {
                    id: c.to_string(),
                    instance: m.xroad_instance,
                    member_class: m.member_class,
                    member_code: m.member_code,
                    subsystem_code: m.subsystem_code,
                    status: m.client_status,
                    is_owner: m.is_authentication,
                    is_auth: m.is_authentication,
                });
            }
        }
        Json(list)
    }

    /// Registers a client on this Security Server.
    #[oai(path = "/v1/clients", method = "post")]
    pub async fn register_client(
        &self,
        req: Json<RegisterClientReq>,
    ) -> Json<SimpleSuccessResponse> {
        let r = req.0;
        let client_id = if let Some(ref sub) = r.subsystem_code {
            ClientId::subsystem_unchecked(&r.instance, &r.member_class, &r.member_code, sub)
        } else {
            ClientId::member_unchecked(&r.instance, &r.member_class, &r.member_code)
        };

        match self
            .state
            .server_conf
            .add_client(&client_id, r.is_authentication.unwrap_or(false))
            .await
        {
            Ok(_) => Json(SimpleSuccessResponse {
                success: true,
                message: format!("Client {client_id} successfully registered"),
            }),
            Err(e) => Json(SimpleSuccessResponse {
                success: false,
                message: e.to_string(),
            }),
        }
    }

    /// Gets details of a single registered client.
    #[oai(path = "/v1/clients/:id", method = "get")]
    pub async fn get_client(&self, id: Path<String>) -> poem::Result<Json<ClientDto>> {
        let client_id: ClientId = id.0.parse().map_err(poem::error::BadRequest)?;
        let model = self
            .state
            .server_conf
            .get_client(&client_id)
            .await
            .map_err(poem::error::InternalServerError)?
            .ok_or_else(|| {
                poem::Error::from_string("Client not found", poem::http::StatusCode::NOT_FOUND)
            })?;

        Ok(Json(ClientDto {
            id: client_id.to_string(),
            instance: model.xroad_instance,
            member_class: model.member_class,
            member_code: model.member_code,
            subsystem_code: model.subsystem_code,
            status: model.client_status,
            is_owner: model.is_authentication,
            is_auth: model.is_authentication,
        }))
    }

    /// Deletes a client from the Security Server.
    #[oai(path = "/v1/clients/:id", method = "delete")]
    pub async fn delete_client(
        &self,
        id: Path<String>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        let client_id: ClientId = id.0.parse().map_err(poem::error::BadRequest)?;
        self.state
            .server_conf
            .delete_client(&client_id)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Client {client_id} successfully deleted"),
        }))
    }

    /// Enables a client.
    #[oai(path = "/v1/clients/:id/enable", method = "put")]
    pub async fn enable_client(
        &self,
        id: Path<String>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        let client_id: ClientId = id.0.parse().map_err(poem::error::BadRequest)?;
        self.state
            .server_conf
            .set_client_status(&client_id, "REGISTERED")
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Client {client_id} enabled"),
        }))
    }

    /// Disables a client.
    #[oai(path = "/v1/clients/:id/disable", method = "put")]
    pub async fn disable_client(
        &self,
        id: Path<String>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        let client_id: ClientId = id.0.parse().map_err(poem::error::BadRequest)?;
        self.state
            .server_conf
            .set_client_status(&client_id, "DISABLED")
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Client {client_id} disabled"),
        }))
    }

    /// Designates a client as the Security Server owner.
    #[oai(path = "/v1/clients/:id/make-owner", method = "put")]
    pub async fn make_owner(&self, id: Path<String>) -> poem::Result<Json<SimpleSuccessResponse>> {
        let client_id: ClientId = id.0.parse().map_err(poem::error::BadRequest)?;
        self.state
            .server_conf
            .make_owner(&client_id)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Client {client_id} designated as Security Server owner"),
        }))
    }

    /// Lists TLS certificates registered for a client.
    #[oai(path = "/v1/clients/:id/tls-certificates", method = "get")]
    pub async fn list_client_tls_certificates(
        &self,
        id: Path<String>,
    ) -> poem::Result<Json<Vec<String>>> {
        let client_id: ClientId = id.0.parse().map_err(poem::error::BadRequest)?;
        let certs = self
            .state
            .server_conf
            .get_certificates(&client_id)
            .await
            .map_err(poem::error::InternalServerError)?;

        let b64: Vec<String> = certs
            .into_iter()
            .map(|c| base64::engine::general_purpose::STANDARD.encode(c))
            .collect();
        Ok(Json(b64))
    }

    /// Uploads a TLS certificate for a client.
    #[oai(path = "/v1/clients/:id/tls-certificates", method = "post")]
    pub async fn upload_client_tls_certificate(
        &self,
        id: Path<String>,
        body: Json<StoreCertReq>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        let client_id: ClientId = id.0.parse().map_err(poem::error::BadRequest)?;
        let der = base64::engine::general_purpose::STANDARD
            .decode(&body.cert_der_base64)
            .map_err(poem::error::BadRequest)?;

        self.state
            .server_conf
            .store_certificate(&client_id, &der)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: "TLS certificate successfully added".to_string(),
        }))
    }

    /// Stores a client certificate in DER format (Base64 encoded).
    #[oai(path = "/v1/certificates", method = "post")]
    pub async fn store_certificate(&self, req: Json<StoreCertReq>) -> Json<SimpleSuccessResponse> {
        let client_id: ClientId = match req.client_id.parse() {
            Ok(c) => c,
            Err(e) => {
                return Json(SimpleSuccessResponse {
                    success: false,
                    message: format!("Invalid client_id: {e}"),
                });
            }
        };

        let der_bytes = match base64::engine::general_purpose::STANDARD.decode(&req.cert_der_base64)
        {
            Ok(b) => b,
            Err(e) => {
                return Json(SimpleSuccessResponse {
                    success: false,
                    message: format!("Invalid base64 encoding: {e}"),
                });
            }
        };

        match self
            .state
            .server_conf
            .store_certificate(&client_id, &der_bytes)
            .await
        {
            Ok(_) => Json(SimpleSuccessResponse {
                success: true,
                message: format!("Certificate stored for {client_id}"),
            }),
            Err(e) => Json(SimpleSuccessResponse {
                success: false,
                message: e.to_string(),
            }),
        }
    }
}
