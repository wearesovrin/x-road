use proto::key_service_server::{KeyService, KeyServiceServer};
use proto::sign_service_server::{SignService, SignServiceServer};
use proto::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

/// In-memory signer service implementation for managing keys and signing digests.
#[derive(Debug, Default)]
pub struct SignerServiceImpl {
    keys: Arc<RwLock<HashMap<String, Vec<u8>>>>,
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
        let key_id = format!("{}-key", r.token_id);
        let key_info = KeyInfoProto {
            id: key_id.clone(),
            friendly_name: Some(r.key_label),
            usage: 0,
            available: true,
            label: Some("Generated Software Key".to_string()),
            public_key: String::new(),
            certs: vec![],
            cert_requests: vec![],
            sign_mechanism_name: "SHA256withRSA".to_string(),
        };
        self.inner.keys.write().await.insert(key_id, vec![1, 2, 3]);
        Ok(Response::new(key_info))
    }

    async fn set_key_friendly_name(
        &self,
        _req: Request<SetKeyFriendlyNameReq>,
    ) -> Result<Response<proto::Empty>, Status> {
        Ok(Response::new(proto::Empty {}))
    }

    async fn get_key_id_for_cert_hash(
        &self,
        _req: Request<GetKeyIdForCertHashReq>,
    ) -> Result<Response<GetKeyIdForCertHashResp>, Status> {
        Ok(Response::new(GetKeyIdForCertHashResp {
            key_id: "default-key".to_string(),
            sign_mechanism_name: "SHA256withRSA".to_string(),
        }))
    }

    async fn get_sign_mechanism(
        &self,
        _req: Request<GetSignMechanismReq>,
    ) -> Result<Response<GetSignMechanismResp>, Status> {
        Ok(Response::new(GetSignMechanismResp {
            sign_mechanism_name: "SHA256withRSA".to_string(),
        }))
    }

    async fn delete_key(
        &self,
        req: Request<DeleteKeyReq>,
    ) -> Result<Response<proto::Empty>, Status> {
        let r = req.into_inner();
        self.inner.keys.write().await.remove(&r.key_id);
        Ok(Response::new(proto::Empty {}))
    }

    async fn get_auth_key(
        &self,
        _req: Request<GetAuthKeyReq>,
    ) -> Result<Response<AuthKeyProto>, Status> {
        Ok(Response::new(AuthKeyProto {
            alias: "auth-key".to_string(),
            key_store: vec![],
            password: "empty".to_string(),
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
        Ok(Response::new(ListSoftwareTokenKeysResp { keys: vec![] }))
    }
}

pub struct SignerSignServiceAdapter {
    inner: Arc<SignerServiceImpl>,
}

#[tonic::async_trait]
impl SignService for SignerSignServiceAdapter {
    async fn sign(&self, req: Request<SignReq>) -> Result<Response<SignResp>, Status> {
        let r = req.into_inner();
        let mock_sig = format!("sig-for-digest-len-{}", r.digest.len()).into_bytes();
        Ok(Response::new(SignResp {
            signature: mock_sig,
        }))
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
