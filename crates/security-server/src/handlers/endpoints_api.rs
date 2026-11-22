use identifiers::ClientId;
use poem_openapi::{OpenApi, param::Path, payload::Json};
use sea_orm::EntityTrait;

use super::AdminApiState;
use super::dto::{AddEndpointReq, AddGroupMemberReq, EndpointDto, SimpleSuccessResponse};

pub struct EndpointsApi {
    pub state: AdminApiState,
}

#[OpenApi]
impl EndpointsApi {
    /// Creates an endpoint on a service.
    #[oai(path = "/v1/services/:id/endpoints", method = "post")]
    pub async fn add_endpoint(
        &self,
        id: Path<i64>,
        body: Json<AddEndpointReq>,
    ) -> poem::Result<Json<EndpointDto>> {
        let svc = self
            .state
            .server_conf
            .get_service_by_id(id.0)
            .await
            .map_err(poem::error::InternalServerError)?
            .ok_or_else(|| {
                poem::Error::from_string("Service not found", poem::http::StatusCode::NOT_FOUND)
            })?;

        let desc = self
            .state
            .server_conf
            .get_service_description(svc.service_description_id)
            .await
            .map_err(poem::error::InternalServerError)?
            .ok_or_else(|| {
                poem::Error::from_string(
                    "Service description not found",
                    poem::http::StatusCode::NOT_FOUND,
                )
            })?;

        let client = serverconf::entities::client::Entity::find_by_id(desc.client_id)
            .one(self.state.server_conf.raw_connection())
            .await
            .map_err(poem::error::InternalServerError)?
            .ok_or_else(|| {
                poem::Error::from_string("Client not found", poem::http::StatusCode::NOT_FOUND)
            })?;

        let client_id = if let Some(sub) = client.subsystem_code {
            ClientId::subsystem_unchecked(
                client.xroad_instance,
                client.member_class,
                client.member_code,
                sub,
            )
        } else {
            ClientId::member_unchecked(
                client.xroad_instance,
                client.member_class,
                client.member_code,
            )
        };

        let ep = self
            .state
            .server_conf
            .add_endpoint(&client_id, &svc.service_code, &body.method, &body.path)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(EndpointDto {
            id: ep.id,
            service_code: ep.service_code,
            method: ep.method,
            path: ep.path,
            generated: ep.generated,
        }))
    }

    /// Gets an endpoint by ID.
    #[oai(path = "/v1/endpoints/:id", method = "get")]
    pub async fn get_endpoint(&self, id: Path<i64>) -> poem::Result<Json<EndpointDto>> {
        let ep = self
            .state
            .server_conf
            .get_endpoint(id.0)
            .await
            .map_err(poem::error::InternalServerError)?
            .ok_or_else(|| {
                poem::Error::from_string("Endpoint not found", poem::http::StatusCode::NOT_FOUND)
            })?;

        Ok(Json(EndpointDto {
            id: ep.id,
            service_code: ep.service_code,
            method: ep.method,
            path: ep.path,
            generated: ep.generated,
        }))
    }

    /// Deletes an endpoint by ID.
    #[oai(path = "/v1/endpoints/:id", method = "delete")]
    pub async fn delete_endpoint(
        &self,
        id: Path<i64>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        self.state
            .server_conf
            .delete_endpoint(id.0)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Endpoint {} deleted", id.0),
        }))
    }

    /// Lists service clients granted access to an endpoint.
    #[oai(path = "/v1/endpoints/:id/service-clients", method = "get")]
    pub async fn list_endpoint_clients(&self, id: Path<i64>) -> poem::Result<Json<Vec<String>>> {
        let clients = self
            .state
            .server_conf
            .list_endpoint_access_rights(id.0)
            .await
            .unwrap_or_default();
        Ok(Json(clients))
    }

    /// Grants access to an endpoint for a subject.
    #[oai(path = "/v1/endpoints/:id/service-clients", method = "post")]
    pub async fn grant_endpoint_access(
        &self,
        id: Path<i64>,
        body: Json<AddGroupMemberReq>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        self.state
            .server_conf
            .grant_endpoint_access(id.0, &body.member_id)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Access granted to endpoint {} for {}", id.0, body.member_id),
        }))
    }

    /// Revokes access to an endpoint for a subject.
    #[oai(path = "/v1/endpoints/:id/service-clients/delete", method = "post")]
    pub async fn revoke_endpoint_access(
        &self,
        id: Path<i64>,
        body: Json<AddGroupMemberReq>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        self.state
            .server_conf
            .revoke_endpoint_access(id.0, &body.member_id)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!(
                "Access revoked for endpoint {} from {}",
                id.0, body.member_id
            ),
        }))
    }
}
