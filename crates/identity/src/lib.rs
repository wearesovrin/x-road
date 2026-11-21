use common::XRoadResult;
use identifiers::ClientId;
use serde::{Deserialize, Serialize};

/// Decentralized Identifier (DID) Document model matching W3C specifications and Sovrin DID methods.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidDocument {
    pub id: String,
    pub verification_method: Vec<VerificationMethod>,
    pub authentication: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    pub id: String,
    #[serde(rename = "type")]
    pub method_type: String,
    pub controller: String,
    pub public_key_multibase: Option<String>,
}

/// Verifiable Credential model for X-Road Dataspace access control.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiableCredential {
    pub context: Vec<String>,
    pub id: String,
    #[serde(rename = "type")]
    pub credential_type: Vec<String>,
    pub issuer: String,
    pub issuance_date: String,
    pub credential_subject: serde_json::Value,
}

/// Trait for resolving Sovrin / W3C DIDs and verifying credentials.
#[tonic::async_trait]
pub trait IdentityResolver: Send + Sync {
    /// Resolves a DID into a DID Document.
    async fn resolve_did(&self, did: &str) -> XRoadResult<Option<DidDocument>>;

    /// Maps an X-Road ClientId to an associated DID.
    async fn get_did_for_client(&self, client_id: &ClientId) -> XRoadResult<Option<String>>;
}
