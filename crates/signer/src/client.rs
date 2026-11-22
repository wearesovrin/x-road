use crate::{KeyUsageInfo, SignerServiceImpl};
use common::{XRoadError, XRoadResult};
use identifiers::ClientId;
use proto::certificate_service_client::CertificateServiceClient;
use proto::key_service_client::KeyServiceClient;
use proto::ocsp_service_client::OcspServiceClient;
use proto::sign_service_client::SignServiceClient;
use proto::token_service_client::TokenServiceClient;
use proto::*;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tonic::transport::Channel;

/// High-level client for communicating with the X-Road Signer.
/// Can operate either as a remote gRPC client or in-process (in-memory) for embedded deployments.
#[derive(Clone)]
pub enum SignerClient {
    Remote {
        key_client: KeyServiceClient<Channel>,
        sign_client: SignServiceClient<Channel>,
        token_client: TokenServiceClient<Channel>,
        cert_client: CertificateServiceClient<Channel>,
        ocsp_client: OcspServiceClient<Channel>,
    },
    InMemory {
        service: Arc<SignerServiceImpl>,
    },
}

impl SignerClient {
    /// Connects to a remote Signer gRPC daemon.
    pub async fn connect(endpoint: impl Into<String>) -> XRoadResult<Self> {
        let ep_str = endpoint.into();
        let channel = Channel::from_shared(ep_str)
            .map_err(|e| XRoadError::Configuration(e.to_string()))?
            .connect()
            .await
            .map_err(|e| XRoadError::Network(format!("Failed to connect to Signer daemon: {e}")))?;

        Ok(Self::Remote {
            key_client: KeyServiceClient::new(channel.clone()),
            sign_client: SignServiceClient::new(channel.clone()),
            token_client: TokenServiceClient::new(channel.clone()),
            cert_client: CertificateServiceClient::new(channel.clone()),
            ocsp_client: OcspServiceClient::new(channel),
        })
    }

    /// Wraps an in-memory `SignerServiceImpl` for zero-overhead in-process operations.
    pub fn in_memory(service: Arc<SignerServiceImpl>) -> Self {
        Self::InMemory { service }
    }

    /// Creates a new default in-memory Signer client.
    pub fn new_in_memory() -> Self {
        Self::InMemory {
            service: Arc::new(SignerServiceImpl::new()),
        }
    }

    /// Digitally signs a pre-computed digest with the given key.
    pub async fn sign(&self, key_id: &str, digest: &[u8]) -> XRoadResult<Vec<u8>> {
        match self {
            Self::Remote { sign_client, .. } => {
                let mut client = sign_client.clone();
                let resp = client
                    .sign(SignReq {
                        key_id: key_id.to_string(),
                        digest: digest.to_vec(),
                        signature_algorithm_id: "SHA256withECDSA".to_string(),
                    })
                    .await
                    .map_err(|e| XRoadError::Crypto(format!("Signer RPC failed: {e}")))?;
                Ok(resp.into_inner().signature)
            }
            Self::InMemory { service } => service.sign_digest(key_id, digest).await,
        }
    }

    /// Retrieves signing key information for a member.
    pub async fn get_member_signing_info(
        &self,
        member_id: &ClientId,
    ) -> XRoadResult<Option<(String, Option<CertificateInfoProto>, String)>> {
        match self {
            Self::Remote { token_client, .. } => {
                let mut client = token_client.clone();
                let member_proto = ClientIdProto {
                    xroad_instance: member_id.xroad_instance.clone(),
                    member_class: member_id.member_class.clone(),
                    member_code: member_id.member_code.clone(),
                    subsystem_code: member_id.subsystem_code.clone(),
                    object_type: XRoadObjectType::Member as i32,
                };
                let resp = client
                    .get_member_signing_info(GetMemberSigningInfoReq {
                        member_id: Some(member_proto),
                    })
                    .await
                    .map_err(|e| XRoadError::Crypto(format!("Signer RPC failed: {e}")))?;
                let inner = resp.into_inner();
                if inner.key_id.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some((inner.key_id, inner.cert, inner.sign_mechanism_name)))
                }
            }
            Self::InMemory { service } => {
                let keys = service.keys.read().await;
                for k in keys.values() {
                    if k.usage == KeyUsageInfo::Signing as i32 {
                        return Ok(Some((
                            k.id.clone(),
                            k.certs.first().cloned(),
                            k.sign_mechanism.clone(),
                        )));
                    }
                }
                Ok(None)
            }
        }
    }

    /// Retrieves authentication key for the Security Server.
    pub async fn get_auth_key(&self, server_id: &str) -> XRoadResult<Option<AuthKeyProto>> {
        match self {
            Self::Remote { key_client, .. } => {
                let mut client = key_client.clone();
                let resp = client
                    .get_auth_key(GetAuthKeyReq {
                        security_server: Some(SecurityServerIdProto {
                            xroad_instance: "DEV".to_string(),
                            member_class: "COM".to_string(),
                            member_code: "12345678".to_string(),
                            server_code: server_id.to_string(),
                            object_type: XRoadObjectType::Server as i32,
                        }),
                    })
                    .await
                    .map_err(|e| XRoadError::Crypto(format!("Signer RPC failed: {e}")))?;
                Ok(Some(resp.into_inner()))
            }
            Self::InMemory { service } => {
                let keys = service.keys.read().await;
                for k in keys.values() {
                    if k.usage == KeyUsageInfo::Authentication as i32 {
                        return Ok(Some(AuthKeyProto {
                            alias: k.id.clone(),
                            key_store: k.key_der.clone(),
                            password: String::new(),
                            cert: k.certs.first().cloned(),
                        }));
                    }
                }
                Ok(None)
            }
        }
    }

    /// Retrieves key info by key ID.
    pub async fn get_key(&self, key_id: &str) -> XRoadResult<Option<KeyInfoProto>> {
        match self {
            Self::Remote { token_client, .. } => {
                let mut client = token_client.clone();
                match client
                    .get_token_by_key(GetTokenByKeyIdReq {
                        key_id: key_id.to_string(),
                    })
                    .await
                {
                    Ok(resp) => {
                        let token = resp.into_inner();
                        Ok(token.key_info.into_iter().find(|k| k.id == key_id))
                    }
                    Err(s) if s.code() == tonic::Code::NotFound => Ok(None),
                    Err(e) => Err(XRoadError::Crypto(format!("Signer RPC failed: {e}"))),
                }
            }
            Self::InMemory { service } => Ok(service.get_key(key_id).await),
        }
    }

    /// Generates a new cryptographic key on the given token.
    pub async fn generate_key(
        &self,
        token_id: &str,
        key_label: &str,
        usage: KeyUsageInfo,
    ) -> XRoadResult<KeyInfoProto> {
        match self {
            Self::Remote { key_client, .. } => {
                let mut client = key_client.clone();
                let resp = client
                    .generate_key(GenerateKeyReq {
                        token_id: token_id.to_string(),
                        key_label: key_label.to_string(),
                        algorithm: Algorithm::Ec as i32,
                    })
                    .await
                    .map_err(|e| XRoadError::Crypto(format!("Signer RPC failed: {e}")))?;
                Ok(resp.into_inner())
            }
            Self::InMemory { service } => {
                service
                    .generate_software_key(token_id, key_label, usage)
                    .await
            }
        }
    }

    /// Deletes a key by ID.
    pub async fn delete_key(&self, key_id: &str) -> XRoadResult<()> {
        match self {
            Self::Remote { key_client, .. } => {
                let mut client = key_client.clone();
                client
                    .delete_key(DeleteKeyReq {
                        key_id: key_id.to_string(),
                        delete_from_device: true,
                    })
                    .await
                    .map_err(|e| XRoadError::Crypto(format!("Signer RPC failed: {e}")))?;
                Ok(())
            }
            Self::InMemory { service } => service.delete_key(key_id).await,
        }
    }

    /// Lists all tokens.
    pub async fn list_tokens(&self) -> XRoadResult<Vec<TokenInfoProto>> {
        match self {
            Self::Remote { token_client, .. } => {
                let mut client = token_client.clone();
                let resp = client
                    .list_tokens(Empty {})
                    .await
                    .map_err(|e| XRoadError::Crypto(format!("Signer RPC failed: {e}")))?;
                Ok(resp.into_inner().tokens)
            }
            Self::InMemory { service } => {
                let tokens_guard = service.tokens.read().await;
                let keys_guard = service.keys.read().await;
                let mut list = Vec::new();
                for t in tokens_guard.values() {
                    let key_list = keys_guard
                        .values()
                        .filter(|k| k.token_id == t.id)
                        .map(|k| KeyInfoProto {
                            id: k.id.clone(),
                            friendly_name: k.friendly_name.clone(),
                            usage: k.usage,
                            available: true,
                            label: k.label.clone(),
                            public_key: k.public_key_pem.clone(),
                            certs: k.certs.clone(),
                            cert_requests: k.cert_requests.clone(),
                            sign_mechanism_name: k.sign_mechanism.clone(),
                        })
                        .collect();

                    list.push(TokenInfoProto {
                        r#type: t.token_type.to_string(),
                        friendly_name: Some(t.name.clone()),
                        id: t.id.clone(),
                        read_only: false,
                        available: true,
                        active: t.logged_in,
                        serial_number: None,
                        label: Some(t.name.clone()),
                        status: TokenStatusInfo::Ok as i32,
                        key_info: key_list,
                        token_info: HashMap::new(),
                    });
                }
                Ok(list)
            }
        }
    }

    /// Gets token by ID.
    pub async fn get_token_by_id(&self, token_id: &str) -> XRoadResult<Option<TokenInfoProto>> {
        let tokens = self.list_tokens().await?;
        Ok(tokens.into_iter().find(|t| t.id == token_id))
    }

    /// Logs into a token with PIN.
    pub async fn login_token(&self, token_id: &str, pin: &str) -> XRoadResult<bool> {
        match self {
            Self::Remote { token_client, .. } => {
                let mut client = token_client.clone();
                client
                    .activate_token(ActivateTokenReq {
                        token_id: token_id.to_string(),
                        activate: true,
                        pin: Some(pin.as_bytes().to_vec()),
                    })
                    .await
                    .map_err(|e| XRoadError::Crypto(format!("Signer RPC failed: {e}")))?;
                Ok(true)
            }
            Self::InMemory { service } => service.login_token(token_id, pin).await,
        }
    }

    /// Logs out of a token.
    pub async fn logout_token(&self, token_id: &str) -> XRoadResult<()> {
        match self {
            Self::Remote { token_client, .. } => {
                let mut client = token_client.clone();
                client
                    .activate_token(ActivateTokenReq {
                        token_id: token_id.to_string(),
                        activate: false,
                        pin: None,
                    })
                    .await
                    .map_err(|e| XRoadError::Crypto(format!("Signer RPC failed: {e}")))?;
                Ok(())
            }
            Self::InMemory { service } => service.logout_token(token_id).await,
        }
    }

    /// Initializes software token password.
    pub async fn init_software_token(&self, pin: &str) -> XRoadResult<()> {
        match self {
            Self::Remote { token_client, .. } => {
                let mut client = token_client.clone();
                client
                    .init_software_token(InitSoftwareTokenReq {
                        pin: pin.as_bytes().to_vec(),
                    })
                    .await
                    .map_err(|e| XRoadError::Crypto(format!("Signer RPC failed: {e}")))?;
                Ok(())
            }
            Self::InMemory { service } => {
                let mut tokens = service.tokens.write().await;
                if let Some(t) = tokens.get_mut("0") {
                    t.pin = Some(pin.to_string());
                    t.has_pin = true;
                    t.logged_in = true;
                }
                Ok(())
            }
        }
    }

    /// Generates a PKCS#10 Certificate Signing Request (CSR).
    pub async fn generate_csr(&self, key_id: &str, subject_name: &str) -> XRoadResult<String> {
        match self {
            Self::Remote { cert_client, .. } => {
                let mut client = cert_client.clone();
                let resp = client
                    .generate_cert_request(GenerateCertRequestReq {
                        key_id: key_id.to_string(),
                        member_id: None,
                        key_usage: KeyUsageInfo::Signing as i32,
                        subject_name: subject_name.to_string(),
                        subject_alt_name: None,
                        format: 0,
                        certificate_profile: None,
                    })
                    .await
                    .map_err(|e| XRoadError::Crypto(format!("Signer RPC failed: {e}")))?;
                Ok(String::from_utf8_lossy(&resp.into_inner().cert_request).to_string())
            }
            Self::InMemory { service } => service.generate_csr(key_id, subject_name).await,
        }
    }

    /// Imports an issued certificate.
    pub async fn import_cert(
        &self,
        cert_data: Vec<u8>,
        initial_status: &str,
        activate: bool,
    ) -> XRoadResult<String> {
        match self {
            Self::Remote { cert_client, .. } => {
                let mut client = cert_client.clone();
                let resp = client
                    .import_cert(ImportCertReq {
                        cert_data,
                        initial_status: initial_status.to_string(),
                        activate,
                        member_id: None,
                    })
                    .await
                    .map_err(|e| XRoadError::Crypto(format!("Signer RPC failed: {e}")))?;
                Ok(resp.into_inner().key_id)
            }
            Self::InMemory { service } => {
                let keys = service.keys.read().await;
                let key_id = keys
                    .keys()
                    .next()
                    .cloned()
                    .unwrap_or_else(|| "soft-token-0".to_string());
                drop(keys);
                service.add_certificate(&key_id, cert_data).await?;
                Ok(key_id)
            }
        }
    }

    /// Activates or deactivates a certificate.
    pub async fn activate_cert(&self, cert_id_or_hash: &str, active: bool) -> XRoadResult<()> {
        match self {
            Self::Remote { cert_client, .. } => {
                let mut client = cert_client.clone();
                client
                    .activate_cert(ActivateCertReq {
                        cert_id_or_hash: cert_id_or_hash.to_string(),
                        active,
                    })
                    .await
                    .map_err(|e| XRoadError::Crypto(format!("Signer RPC failed: {e}")))?;
                Ok(())
            }
            Self::InMemory { service } => {
                let mut keys = service.keys.write().await;
                for k in keys.values_mut() {
                    if let Some(c) = k.certs.iter_mut().find(|c| c.id == cert_id_or_hash) {
                        c.active = active;
                        return Ok(());
                    }
                }
                Err(XRoadError::Crypto("Certificate not found".to_string()))
            }
        }
    }

    /// Deletes a certificate.
    pub async fn delete_cert(&self, cert_id: &str) -> XRoadResult<()> {
        match self {
            Self::Remote { cert_client, .. } => {
                let mut client = cert_client.clone();
                client
                    .delete_cert(DeleteCertReq {
                        cert_id: cert_id.to_string(),
                    })
                    .await
                    .map_err(|e| XRoadError::Crypto(format!("Signer RPC failed: {e}")))?;
                Ok(())
            }
            Self::InMemory { service } => {
                let mut keys = service.keys.write().await;
                for k in keys.values_mut() {
                    k.certs.retain(|c| c.id != cert_id);
                }
                Ok(())
            }
        }
    }

    /// Looks up certificate info by SHA-256 hash.
    pub async fn get_certificate_info_for_hash(
        &self,
        hash: &str,
    ) -> XRoadResult<Option<CertificateInfoProto>> {
        match self {
            Self::Remote { cert_client, .. } => {
                let mut client = cert_client.clone();
                match client
                    .get_certificate_info_for_hash(GetCertificateInfoForHashReq {
                        cert_hash: hash.to_string(),
                    })
                    .await
                {
                    Ok(resp) => Ok(resp.into_inner().certificate_info),
                    Err(s) if s.code() == tonic::Code::NotFound => Ok(None),
                    Err(e) => Err(XRoadError::Crypto(format!("Signer RPC failed: {e}"))),
                }
            }
            Self::InMemory { service } => {
                let target_hash = hash.to_lowercase();
                let keys = service.keys.read().await;
                for k in keys.values() {
                    for c in &k.certs {
                        let mut hasher = Sha256::new();
                        hasher.update(&c.certificate_bytes);
                        let hex = format!("{:x}", hasher.finalize());
                        if hex == target_hash {
                            return Ok(Some(c.clone()));
                        }
                    }
                }
                Ok(None)
            }
        }
    }

    /// Gets key ID corresponding to a certificate hash.
    pub async fn get_key_id_for_cert_hash(&self, hash: &str) -> XRoadResult<Option<String>> {
        match self {
            Self::Remote { key_client, .. } => {
                let mut client = key_client.clone();
                match client
                    .get_key_id_for_cert_hash(GetKeyIdForCertHashReq {
                        cert_hash: hash.to_string(),
                    })
                    .await
                {
                    Ok(resp) => Ok(Some(resp.into_inner().key_id)),
                    Err(s) if s.code() == tonic::Code::NotFound => Ok(None),
                    Err(e) => Err(XRoadError::Crypto(format!("Signer RPC failed: {e}"))),
                }
            }
            Self::InMemory { service } => {
                let target_hash = hash.to_lowercase();
                let keys = service.keys.read().await;
                for k in keys.values() {
                    for c in &k.certs {
                        let mut hasher = Sha256::new();
                        hasher.update(&c.certificate_bytes);
                        let hex = format!("{:x}", hasher.finalize());
                        if hex == target_hash {
                            return Ok(Some(k.id.clone()));
                        }
                    }
                }
                Ok(None)
            }
        }
    }

    /// Retrieves cached OCSP responses for given hashes.
    pub async fn get_ocsp_responses(
        &self,
        cert_hashes: &[String],
    ) -> XRoadResult<HashMap<String, String>> {
        match self {
            Self::Remote { ocsp_client, .. } => {
                let mut client = ocsp_client.clone();
                let resp = client
                    .get_ocsp_responses(GetOcspResponsesReq {
                        cert_hash: cert_hashes.to_vec(),
                    })
                    .await
                    .map_err(|e| XRoadError::Crypto(format!("Signer RPC failed: {e}")))?;
                Ok(resp.into_inner().base64_encoded_responses)
            }
            Self::InMemory { service } => {
                let cache = service.ocsp_cache.read().await;
                let mut map = HashMap::new();
                for h in cert_hashes {
                    if let Some(r) = cache.get(&h.to_lowercase()) {
                        map.insert(h.clone(), r.clone());
                    }
                }
                Ok(map)
            }
        }
    }

    /// Stores OCSP responses in Signer's OCSP cache.
    pub async fn set_ocsp_responses(
        &self,
        cert_hashes: Vec<String>,
        responses: Vec<String>,
    ) -> XRoadResult<()> {
        match self {
            Self::Remote { ocsp_client, .. } => {
                let mut client = ocsp_client.clone();
                client
                    .set_ocsp_responses(SetOcspResponsesReq {
                        cert_hashes,
                        base64_encoded_responses: responses,
                    })
                    .await
                    .map_err(|e| XRoadError::Crypto(format!("Signer RPC failed: {e}")))?;
                Ok(())
            }
            Self::InMemory { service } => {
                let mut cache = service.ocsp_cache.write().await;
                for (h, r) in cert_hashes.into_iter().zip(responses) {
                    cache.insert(h.to_lowercase(), r);
                }
                Ok(())
            }
        }
    }
}
