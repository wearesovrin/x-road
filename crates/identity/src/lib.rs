use chrono::{DateTime, Utc};
use common::{XRoadError, XRoadResult};
use identifiers::ClientId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Converts an X-Road `ClientId` into a standard decentralized identifier (DID).
pub fn client_id_to_did(client_id: &ClientId) -> String {
    if let Some(ref sub) = client_id.subsystem_code {
        format!(
            "did:xroad:{}:{}:{}:{}",
            client_id.xroad_instance, client_id.member_class, client_id.member_code, sub
        )
    } else {
        format!(
            "did:xroad:{}:{}:{}",
            client_id.xroad_instance, client_id.member_class, client_id.member_code
        )
    }
}

/// Parses a `did:xroad:...` string into an X-Road `ClientId`.
pub fn did_to_client_id(did: &str) -> XRoadResult<ClientId> {
    let parts: Vec<&str> = did.split(':').collect();
    if parts.len() < 5 || parts[0] != "did" || parts[1] != "xroad" {
        return Err(XRoadError::IdentifierParse(format!(
            "Invalid X-Road DID format: {did}"
        )));
    }

    let instance = parts[2];
    let class = parts[3];
    let code = parts[4];

    if parts.len() >= 6 {
        let subsystem = parts[5];
        ClientId::subsystem(instance, class, code, subsystem)
    } else {
        ClientId::member(instance, class, code)
    }
}

/// Decentralized Identifier (DID) Document model matching W3C specifications and Sovrin DID methods.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DidDocument {
    pub id: String,
    pub verification_method: Vec<VerificationMethod>,
    pub authentication: Vec<String>,
}

impl DidDocument {
    pub fn new(
        id: impl Into<String>,
        verification_method: Vec<VerificationMethod>,
        authentication: Vec<String>,
    ) -> Self {
        Self {
            id: id.into(),
            verification_method,
            authentication,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationMethod {
    pub id: String,
    #[serde(rename = "type")]
    pub method_type: String,
    pub controller: String,
    pub public_key_multibase: Option<String>,
}

/// Verifiable Credential model for X-Road Dataspace access control.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerifiableCredential {
    pub context: Vec<String>,
    pub id: String,
    #[serde(rename = "type")]
    pub credential_type: Vec<String>,
    pub issuer: String,
    pub issuance_date: String,
    pub expiration_date: Option<String>,
    pub credential_subject: serde_json::Value,
}

impl VerifiableCredential {
    /// Validates the structure and temporal validity of the Verifiable Credential.
    pub fn validate(&self) -> XRoadResult<()> {
        if !self
            .context
            .iter()
            .any(|c| c == "https://www.w3.org/2018/credentials/v1")
        {
            return Err(XRoadError::AccessDenied(
                "Missing required W3C credential context".to_string(),
            ));
        }

        if !self
            .credential_type
            .iter()
            .any(|t| t == "VerifiableCredential")
        {
            return Err(XRoadError::AccessDenied(
                "Missing VerifiableCredential type".to_string(),
            ));
        }

        if let Some(ref exp_str) = self.expiration_date {
            if let Ok(exp_time) = exp_str.parse::<DateTime<Utc>>() {
                if Utc::now() > exp_time {
                    return Err(XRoadError::AccessDenied(format!(
                        "Verifiable credential expired at {exp_str}"
                    )));
                }
            }
        }

        Ok(())
    }
}

/// Trait for resolving Sovrin / W3C DIDs and verifying credentials.
#[tonic::async_trait]
pub trait IdentityResolver: Send + Sync {
    /// Resolves a DID into a DID Document.
    async fn resolve_did(&self, did: &str) -> XRoadResult<Option<DidDocument>>;

    /// Maps an X-Road ClientId to an associated DID.
    async fn get_did_for_client(&self, client_id: &ClientId) -> XRoadResult<Option<String>>;
}

/// In-memory identity resolver for caching and mapping DIDs and documents.
#[derive(Debug, Clone, Default)]
pub struct InMemoryIdentityResolver {
    dids: Arc<RwLock<HashMap<String, DidDocument>>>,
    client_mappings: Arc<RwLock<HashMap<ClientId, String>>>,
}

impl InMemoryIdentityResolver {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a DID Document in the resolver.
    pub async fn register_did_document(&self, doc: DidDocument) {
        self.dids.write().await.insert(doc.id.clone(), doc);
    }

    /// Associates an X-Road ClientId with a DID.
    pub async fn bind_client_to_did(&self, client_id: ClientId, did: impl Into<String>) {
        self.client_mappings
            .write()
            .await
            .insert(client_id, did.into());
    }
}

#[tonic::async_trait]
impl IdentityResolver for InMemoryIdentityResolver {
    async fn resolve_did(&self, did: &str) -> XRoadResult<Option<DidDocument>> {
        Ok(self.dids.read().await.get(did).cloned())
    }

    async fn get_did_for_client(&self, client_id: &ClientId) -> XRoadResult<Option<String>> {
        // Return explicitly mapped DID or default deterministic xroad DID
        if let Some(did) = self.client_mappings.read().await.get(client_id) {
            return Ok(Some(did.clone()));
        }
        Ok(Some(client_id_to_did(client_id)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_did_formatting_and_parsing() {
        let member = ClientId::member("DEV", "COM", "12345678").unwrap();
        let member_did = client_id_to_did(&member);
        assert_eq!(member_did, "did:xroad:DEV:COM:12345678");

        let parsed_member = did_to_client_id(&member_did).unwrap();
        assert_eq!(parsed_member, member);

        let sub = ClientId::subsystem("DEV", "COM", "12345678", "MANAGEMENT").unwrap();
        let sub_did = client_id_to_did(&sub);
        assert_eq!(sub_did, "did:xroad:DEV:COM:12345678:MANAGEMENT");

        let parsed_sub = did_to_client_id(&sub_did).unwrap();
        assert_eq!(parsed_sub, sub);
    }

    #[tokio::test]
    async fn test_in_memory_identity_resolver() {
        let resolver = InMemoryIdentityResolver::new();
        let client = ClientId::subsystem("DEV", "COM", "12345678", "FINANCE").unwrap();

        let doc = DidDocument::new(
            "did:sov:123456789abcdef",
            vec![VerificationMethod {
                id: "did:sov:123456789abcdef#key-1".to_string(),
                method_type: "Ed25519VerificationKey2020".to_string(),
                controller: "did:sov:123456789abcdef".to_string(),
                public_key_multibase: Some("z6Mkm...".to_string()),
            }],
            vec!["did:sov:123456789abcdef#key-1".to_string()],
        );

        resolver.register_did_document(doc.clone()).await;
        resolver.bind_client_to_did(client.clone(), "did:sov:123456789abcdef").await;

        let resolved_did = resolver.get_did_for_client(&client).await.unwrap();
        assert_eq!(resolved_did, Some("did:sov:123456789abcdef".to_string()));

        let resolved_doc = resolver.resolve_did("did:sov:123456789abcdef").await.unwrap();
        assert_eq!(resolved_doc, Some(doc));
    }

    #[test]
    fn test_verifiable_credential_validation() {
        let vc = VerifiableCredential {
            context: vec!["https://www.w3.org/2018/credentials/v1".to_string()],
            id: "urn:uuid:abc-123".to_string(),
            credential_type: vec!["VerifiableCredential".to_string(), "DataspaceParticipantCredential".to_string()],
            issuer: "did:xroad:DEV:GOV:AUTHORITY".to_string(),
            issuance_date: "2026-01-01T00:00:00Z".to_string(),
            expiration_date: Some("2030-01-01T00:00:00Z".to_string()),
            credential_subject: serde_json::json!({
                "id": "did:xroad:DEV:COM:12345678:FINANCE",
                "role": "DataspaceConsumer"
            }),
        };

        assert!(vc.validate().is_ok());

        // Expired credential
        let expired_vc = VerifiableCredential {
            context: vec!["https://www.w3.org/2018/credentials/v1".to_string()],
            id: "urn:uuid:abc-expired".to_string(),
            credential_type: vec!["VerifiableCredential".to_string()],
            issuer: "did:xroad:DEV:GOV:AUTHORITY".to_string(),
            issuance_date: "2020-01-01T00:00:00Z".to_string(),
            expiration_date: Some("2021-01-01T00:00:00Z".to_string()),
            credential_subject: serde_json::json!({}),
        };

        assert!(expired_vc.validate().is_err());
    }
}
