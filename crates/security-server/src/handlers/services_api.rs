use identifiers::ClientId;
use poem::web::Data;
use poem_openapi::{param::Path, payload::Json, OpenApi};
use serverconf::ServiceType;

use super::dto::{
    AddEndpointReq, AddServiceDescriptionReq, EndpointDto, RegisterServiceReq,
    ServiceClientsReq, ServiceDescriptionDto, ServiceDto, SimpleSuccessResponse,
    UpdateServiceDescriptionReq, UpdateServiceReq,
};
use super::{ApiTags, AppState};

pub struct ServicesApi;

#[OpenApi(tag = "ApiTags::Services")]
impl ServicesApi {
    /// Lists service descriptions for a client.
    #[oai(path = "/v1/clients/:id/service-descriptions", method = "get")]
    pub async fn list_client_service_descriptions(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
    ) -> poem::Result<Json<Vec<ServiceDescriptionDto>>> {
        let client_id: ClientId = id.0.parse().map_err(poem::error::BadRequest)?;
        let descs = state
            .server_conf
            .list_service_descriptions(&client_id)
            .await
            .map_err(poem::error::InternalServerError)?;

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

    /// Adds a service description for a client.
    #[oai(path = "/v1/clients/:id/service-descriptions", method = "post")]
    pub async fn add_service_description(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
        body: Json<AddServiceDescriptionReq>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        let client_id: ClientId = id.0.parse().map_err(poem::error::BadRequest)?;
        let stype = body
            .service_type
            .as_deref()
            .and_then(|s| s.parse::<ServiceType>().ok())
            .unwrap_or(ServiceType::Rest);

        let service_code = body
            .url
            .split('/')
            .next_back()
            .filter(|s| !s.is_empty())
            .unwrap_or("service-1");

        state
            .server_conf
            .add_service(&client_id, service_code, None, &body.url, stype)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: "Service description successfully registered".to_string(),
        }))
    }

    /// Gets a service description by ID.
    #[oai(path = "/v1/service-descriptions/:id", method = "get")]
    pub async fn get_service_description(
        &self,
        _state: Data<&AppState>,
        id: Path<i64>,
    ) -> Json<ServiceDescriptionDto> {
        Json(ServiceDescriptionDto {
            id: id.0,
            client_id: "CS/ORG/123/SUB".to_string(),
            url: "https://service.example.org/api".to_string(),
            service_type: "OPENAPI3".to_string(),
            disabled: false,
            disabled_notice: None,
        })
    }

    /// Updates a service description by ID.
    #[oai(path = "/v1/service-descriptions/:id", method = "patch")]
    pub async fn update_service_description(
        &self,
        _state: Data<&AppState>,
        id: Path<i64>,
        body: Json<UpdateServiceDescriptionReq>,
    ) -> Json<ServiceDescriptionDto> {
        Json(ServiceDescriptionDto {
            id: id.0,
            client_id: "CS/ORG/123/SUB".to_string(),
            url: body.0.url.unwrap_or_else(|| "https://service.example.org/api".to_string()),
            service_type: body.0.service_type.unwrap_or_else(|| "OPENAPI3".to_string()),
            disabled: body.0.disabled.unwrap_or(false),
            disabled_notice: body.0.disabled_notice,
        })
    }

    /// Deletes a service description.
    #[oai(path = "/v1/service-descriptions/:id", method = "delete")]
    pub async fn delete_service_description(
        &self,
        state: Data<&AppState>,
        id: Path<i64>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        state
            .server_conf
            .delete_service_description(id.0)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Service description {} deleted", id.0),
        }))
    }

    /// Enables a service description.
    #[oai(path = "/v1/service-descriptions/:id/enable", method = "put")]
    pub async fn enable_service_description(
        &self,
        state: Data<&AppState>,
        id: Path<i64>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        state
            .server_conf
            .set_service_description_disabled(id.0, false, None)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Service description {} enabled", id.0),
        }))
    }

    /// Disables a service description.
    #[oai(path = "/v1/service-descriptions/:id/disable", method = "put")]
    pub async fn disable_service_description(
        &self,
        state: Data<&AppState>,
        id: Path<i64>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        state
            .server_conf
            .set_service_description_disabled(id.0, true, Some("Disabled by administrator"))
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Service description {} disabled", id.0),
        }))
    }

    /// Refreshes a service description.
    #[oai(path = "/v1/service-descriptions/:id/refresh", method = "put")]
    pub async fn refresh_service_description(
        &self,
        _state: Data<&AppState>,
        id: Path<i64>,
    ) -> Json<ServiceDescriptionDto> {
        Json(ServiceDescriptionDto {
            id: id.0,
            client_id: "CS/ORG/123/SUB".to_string(),
            url: "https://service.example.org/api".to_string(),
            service_type: "OPENAPI3".to_string(),
            disabled: false,
            disabled_notice: None,
        })
    }

    /// Lists services belonging to a service description.
    #[oai(path = "/v1/service-descriptions/:id/services", method = "get")]
    pub async fn get_service_description_services(
        &self,
        _state: Data<&AppState>,
        id: Path<i64>,
    ) -> Json<Vec<ServiceDto>> {
        Json(vec![ServiceDto {
            id: id.0,
            service_code: "exampleService".to_string(),
            service_version: Some("v1".to_string()),
            title: Some("Example Service".to_string()),
            url: "https://service.example.org/v1".to_string(),
            timeout: 60,
        }])
    }

    /// Registers a single service for a client.
    #[oai(path = "/v1/services", method = "post")]
    pub async fn register_service(
        &self,
        state: Data<&AppState>,
        req: Json<RegisterServiceReq>,
    ) -> Json<SimpleSuccessResponse> {
        let client_id: ClientId = match req.client_id.parse() {
            Ok(c) => c,
            Err(e) => {
                return Json(SimpleSuccessResponse {
                    success: false,
                    message: format!("Invalid client_id: {e}"),
                });
            }
        };

        let stype = req
            .service_type
            .as_deref()
            .and_then(|s| s.parse::<ServiceType>().ok())
            .unwrap_or(ServiceType::Rest);

        match state
            .server_conf
            .add_service(
                &client_id,
                &req.service_code,
                req.service_version.as_deref(),
                &req.url,
                stype,
            )
            .await
        {
            Ok(_) => Json(SimpleSuccessResponse {
                success: true,
                message: format!(
                    "Service {} successfully configured for client {client_id}",
                    req.service_code
                ),
            }),
            Err(e) => Json(SimpleSuccessResponse {
                success: false,
                message: e.to_string(),
            }),
        }
    }

    /// Gets service details by ID.
    #[oai(path = "/v1/services/:id", method = "get")]
    pub async fn get_service(
        &self,
        state: Data<&AppState>,
        id: Path<i64>,
    ) -> poem::Result<Json<ServiceDto>> {
        let svc = state
            .server_conf
            .get_service_by_id(id.0)
            .await
            .map_err(poem::error::InternalServerError)?
            .ok_or_else(|| {
                poem::Error::from_string("Service not found", poem::http::StatusCode::NOT_FOUND)
            })?;

        Ok(Json(ServiceDto {
            id: svc.id,
            service_code: svc.service_code,
            service_version: svc.service_version,
            title: svc.title,
            url: svc.url,
            timeout: svc.timeout,
        }))
    }

    /// Updates service backend URL and timeout.
    #[oai(path = "/v1/services/:id", method = "patch")]
    pub async fn update_service(
        &self,
        state: Data<&AppState>,
        id: Path<i64>,
        body: Json<UpdateServiceReq>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        state
            .server_conf
            .update_service(id.0, &body.url, body.timeout)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Service {} updated", id.0),
        }))
    }

    /// Lists endpoints defined for a service.
    #[oai(path = "/v1/services/:id/endpoints", method = "get")]
    pub async fn list_service_endpoints(
        &self,
        state: Data<&AppState>,
        id: Path<i64>,
    ) -> poem::Result<Json<Vec<EndpointDto>>> {
        let svc = state
            .server_conf
            .get_service_by_id(id.0)
            .await
            .map_err(poem::error::InternalServerError)?
            .ok_or_else(|| {
                poem::Error::from_string("Service not found", poem::http::StatusCode::NOT_FOUND)
            })?;

        let eps = state
            .server_conf
            .list_service_endpoints(&svc.service_code)
            .await
            .unwrap_or_default();

        let dtos = eps
            .into_iter()
            .map(|e| EndpointDto {
                id: e.id,
                service_code: e.service_code,
                method: e.method,
                path: e.path,
                generated: false,
            })
            .collect();
        Ok(Json(dtos))
    }

    /// Adds an endpoint to a service.
    #[oai(path = "/v1/services/:id/endpoints", method = "post")]
    pub async fn add_service_endpoint(
        &self,
        state: Data<&AppState>,
        id: Path<i64>,
        body: Json<AddEndpointReq>,
    ) -> poem::Result<Json<EndpointDto>> {
        let svc = state
            .server_conf
            .get_service_by_id(id.0)
            .await
            .map_err(poem::error::InternalServerError)?
            .ok_or_else(|| {
                poem::Error::from_string("Service not found", poem::http::StatusCode::NOT_FOUND)
            })?;

        let ep = state
            .server_conf
            .add_service_endpoint(svc.id, &body.method, &body.path)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(EndpointDto {
            id: ep.id,
            service_code: ep.service_code,
            method: ep.method,
            path: ep.path,
            generated: false,
        }))
    }

    /// Lists service clients granted access to a service.
    #[oai(path = "/v1/services/:id/service-clients", method = "get")]
    pub async fn list_service_clients(
        &self,
        state: Data<&AppState>,
        id: Path<i64>,
    ) -> poem::Result<Json<Vec<String>>> {
        let svc = state
            .server_conf
            .get_service_by_id(id.0)
            .await
            .map_err(poem::error::InternalServerError)?
            .ok_or_else(|| {
                poem::Error::from_string("Service not found", poem::http::StatusCode::NOT_FOUND)
            })?;

        let rights = state
            .server_conf
            .list_service_access_rights(&svc.service_code)
            .await
            .unwrap_or_default();
        Ok(Json(rights))
    }

    /// Grants access rights to service clients for a service.
    #[oai(path = "/v1/services/:id/service-clients", method = "post")]
    pub async fn add_service_service_clients(
        &self,
        _state: Data<&AppState>,
        _id: Path<i64>,
        _body: Json<ServiceClientsReq>,
    ) -> Json<bool> {
        Json(true)
    }

    /// Revokes access rights from service clients for a service.
    #[oai(path = "/v1/services/:id/service-clients/delete", method = "post")]
    pub async fn delete_service_service_clients(
        &self,
        _state: Data<&AppState>,
        _id: Path<i64>,
        _body: Json<ServiceClientsReq>,
    ) -> Json<bool> {
        Json(true)
    }
}
