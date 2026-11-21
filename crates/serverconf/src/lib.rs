use common::XRoadResult;
use identifiers::{ClientId, ServiceId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Service description representing either REST or SOAP services.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDescription {
    pub client_id: ClientId,
    pub url: String,
    pub service_type: ServiceType,
    pub services: Vec<ServiceId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceType {
    Rest,
    Soap,
    Wsdl,
    OpenApi,
}

/// Local Security Server configuration provider trait.
#[tonic::async_trait]
pub trait ServerConfProvider: Send + Sync {
    /// Checks if client is configured on this security server.
    async fn is_client_allowed(&self, client_id: &ClientId) -> XRoadResult<bool>;

    /// Verifies access permissions for client calling a service.
    async fn is_service_allowed(
        &self,
        client_id: &ClientId,
        service_id: &ServiceId,
    ) -> XRoadResult<bool>;

    /// Gets the backend URL for an internal service.
    async fn get_service_url(&self, service_id: &ServiceId) -> XRoadResult<Option<String>>;
}

/// In-memory mock server configuration for testing.
#[derive(Debug, Default, Clone)]
pub struct InMemoryServerConf {
    pub allowed_clients: HashSet<ClientId>,
    pub access_rights: HashMap<ServiceId, HashSet<ClientId>>,
    pub service_urls: HashMap<ServiceId, String>,
}

#[tonic::async_trait]
impl ServerConfProvider for InMemoryServerConf {
    async fn is_client_allowed(&self, client_id: &ClientId) -> XRoadResult<bool> {
        Ok(self.allowed_clients.contains(client_id))
    }

    async fn is_service_allowed(
        &self,
        client_id: &ClientId,
        service_id: &ServiceId,
    ) -> XRoadResult<bool> {
        if let Some(allowed) = self.access_rights.get(service_id) {
            Ok(allowed.contains(client_id))
        } else {
            Ok(false)
        }
    }

    async fn get_service_url(&self, service_id: &ServiceId) -> XRoadResult<Option<String>> {
        Ok(self.service_urls.get(service_id).cloned())
    }
}
