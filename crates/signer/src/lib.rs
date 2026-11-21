use common::{XRoadError, XRoadResult};
use proto::key_service_server::{KeyService, KeyServiceServer};
use proto::sign_service_server::{SignService, SignServiceServer};
use proto::*;
use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair, PKCS_ECDSA_P256_SHA256};
use ring::rand::SystemRandom;
use ring::signature::{EcdsaKeyPair, ECDSA_P256_SHA256_ASN1_SIGNING};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

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

/// Software-token backed signer service implementation for managing keys, CSRs, and digital signatures.
#[derive(Clone, Default)]
pub struct SignerServiceImpl {
    keys: Arc<RwLock<HashMap<String, SoftwareKeyEntry>>>,
}

impl SignerServiceImpl {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_key_service(self: Arc<Self>) -> KeyServiceServer<SignerKeyServiceAdapter> {
        KeyServiceServer::new(SignerKeyServiceAdapter { inner: self })
    }

    pub fn into_sign_service(self: Arc<Self>) -> SignServiceServer<SignerSignServiceAdapter> {
        SignServiceServer::new(SignerSignServiceAdapter { inner: self })
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
    pub async fn generate_csr(
        &self,
        key_id: &str,
        subject_cn: &str,
    ) -> XRoadResult<String> {
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

        Ok(csr_pem)
    }

    /// Digitally signs a pre-computed digest with the given key ID.
    pub async fn sign_digest(&self, key_id: &str, digest: &[u8]) -> XRoadResult<Vec<u8>> {
        let keys_guard = self.keys.read().await;
        let entry = keys_guard
            .get(key_id)
            .ok_or_else(|| XRoadError::Crypto(format!("Key not found: {key_id}")))?;

        let rng = SystemRandom::new();
        let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, &entry.key_der, &rng)
            .map_err(|e| XRoadError::Crypto(format!("Failed to load ring ECDSA key: {e}")))?;

        let signature = key_pair
            .sign(&rng, digest)
            .map_err(|e| XRoadError::Crypto(format!("Failed to sign digest: {e}")))?;

        Ok(signature.as_ref().to_vec())
    }

    /// Stores a certificate associated with a key ID.
    pub async fn add_certificate(
        &self,
        key_id: &str,
        cert_bytes: Vec<u8>,
    ) -> XRoadResult<()> {
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

/// Launches the Signer gRPC service on the given address.
pub async fn run_signer(bind_addr: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Starting X-Road Signer Daemon on {}...", bind_addr);
    let addr = bind_addr.parse()?;
    let signer = Arc::new(SignerServiceImpl::new());

    tonic::transport::Server::builder()
        .add_service(signer.clone().into_key_service())
        .add_service(signer.into_sign_service())
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
            .generate_software_key("soft-token-0", "test-auth-key", KeyUsageInfo::Authentication)
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
        let keys = signer.keys.read().await;
        let entry = keys.get(&key_info.id).unwrap();
        let key_pair = KeyPair::from_pem(&entry.key_pem).unwrap();
        let public_key_der = key_pair.public_key_raw();

        let peer_public_key = ring::signature::UnparsedPublicKey::new(
            &ring::signature::ECDSA_P256_SHA256_ASN1,
            public_key_der,
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
}
