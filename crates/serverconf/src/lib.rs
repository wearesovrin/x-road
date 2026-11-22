pub mod db;
pub mod entities;
pub mod migration;

use common::XRoadResult;
use identifiers::{ClientId, ServiceId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use strum::{AsRefStr, Display, EnumString, IntoStaticStr};

pub use db::DatabaseServerConf;
pub use migration::Migrator;

/// Service description representing either REST or SOAP services.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDescription {
    pub client_id: ClientId,
    pub url: String,
    pub service_type: ServiceType,
    pub services: Vec<ServiceId>,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    AsRefStr,
    IntoStaticStr,
)]
#[strum(ascii_case_insensitive, serialize_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
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

    /// Lists all services configured for a given client (producer).
    async fn list_services(&self, client_id: &ClientId) -> XRoadResult<Vec<ServiceId>>;

    /// Lists all services allowed for a consumer calling a producer.
    async fn get_allowed_services(
        &self,
        producer: &ClientId,
        consumer: &ClientId,
    ) -> XRoadResult<Vec<ServiceId>>;
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

    async fn list_services(&self, client_id: &ClientId) -> XRoadResult<Vec<ServiceId>> {
        let list: Vec<ServiceId> = self
            .service_urls
            .keys()
            .filter(|s| s.client == *client_id)
            .cloned()
            .collect();
        Ok(list)
    }

    async fn get_allowed_services(
        &self,
        producer: &ClientId,
        consumer: &ClientId,
    ) -> XRoadResult<Vec<ServiceId>> {
        let mut allowed = Vec::new();
        for (svc, clients) in &self.access_rights {
            if svc.client == *producer && clients.contains(consumer) {
                allowed.push(svc.clone());
            }
        }
        Ok(allowed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_server_conf_in_memory() {
        let db_conf = DatabaseServerConf::connect_in_memory()
            .await
            .expect("Failed to initialize in-memory database and run migrations");

        let client = ClientId::subsystem_unchecked("DEV", "COM", "12345678", "SUB1");
        let foreign_client = ClientId::subsystem_unchecked("DEV", "COM", "87654321", "SUB2");

        // 1. Initially client is not allowed
        assert!(!db_conf.is_client_allowed(&client).await.unwrap());

        // 2. Register client
        db_conf.add_client(&client, false).await.unwrap();
        assert!(db_conf.is_client_allowed(&client).await.unwrap());

        // 3. Register service
        db_conf
            .add_service(
                &client,
                "getUserData",
                Some("v1"),
                "http://localhost:8080/user",
                ServiceType::Rest,
            )
            .await
            .unwrap();

        let service = ServiceId::new(client.clone(), "getUserData", Some("v1")).unwrap();
        let service_url = db_conf.get_service_url(&service).await.unwrap();
        assert_eq!(service_url, Some("http://localhost:8080/user".to_string()));

        // 4. Access rights
        assert!(
            !db_conf
                .is_service_allowed(&foreign_client, &service)
                .await
                .unwrap()
        );
        db_conf
            .grant_access(&service, &foreign_client)
            .await
            .unwrap();
        assert!(
            db_conf
                .is_service_allowed(&foreign_client, &service)
                .await
                .unwrap()
        );

        // Revoke access
        db_conf
            .revoke_access(&service, &foreign_client)
            .await
            .unwrap();
        assert!(
            !db_conf
                .is_service_allowed(&foreign_client, &service)
                .await
                .unwrap()
        );

        // 5. API Key validation
        let key_id = db_conf
            .create_api_key(
                "secret-token-xyz",
                Some("Test Key"),
                &["XROAD_SECURITY_OFFICER"],
            )
            .await
            .unwrap();
        assert!(!key_id.is_empty());

        let roles = db_conf.validate_api_key("secret-token-xyz").await.unwrap();
        assert_eq!(roles, Some(vec!["XROAD_SECURITY_OFFICER".to_string()]));

        let invalid = db_conf.validate_api_key("non-existent").await.unwrap();
        assert_eq!(invalid, None);

        // 6. Certificate storage
        let dummy_cert = vec![0x30, 0x82, 0x01, 0x02];
        db_conf
            .store_certificate(&client, &dummy_cert)
            .await
            .unwrap();
        let certs = db_conf.get_certificates(&client).await.unwrap();
        assert_eq!(certs.len(), 1);
        assert_eq!(certs[0], dummy_cert);
    }
}
