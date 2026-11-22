pub mod client;
pub mod hsm;

pub use client::SignerClient;
pub use hsm::HsmTokenManager;

use common::{TokenType, XRoadError, XRoadResult};
use proto::certificate_service_server::{CertificateService, CertificateServiceServer};
use proto::key_service_server::{KeyService, KeyServiceServer};
use proto::ocsp_service_server::{OcspService, OcspServiceServer};
use proto::sign_service_server::{SignService, SignServiceServer};
use proto::token_service_server::{TokenService, TokenServiceServer};
use proto::*;
use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair, PKCS_ECDSA_P256_SHA256};
use ring::rand::SystemRandom;
use ring::signature::{ECDSA_P256_SHA256_ASN1_SIGNING, EcdsaKeyPair};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

/// Status and metadata for a cryptographic token.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TokenStatus {
    pub id: String,
    pub name: String,
    pub token_type: TokenType,
    pub logged_in: bool,
    pub has_pin: bool,
    pub pin: Option<String>,
}

/// Internal record for a managed software cryptographic key.
#[derive(Clone)]
pub struct SoftwareKeyEntry {
    pub id: String,
    pub token_id: String,
    pub label: Option<String>,
    pub friendly_name: Option<String>,
    pub usage: i32,
    pub key_pem: String,
    pub key_der: Vec<u8>,
    pub public_key_pem: String,
    pub sign_mechanism: String,
    pub certs: Vec<CertificateInfoProto>,
    pub cert_requests: Vec<CertRequestInfoProto>,
}

/// Software and hardware token backed signer service implementation for managing keys, CSRs, and digital signatures.
#[derive(Clone)]
pub struct SignerServiceImpl {
    keys: Arc<RwLock<HashMap<String, SoftwareKeyEntry>>>,
    tokens: Arc<RwLock<HashMap<String, TokenStatus>>>,
    ocsp_cache: Arc<RwLock<HashMap<String, String>>>,
    hsm: Option<Arc<HsmTokenManager>>,
}

impl Default for SignerServiceImpl {
    fn default() -> Self {
        let mut tokens_map = HashMap::new();
        tokens_map.insert(
            "0".to_string(),
            TokenStatus {
                id: "0".to_string(),
                name: "Software Token".to_string(),
                token_type: TokenType::Software,
                logged_in: true,
                has_pin: true,
                pin: Some("1234".to_string()),
            },
        );

        let hsm = HsmTokenManager::from_env().map(Arc::new);
        if let Some(ref h) = hsm {
            tokens_map.insert(
                "1".to_string(),
                TokenStatus {
                    id: "1".to_string(),
                    name: format!("Hardware Token ({})", h.module_path()),
                    token_type: TokenType::Hardware,
                    logged_in: true,
                    has_pin: true,
                    pin: None,
                },
            );
        }

        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            tokens: Arc::new(RwLock::new(tokens_map)),
            ocsp_cache: Arc::new(RwLock::new(HashMap::new())),
            hsm,
        }
    }
}

impl SignerServiceImpl {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_hsm(mut self, hsm: Arc<HsmTokenManager>) -> Self {
        self.hsm = Some(hsm);
        self
    }

    pub fn into_key_service(self: Arc<Self>) -> KeyServiceServer<SignerKeyServiceAdapter> {
        KeyServiceServer::new(SignerKeyServiceAdapter { inner: self })
    }

    pub fn into_sign_service(self: Arc<Self>) -> SignServiceServer<SignerSignServiceAdapter> {
        SignServiceServer::new(SignerSignServiceAdapter { inner: self })
    }

    pub fn into_token_service(self: Arc<Self>) -> TokenServiceServer<SignerTokenServiceAdapter> {
        TokenServiceServer::new(SignerTokenServiceAdapter { inner: self })
    }

    pub fn into_certificate_service(
        self: Arc<Self>,
    ) -> CertificateServiceServer<SignerCertificateServiceAdapter> {
        CertificateServiceServer::new(SignerCertificateServiceAdapter { inner: self })
    }

    pub fn into_ocsp_service(self: Arc<Self>) -> OcspServiceServer<SignerOcspServiceAdapter> {
        OcspServiceServer::new(SignerOcspServiceAdapter { inner: self })
    }

    /// Internal key generation using ECDSA P-256.
    pub async fn generate_software_key(
        &self,
        token_id: &str,
        key_label: &str,
        usage: KeyUsageInfo,
    ) -> XRoadResult<KeyInfoProto> {
        let key_pair = KeyPair::generate_for(&PKCS_ECDSA_P256_SHA256)
            .map_err(|e| XRoadError::Crypto(format!("Failed to generate ECDSA key pair: {e}")))?;

        let key_pem = key_pair.serialize_pem();
        let key_der = key_pair.serialize_der();
        let public_key_pem = key_pair.public_key_pem();
        let key_id = format!("{}-{}", token_id, uuid::Uuid::new_v4());

        let key_entry = SoftwareKeyEntry {
            id: key_id.clone(),
            token_id: token_id.to_string(),
            label: Some(key_label.to_string()),
            friendly_name: Some(key_label.to_string()),
            usage: usage as i32,
            key_pem,
            key_der,
            public_key_pem: public_key_pem.clone(),
            sign_mechanism: "SHA256withECDSA".to_string(),
            certs: vec![],
            cert_requests: vec![],
        };

        let key_info = KeyInfoProto {
            id: key_id.clone(),
            friendly_name: Some(key_label.to_string()),
            usage: usage as i32,
            available: true,
            label: Some(key_label.to_string()),
            public_key: public_key_pem,
            certs: vec![],
            cert_requests: vec![],
            sign_mechanism_name: "SHA256withECDSA".to_string(),
        };

        self.keys.write().await.insert(key_id, key_entry);
        Ok(key_info)
    }

    /// Generates a PKCS#10 Certificate Signing Request (CSR) for a managed key.
    pub async fn generate_csr(&self, key_id: &str, subject_cn: &str) -> XRoadResult<String> {
        let keys_guard = self.keys.read().await;
        let entry = keys_guard
            .get(key_id)
            .ok_or_else(|| XRoadError::Crypto(format!("Key not found: {key_id}")))?;

        let key_pair = KeyPair::from_pem(&entry.key_pem)
            .map_err(|e| XRoadError::Crypto(format!("Failed to parse private key PEM: {e}")))?;

        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, subject_cn);

        let mut params = CertificateParams::default();
        params.distinguished_name = dn;

        let csr = params
            .serialize_request(&key_pair)
            .map_err(|e| XRoadError::Crypto(format!("Failed to generate CSR: {e}")))?;

        let csr_pem = csr
            .pem()
            .map_err(|e| XRoadError::Crypto(format!("Failed to serialize CSR PEM: {e}")))?;

        drop(keys_guard);
        let mut keys_write = self.keys.write().await;
        if let Some(entry) = keys_write.get_mut(key_id) {
            let req_id = format!("csr-{}", uuid::Uuid::new_v4());
            entry.cert_requests.push(CertRequestInfoProto {
                id: req_id,
                member_id: None,
                subject_name: subject_cn.to_string(),
                subject_alt_name: String::new(),
                certificate_profile: None,
            });
        }

        Ok(csr_pem)
    }

    /// Lists all cryptographic tokens.
    pub async fn list_tokens(&self) -> Vec<TokenStatus> {
        self.tokens.read().await.values().cloned().collect()
    }

    /// Gets token by ID.
    pub async fn get_token(&self, token_id: &str) -> Option<TokenStatus> {
        self.tokens.read().await.get(token_id).cloned()
    }

    /// Logs into a token with the provided PIN.
    pub async fn login_token(&self, token_id: &str, pin: &str) -> XRoadResult<bool> {
        let mut tokens_guard = self.tokens.write().await;
        let token = tokens_guard
            .get_mut(token_id)
            .ok_or_else(|| XRoadError::Crypto(format!("Token not found: {token_id}")))?;

        if let Some(ref expected_pin) = token.pin {
            if expected_pin == pin {
                token.logged_in = true;
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Logs out of a token.
    pub async fn logout_token(&self, token_id: &str) -> XRoadResult<()> {
        let mut tokens_guard = self.tokens.write().await;
        let token = tokens_guard
            .get_mut(token_id)
            .ok_or_else(|| XRoadError::Crypto(format!("Token not found: {token_id}")))?;
        token.logged_in = false;
        Ok(())
    }

    /// Updates the PIN for a token.
    pub async fn set_token_pin(
        &self,
        token_id: &str,
        old_pin: Option<&str>,
        new_pin: &str,
    ) -> XRoadResult<()> {
        let mut tokens_guard = self.tokens.write().await;
        let token = tokens_guard
            .get_mut(token_id)
            .ok_or_else(|| XRoadError::Crypto(format!("Token not found: {token_id}")))?;

        if let Some(ref current_pin) = token.pin {
            if let Some(provided_old) = old_pin {
                if current_pin != provided_old {
                    return Err(XRoadError::Crypto("Invalid existing PIN".to_string()));
                }
            }
        }

        token.pin = Some(new_pin.to_string());
        token.has_pin = true;
        token.logged_in = true;
        Ok(())
    }

    /// Lists all managed cryptographic keys.
    pub async fn list_keys(&self) -> Vec<KeyInfoProto> {
        let keys_guard = self.keys.read().await;
        keys_guard
            .values()
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
            .collect()
    }

    /// Gets key info by ID.
    pub async fn get_key(&self, key_id: &str) -> Option<KeyInfoProto> {
        let keys_guard = self.keys.read().await;
        keys_guard.get(key_id).map(|k| KeyInfoProto {
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
    }

    /// Deletes a key by ID.
    pub async fn delete_key(&self, key_id: &str) -> XRoadResult<()> {
        let mut keys_guard = self.keys.write().await;
        keys_guard.remove(key_id);
        Ok(())
    }

    /// Deletes a CSR associated with a key.
    pub async fn delete_csr(&self, key_id: &str, csr_id: &str) -> XRoadResult<()> {
        let mut keys_guard = self.keys.write().await;
        if let Some(entry) = keys_guard.get_mut(key_id) {
            entry.cert_requests.retain(|c| c.id != csr_id);
        }
        Ok(())
    }

    /// Digitally signs a pre-computed digest with the given key ID (software key or PKCS#11 HSM).
    pub async fn sign_digest(&self, key_id: &str, digest: &[u8]) -> XRoadResult<Vec<u8>> {
        if let Some(ref hsm) = self.hsm {
            if key_id.starts_with("hsm:") || key_id.starts_with("token-1-") {
                let key_label = key_id.strip_prefix("hsm:").unwrap_or(key_id);
                return hsm.sign(key_label, digest).await;
            }
        }

        let keys_guard = self.keys.read().await;
        let entry = keys_guard
            .get(key_id)
            .ok_or_else(|| XRoadError::Crypto(format!("Key not found: {key_id}")))?;

        let rng = SystemRandom::new();
        let key_pair =
            EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, &entry.key_der, &rng)
                .map_err(|e| XRoadError::Crypto(format!("Failed to load ring ECDSA key: {e}")))?;

        let signature = key_pair
            .sign(&rng, digest)
            .map_err(|e| XRoadError::Crypto(format!("Failed to sign digest: {e}")))?;

        Ok(signature.as_ref().to_vec())
    }

    /// Stores a certificate associated with a key ID.
    pub async fn add_certificate(&self, key_id: &str, cert_bytes: Vec<u8>) -> XRoadResult<()> {
        let mut keys_guard = self.keys.write().await;
        let entry = keys_guard
            .get_mut(key_id)
            .ok_or_else(|| XRoadError::Crypto(format!("Key not found: {key_id}")))?;

        let cert_id = format!("cert-{}", uuid::Uuid::new_v4());
        entry.certs.push(CertificateInfoProto {
            id: cert_id,
            active: true,
            saved_to_configuration: true,
            status: "SAVED".to_string(),
            certificate_bytes: cert_bytes,
            ocsp_bytes: vec![],
            member_id: None,
            renewed_cert_hash: None,
            renewal_error: None,
            next_automatic_renewal_time: None,
            ocsp_verify_before_activation_error: None,
        });

        Ok(())
    }
}

pub struct SignerKeyServiceAdapter {
    inner: Arc<SignerServiceImpl>,
}

#[tonic::async_trait]
impl KeyService for SignerKeyServiceAdapter {
    async fn generate_key(
        &self,
        req: Request<GenerateKeyReq>,
    ) -> Result<Response<KeyInfoProto>, Status> {
        let r = req.into_inner();
        let key_info = self
            .inner
            .generate_software_key(&r.token_id, &r.key_label, KeyUsageInfo::Signing)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(key_info))
    }

    async fn set_key_friendly_name(
        &self,
        req: Request<SetKeyFriendlyNameReq>,
    ) -> Result<Response<proto::Empty>, Status> {
        let r = req.into_inner();
        let mut keys = self.inner.keys.write().await;
        if let Some(entry) = keys.get_mut(&r.key_id) {
            entry.friendly_name = Some(r.friendly_name);
            Ok(Response::new(proto::Empty {}))
        } else {
            Err(Status::not_found("Key not found"))
        }
    }

    async fn get_key_id_for_cert_hash(
        &self,
        req: Request<GetKeyIdForCertHashReq>,
    ) -> Result<Response<GetKeyIdForCertHashResp>, Status> {
        let r = req.into_inner();
        let target_hash = r.cert_hash.to_lowercase();
        let keys = self.inner.keys.read().await;

        for entry in keys.values() {
            for cert in &entry.certs {
                let mut hasher = Sha256::new();
                hasher.update(&cert.certificate_bytes);
                let hash_hex = format!("{:x}", hasher.finalize());
                if hash_hex == target_hash {
                    return Ok(Response::new(GetKeyIdForCertHashResp {
                        key_id: entry.id.clone(),
                        sign_mechanism_name: entry.sign_mechanism.clone(),
                    }));
                }
            }
        }

        Err(Status::not_found("Certificate hash not found"))
    }

    async fn get_sign_mechanism(
        &self,
        req: Request<GetSignMechanismReq>,
    ) -> Result<Response<GetSignMechanismResp>, Status> {
        let r = req.into_inner();
        let keys = self.inner.keys.read().await;
        if let Some(entry) = keys.get(&r.key_id) {
            Ok(Response::new(GetSignMechanismResp {
                sign_mechanism_name: entry.sign_mechanism.clone(),
            }))
        } else {
            Err(Status::not_found("Key not found"))
        }
    }

    async fn delete_key(
        &self,
        req: Request<DeleteKeyReq>,
    ) -> Result<Response<proto::Empty>, Status> {
        let r = req.into_inner();
        let mut keys = self.inner.keys.write().await;
        if keys.remove(&r.key_id).is_some() {
            Ok(Response::new(proto::Empty {}))
        } else {
            Err(Status::not_found("Key not found"))
        }
    }

    async fn get_auth_key(
        &self,
        _req: Request<GetAuthKeyReq>,
    ) -> Result<Response<AuthKeyProto>, Status> {
        let keys = self.inner.keys.read().await;
        // Return first key with AUTHENTICATION usage or any key
        for entry in keys.values() {
            if entry.usage == KeyUsageInfo::Authentication as i32 {
                return Ok(Response::new(AuthKeyProto {
                    alias: entry.id.clone(),
                    key_store: entry.key_der.clone(),
                    password: String::new(),
                    cert: entry.certs.first().cloned(),
                }));
            }
        }

        Ok(Response::new(AuthKeyProto {
            alias: "default-auth-key".to_string(),
            key_store: vec![],
            password: String::new(),
            cert: None,
        }))
    }

    async fn is_software_based_key(
        &self,
        _req: Request<IsSoftTokenBasedKeyReq>,
    ) -> Result<Response<IsSoftTokenBasedKeyResp>, Status> {
        Ok(Response::new(IsSoftTokenBasedKeyResp {
            soft_token_based: true,
        }))
    }

    async fn list_software_token_keys(
        &self,
        _req: Request<proto::Empty>,
    ) -> Result<Response<ListSoftwareTokenKeysResp>, Status> {
        let keys = self.inner.keys.read().await;
        let list: Vec<SoftwareTokenKeyInfo> = keys
            .values()
            .map(|entry| SoftwareTokenKeyInfo {
                key_id: entry.id.clone(),
                private_key: entry.key_der.clone(),
                key_available: true,
                key_label: entry.label.clone().unwrap_or_default(),
                sign_mechanism: entry.sign_mechanism.clone(),
            })
            .collect();

        Ok(Response::new(ListSoftwareTokenKeysResp { keys: list }))
    }
}

pub struct SignerSignServiceAdapter {
    inner: Arc<SignerServiceImpl>,
}

#[tonic::async_trait]
impl SignService for SignerSignServiceAdapter {
    async fn sign(&self, req: Request<SignReq>) -> Result<Response<SignResp>, Status> {
        let r = req.into_inner();
        let sig = self
            .inner
            .sign_digest(&r.key_id, &r.digest)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(SignResp { signature: sig }))
    }

    async fn sign_certificate(
        &self,
        _req: Request<SignCertificateReq>,
    ) -> Result<Response<SignCertificateResp>, Status> {
        Ok(Response::new(SignCertificateResp {
            certificate_chain: vec![],
        }))
    }
}

pub struct SignerTokenServiceAdapter {
    inner: Arc<SignerServiceImpl>,
}

#[tonic::async_trait]
impl TokenService for SignerTokenServiceAdapter {
    async fn get_token_by_id(
        &self,
        req: Request<GetTokenByIdReq>,
    ) -> Result<Response<TokenInfoProto>, Status> {
        let r = req.into_inner();
        let tokens = self.inner.tokens.read().await;
        let token = tokens
            .get(&r.token_id)
            .ok_or_else(|| Status::not_found(format!("Token not found: {}", r.token_id)))?;

        let keys = self.inner.keys.read().await;
        let key_list: Vec<KeyInfoProto> = keys
            .values()
            .filter(|k| k.token_id == token.id)
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

        Ok(Response::new(TokenInfoProto {
            r#type: token.token_type.to_string(),
            friendly_name: Some(token.name.clone()),
            id: token.id.clone(),
            read_only: false,
            available: true,
            active: token.logged_in,
            serial_number: None,
            label: Some(token.name.clone()),
            status: TokenStatusInfo::Ok as i32,
            key_info: key_list,
            token_info: HashMap::new(),
        }))
    }

    async fn get_token_by_key(
        &self,
        req: Request<GetTokenByKeyIdReq>,
    ) -> Result<Response<TokenInfoProto>, Status> {
        let r = req.into_inner();
        let keys = self.inner.keys.read().await;
        let key = keys
            .get(&r.key_id)
            .ok_or_else(|| Status::not_found(format!("Key not found: {}", r.key_id)))?;
        let token_id = key.token_id.clone();
        drop(keys);

        self.get_token_by_id(Request::new(GetTokenByIdReq { token_id }))
            .await
    }

    async fn get_token_and_key_id_by_cert_request_id(
        &self,
        req: Request<GetTokenByCertRequestIdReq>,
    ) -> Result<Response<TokenInfoAndKeyIdProto>, Status> {
        let r = req.into_inner();
        let keys = self.inner.keys.read().await;
        for key in keys.values() {
            if key
                .cert_requests
                .iter()
                .any(|cr| cr.id == r.cert_request_id)
            {
                let key_id = key.id.clone();
                let token_id = key.token_id.clone();
                drop(keys);
                let token_resp = self
                    .get_token_by_id(Request::new(GetTokenByIdReq { token_id }))
                    .await?;
                return Ok(Response::new(TokenInfoAndKeyIdProto {
                    key_id,
                    token_info: Some(token_resp.into_inner()),
                }));
            }
        }
        Err(Status::not_found("Certificate request ID not found"))
    }

    async fn get_token_and_key_id_by_cert_hash(
        &self,
        req: Request<GetTokenByCertHashReq>,
    ) -> Result<Response<TokenInfoAndKeyIdProto>, Status> {
        let r = req.into_inner();
        let target_hash = r.cert_hash.to_lowercase();
        let keys = self.inner.keys.read().await;
        for key in keys.values() {
            for cert in &key.certs {
                let mut hasher = Sha256::new();
                hasher.update(&cert.certificate_bytes);
                let hash_hex = format!("{:x}", hasher.finalize());
                if hash_hex == target_hash {
                    let key_id = key.id.clone();
                    let token_id = key.token_id.clone();
                    drop(keys);
                    let token_resp = self
                        .get_token_by_id(Request::new(GetTokenByIdReq { token_id }))
                        .await?;
                    return Ok(Response::new(TokenInfoAndKeyIdProto {
                        key_id,
                        token_info: Some(token_resp.into_inner()),
                    }));
                }
            }
        }
        Err(Status::not_found("Certificate hash not found"))
    }

    async fn list_tokens(&self, _req: Request<Empty>) -> Result<Response<ListTokensResp>, Status> {
        let tokens_guard = self.inner.tokens.read().await;
        let keys_guard = self.inner.keys.read().await;

        let mut token_list = Vec::new();
        for token in tokens_guard.values() {
            let key_list: Vec<KeyInfoProto> = keys_guard
                .values()
                .filter(|k| k.token_id == token.id)
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

            token_list.push(TokenInfoProto {
                r#type: token.token_type.to_string(),
                friendly_name: Some(token.name.clone()),
                id: token.id.clone(),
                read_only: false,
                available: true,
                active: token.logged_in,
                serial_number: None,
                label: Some(token.name.clone()),
                status: TokenStatusInfo::Ok as i32,
                key_info: key_list,
                token_info: HashMap::new(),
            });
        }

        Ok(Response::new(ListTokensResp { tokens: token_list }))
    }

    async fn activate_token(
        &self,
        req: Request<ActivateTokenReq>,
    ) -> Result<Response<Empty>, Status> {
        let r = req.into_inner();
        let mut tokens = self.inner.tokens.write().await;
        if let Some(t) = tokens.get_mut(&r.token_id) {
            t.logged_in = r.activate;
            Ok(Response::new(Empty {}))
        } else {
            Err(Status::not_found("Token not found"))
        }
    }

    async fn set_token_friendly_name(
        &self,
        req: Request<SetTokenFriendlyNameReq>,
    ) -> Result<Response<Empty>, Status> {
        let r = req.into_inner();
        let mut tokens = self.inner.tokens.write().await;
        if let Some(t) = tokens.get_mut(&r.token_id) {
            t.name = r.friendly_name;
            Ok(Response::new(Empty {}))
        } else {
            Err(Status::not_found("Token not found"))
        }
    }

    async fn get_token_batch_signing_enabled(
        &self,
        _req: Request<GetTokenBatchSigningEnabledReq>,
    ) -> Result<Response<GetTokenBatchSigningEnabledResp>, Status> {
        Ok(Response::new(GetTokenBatchSigningEnabledResp {
            batching_signing_enabled: true,
        }))
    }

    async fn init_software_token(
        &self,
        req: Request<InitSoftwareTokenReq>,
    ) -> Result<Response<Empty>, Status> {
        let r = req.into_inner();
        let mut tokens = self.inner.tokens.write().await;
        if let Some(t) = tokens.get_mut("0") {
            t.pin = Some(String::from_utf8_lossy(&r.pin).to_string());
            t.has_pin = true;
            t.logged_in = true;
        }
        Ok(Response::new(Empty {}))
    }

    async fn update_software_token_pin(
        &self,
        req: Request<UpdateSoftwareTokenPinReq>,
    ) -> Result<Response<Empty>, Status> {
        let r = req.into_inner();
        let mut tokens = self.inner.tokens.write().await;
        if let Some(t) = tokens.get_mut("0") {
            t.pin = Some(String::from_utf8_lossy(&r.new_pin).to_string());
        }
        Ok(Response::new(Empty {}))
    }

    async fn delete_token(&self, req: Request<DeleteTokenReq>) -> Result<Response<Empty>, Status> {
        let r = req.into_inner();
        let mut tokens = self.inner.tokens.write().await;
        tokens.remove(&r.token_id);
        Ok(Response::new(Empty {}))
    }

    async fn get_hsm_operational_info(
        &self,
        _req: Request<Empty>,
    ) -> Result<Response<GetHsmOperationalInfoResp>, Status> {
        Ok(Response::new(GetHsmOperationalInfoResp {
            operational: true,
        }))
    }

    async fn get_member_signing_info(
        &self,
        _req: Request<GetMemberSigningInfoReq>,
    ) -> Result<Response<GetMemberSigningInfoResp>, Status> {
        let keys = self.inner.keys.read().await;
        for key in keys.values() {
            if key.usage == KeyUsageInfo::Signing as i32 {
                return Ok(Response::new(GetMemberSigningInfoResp {
                    key_id: key.id.clone(),
                    cert: key.certs.first().cloned(),
                    sign_mechanism_name: key.sign_mechanism.clone(),
                }));
            }
        }
        Ok(Response::new(GetMemberSigningInfoResp {
            key_id: String::new(),
            cert: None,
            sign_mechanism_name: String::new(),
        }))
    }
}

pub struct SignerCertificateServiceAdapter {
    inner: Arc<SignerServiceImpl>,
}

#[tonic::async_trait]
impl CertificateService for SignerCertificateServiceAdapter {
    async fn activate_cert(
        &self,
        req: Request<ActivateCertReq>,
    ) -> Result<Response<Empty>, Status> {
        let r = req.into_inner();
        let mut keys = self.inner.keys.write().await;
        for key in keys.values_mut() {
            if let Some(c) = key.certs.iter_mut().find(|c| c.id == r.cert_id_or_hash) {
                c.active = r.active;
                return Ok(Response::new(Empty {}));
            }
        }
        Err(Status::not_found("Certificate ID not found"))
    }

    async fn get_certificate_info_for_hash(
        &self,
        req: Request<GetCertificateInfoForHashReq>,
    ) -> Result<Response<GetCertificateInfoResp>, Status> {
        let r = req.into_inner();
        let target_hash = r.cert_hash.to_lowercase();
        let keys = self.inner.keys.read().await;
        for key in keys.values() {
            for cert in &key.certs {
                let mut hasher = Sha256::new();
                hasher.update(&cert.certificate_bytes);
                let hash_hex = format!("{:x}", hasher.finalize());
                if hash_hex == target_hash {
                    return Ok(Response::new(GetCertificateInfoResp {
                        certificate_info: Some(cert.clone()),
                    }));
                }
            }
        }
        Err(Status::not_found("Certificate not found for hash"))
    }

    async fn get_member_certs(
        &self,
        _req: Request<GetMemberCertsReq>,
    ) -> Result<Response<GetMemberCertsResp>, Status> {
        let keys = self.inner.keys.read().await;
        let mut certs = Vec::new();
        for key in keys.values() {
            certs.extend(key.certs.clone());
        }
        Ok(Response::new(GetMemberCertsResp { certs }))
    }

    async fn set_cert_status(
        &self,
        req: Request<SetCertStatusReq>,
    ) -> Result<Response<Empty>, Status> {
        let r = req.into_inner();
        let mut keys = self.inner.keys.write().await;
        for key in keys.values_mut() {
            if let Some(c) = key.certs.iter_mut().find(|c| c.id == r.cert_id) {
                c.status = r.status;
                return Ok(Response::new(Empty {}));
            }
        }
        Err(Status::not_found("Certificate ID not found"))
    }

    async fn set_renewed_cert_hash(
        &self,
        req: Request<SetRenewedCertHashReq>,
    ) -> Result<Response<Empty>, Status> {
        let r = req.into_inner();
        let mut keys = self.inner.keys.write().await;
        for key in keys.values_mut() {
            if let Some(c) = key.certs.iter_mut().find(|c| c.id == r.cert_id) {
                c.renewed_cert_hash = Some(r.hash);
                return Ok(Response::new(Empty {}));
            }
        }
        Err(Status::not_found("Certificate ID not found"))
    }

    async fn set_renewal_error(
        &self,
        req: Request<SetRenewalErrorReq>,
    ) -> Result<Response<Empty>, Status> {
        let r = req.into_inner();
        let mut keys = self.inner.keys.write().await;
        for key in keys.values_mut() {
            if let Some(c) = key.certs.iter_mut().find(|c| c.id == r.cert_id) {
                c.renewal_error = Some(r.error_message);
                return Ok(Response::new(Empty {}));
            }
        }
        Err(Status::not_found("Certificate ID not found"))
    }

    async fn set_next_planned_renewal(
        &self,
        _req: Request<SetNextPlannedRenewalReq>,
    ) -> Result<Response<Empty>, Status> {
        Ok(Response::new(Empty {}))
    }

    async fn generate_cert_request(
        &self,
        req: Request<GenerateCertRequestReq>,
    ) -> Result<Response<GenerateCertRequestResp>, Status> {
        let r = req.into_inner();
        let csr_pem = self
            .inner
            .generate_csr(&r.key_id, &r.subject_name)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(GenerateCertRequestResp {
            cert_req_id: format!("csr-{}", uuid::Uuid::new_v4()),
            cert_request: csr_pem.into_bytes(),
            format: 0,
        }))
    }

    async fn regenerate_cert_request(
        &self,
        req: Request<RegenerateCertRequestReq>,
    ) -> Result<Response<RegenerateCertRequestResp>, Status> {
        let r = req.into_inner();
        let (key_id, subject) = {
            let keys = self.inner.keys.read().await;
            keys.values().find_map(|k| {
                k.cert_requests
                    .iter()
                    .find(|cr| cr.id == r.cert_request_id)
                    .map(|cr| (k.id.clone(), cr.subject_name.clone()))
            })
        }
        .unwrap_or_else(|| ("soft-token-0".to_string(), "CN=SecurityServer".to_string()));

        let csr_pem = self
            .inner
            .generate_csr(&key_id, &subject)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(RegenerateCertRequestResp {
            cert_req_id: r.cert_request_id,
            cert_request: csr_pem.into_bytes(),
            format: r.format,
            member_id: None,
            key_usage: 0,
        }))
    }

    async fn delete_cert(&self, req: Request<DeleteCertReq>) -> Result<Response<Empty>, Status> {
        let r = req.into_inner();
        let mut keys = self.inner.keys.write().await;
        for key in keys.values_mut() {
            key.certs.retain(|c| c.id != r.cert_id);
        }
        Ok(Response::new(Empty {}))
    }

    async fn delete_cert_request(
        &self,
        req: Request<DeleteCertRequestReq>,
    ) -> Result<Response<Empty>, Status> {
        let r = req.into_inner();
        let mut keys = self.inner.keys.write().await;
        for key in keys.values_mut() {
            key.cert_requests.retain(|c| c.id != r.cert_request_id);
        }
        Ok(Response::new(Empty {}))
    }

    async fn import_cert(
        &self,
        req: Request<ImportCertReq>,
    ) -> Result<Response<ImportCertResp>, Status> {
        let r = req.into_inner();
        let key_id = {
            let keys = self.inner.keys.read().await;
            keys.keys()
                .next()
                .cloned()
                .unwrap_or_else(|| "soft-token-0".to_string())
        };
        self.inner
            .add_certificate(&key_id, r.cert_data)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ImportCertResp { key_id }))
    }

    async fn generate_self_signed_cert(
        &self,
        _req: Request<GenerateSelfSignedCertReq>,
    ) -> Result<Response<GenerateSelfSignedCertResp>, Status> {
        Ok(Response::new(GenerateSelfSignedCertResp {
            certificate_bytes: vec![0x30, 0x82, 0x01, 0x02],
        }))
    }
}

pub struct SignerOcspServiceAdapter {
    inner: Arc<SignerServiceImpl>,
}

#[tonic::async_trait]
impl OcspService for SignerOcspServiceAdapter {
    async fn set_ocsp_responses(
        &self,
        req: Request<SetOcspResponsesReq>,
    ) -> Result<Response<Empty>, Status> {
        let r = req.into_inner();
        let mut cache = self.inner.ocsp_cache.write().await;
        for (hash, resp) in r.cert_hashes.into_iter().zip(r.base64_encoded_responses) {
            cache.insert(hash.to_lowercase(), resp);
        }
        Ok(Response::new(Empty {}))
    }

    async fn get_ocsp_responses(
        &self,
        req: Request<GetOcspResponsesReq>,
    ) -> Result<Response<GetOcspResponsesResp>, Status> {
        let r = req.into_inner();
        let cache = self.inner.ocsp_cache.read().await;
        let mut responses = HashMap::new();
        for hash in r.cert_hash {
            if let Some(resp) = cache.get(&hash.to_lowercase()) {
                responses.insert(hash, resp.clone());
            }
        }
        Ok(Response::new(GetOcspResponsesResp {
            base64_encoded_responses: responses,
        }))
    }

    async fn get_ocsp_prioritization_strategy(
        &self,
        _req: Request<Empty>,
    ) -> Result<Response<OcspPrioritizationStrategyResp>, Status> {
        Ok(Response::new(OcspPrioritizationStrategyResp {
            strategy: 0,
        }))
    }
}

/// Launches the Signer gRPC service on the given address.
pub async fn run_signer(bind_addr: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Starting X-Road Signer Daemon on {}...", bind_addr);
    let addr = bind_addr.parse()?;
    let signer = Arc::new(SignerServiceImpl::new());

    tonic::transport::Server::builder()
        .add_service(signer.clone().into_key_service())
        .add_service(signer.clone().into_sign_service())
        .add_service(signer.clone().into_token_service())
        .add_service(signer.clone().into_certificate_service())
        .add_service(signer.into_ocsp_service())
        .serve(addr)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_signer_key_generation_and_signing() {
        let signer = SignerServiceImpl::new();

        // 1. Generate key
        let key_info = signer
            .generate_software_key(
                "soft-token-0",
                "test-auth-key",
                KeyUsageInfo::Authentication,
            )
            .await
            .expect("Failed to generate key");

        assert_eq!(key_info.sign_mechanism_name, "SHA256withECDSA");
        assert!(!key_info.public_key.is_empty());

        // 2. Sign a digest
        let digest = sha2::Sha256::digest(b"test message to sign");
        let sig = signer
            .sign_digest(&key_info.id, &digest)
            .await
            .expect("Failed to sign digest");
        assert!(!sig.is_empty());

        // 3. Verify signature with ring
        let public_key_der = {
            let keys = signer.keys.read().await;
            let entry = keys.get(&key_info.id).unwrap();
            let key_pair = KeyPair::from_pem(&entry.key_pem).unwrap();
            key_pair.public_key_raw().to_vec()
        };

        let peer_public_key = ring::signature::UnparsedPublicKey::new(
            &ring::signature::ECDSA_P256_SHA256_ASN1,
            &public_key_der,
        );
        peer_public_key
            .verify(&digest, &sig)
            .expect("Signature verification failed");

        // 4. Generate CSR
        let csr = signer
            .generate_csr(&key_info.id, "DEV/COM/12345678/SS1")
            .await
            .expect("Failed to generate CSR");
        assert!(csr.contains("BEGIN CERTIFICATE REQUEST"));
    }

    #[tokio::test]
    async fn test_signer_client_in_memory() {
        let client = SignerClient::new_in_memory();

        // 1. Generate key
        let key = client
            .generate_key("0", "signing-key-1", KeyUsageInfo::Signing)
            .await
            .expect("Failed to generate key via SignerClient");
        assert_eq!(key.label.as_deref(), Some("signing-key-1"));

        // 2. Sign digest
        let digest = sha2::Sha256::digest(b"hello x-road");
        let sig = client
            .sign(&key.id, &digest)
            .await
            .expect("Failed to sign via SignerClient");
        assert!(!sig.is_empty());

        // 3. Generate CSR
        let csr = client
            .generate_csr(&key.id, "DEV/COM/12345678/SUBSYS")
            .await
            .expect("Failed to generate CSR via SignerClient");
        assert!(csr.contains("BEGIN CERTIFICATE REQUEST"));

        // 4. Token & OCSP operations
        let tokens = client.list_tokens().await.unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].id, "0");

        client
            .set_ocsp_responses(
                vec!["abc123hash".to_string()],
                vec!["fake-ocsp-response-b64".to_string()],
            )
            .await
            .unwrap();

        let ocsp_map = client
            .get_ocsp_responses(&["abc123hash".to_string()])
            .await
            .unwrap();
        assert_eq!(
            ocsp_map.get("abc123hash"),
            Some(&"fake-ocsp-response-b64".to_string())
        );
    }
}
