use crate::server_proxy::{ServerProxyState, server_proxy_app};
use globalconf::GlobalConf;
use poem::{Endpoint, Request};
use rcgen::generate_simple_self_signed;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::server::WebPkiClientVerifier;
use rustls::{RootCertStore, ServerConfig};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

/// Generates a self-signed TLS server/client certificate and private key for X-Road proxies.
pub fn generate_self_signed_cert(
    common_name: &str,
) -> Result<
    (CertificateDer<'static>, PrivateKeyDer<'static>),
    Box<dyn std::error::Error + Send + Sync>,
> {
    let cert = generate_simple_self_signed(vec![common_name.to_string()])?;
    let cert_der = CertificateDer::from(cert.cert.der().to_vec());
    let key_der = PrivateKeyDer::Pkcs8(cert.signing_key.serialize_der().into());
    Ok((cert_der, key_der))
}

/// Builds an mTLS `ServerConfig` requiring peer certificates verified against GlobalConf CAs or self-signed roots.
pub fn create_server_mtls_config(
    cert: CertificateDer<'static>,
    key: PrivateKeyDer<'static>,
    global_conf: Option<&GlobalConf>,
) -> Result<ServerConfig, Box<dyn std::error::Error + Send + Sync>> {
    let mut root_store = RootCertStore::empty();

    // 1. Load approved CAs from GlobalConf
    if let Some(gconf) = global_conf {
        for ca in &gconf.approved_cas {
            if !ca.top_ca_cert.is_empty() {
                for c in CertificateDer::pem_slice_iter(&ca.top_ca_cert) {
                    if let Ok(c_der) = c {
                        let _ = root_store.add(c_der);
                    }
                }
            }
        }
    }

    // 2. If no external CAs present, add the server cert itself as a trusted root (for zero-config / test dev)
    if root_store.is_empty() {
        let _ = root_store.add(cert.clone());
    }

    let client_verifier = WebPkiClientVerifier::builder(Arc::new(root_store)).build()?;

    let config = ServerConfig::builder()
        .with_client_cert_verifier(client_verifier)
        .with_single_cert(vec![cert], key)?;

    Ok(config)
}

/// Runs the Server Proxy with native Mutual TLS (mTLS) listening on port 5500 (or custom bind address).
pub async fn run_server_proxy_mtls(
    bind_addr: &str,
    state: ServerProxyState,
    global_conf: Arc<GlobalConf>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!(
        "Starting X-Road Server Proxy with native mTLS on {} (X-Road Wire Port 5500)...",
        bind_addr
    );

    let (cert, key) = generate_self_signed_cert("xroad-server-proxy")?;
    let tls_config = create_server_mtls_config(cert, key, Some(&global_conf))?;

    let listener = TcpListener::bind(bind_addr).await?;
    let acceptor = TlsAcceptor::from(Arc::new(tls_config));
    let app = Arc::new(server_proxy_app(state));

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let acceptor = acceptor.clone();
        let app = app.clone();

        tokio::spawn(async move {
            match acceptor.accept(stream).await {
                Ok(tls_stream) => {
                    let (_, session) = tls_stream.get_ref();
                    // Extract client peer certificate SHA-256 fingerprint from mTLS session
                    let client_cert_fingerprint = session
                        .peer_certificates()
                        .and_then(|certs| certs.first())
                        .map(|c| format!("{:x}", Sha256::digest(c.as_ref())));

                    tracing::debug!(
                        peer = %peer_addr,
                        client_cert_hash = ?client_cert_fingerprint,
                        "ServerProxy mTLS: Handshake completed successfully"
                    );

                    // Process HTTP request on mTLS stream
                    let (mut reader, mut writer) = tokio::io::split(tls_stream);
                    let mut buf = vec![0u8; 65536];
                    if let Ok(n) = reader.read(&mut buf).await {
                        if n > 0 {
                            buf.truncate(n);
                            // Parse simple HTTP request from stream
                            let mut headers = [httparse::EMPTY_HEADER; 64];
                            let mut req = httparse::Request::new(&mut headers);
                            if let Ok(httparse::Status::Complete(body_offset)) = req.parse(&buf) {
                                let method = req.method.unwrap_or("POST");
                                let path = req.path.unwrap_or("/");
                                let body_bytes = &buf[body_offset..];

                                let mut poem_req = Request::builder()
                                    .method(method.parse().unwrap_or(poem::http::Method::POST))
                                    .uri(
                                        path.parse()
                                            .unwrap_or_else(|_| poem::http::Uri::from_static("/")),
                                    );

                                for h in req.headers.iter() {
                                    if let Ok(val) = std::str::from_utf8(h.value) {
                                        poem_req = poem_req.header(h.name, val);
                                    }
                                }

                                if let Some(ref fp) = client_cert_fingerprint {
                                    poem_req = poem_req.header("x-road-auth-cert", fp.as_str());
                                    poem_req = poem_req.header("x-road-client-cert", fp.as_str());
                                }

                                let poem_req = poem_req.body(body_bytes.to_vec());
                                if let Ok(resp) = app.call(poem_req).await {
                                    let status = resp.status().as_u16();
                                    let status_text =
                                        resp.status().canonical_reason().unwrap_or("OK");
                                    let body =
                                        resp.into_body().into_bytes().await.unwrap_or_default();

                                    let http_resp = format!(
                                        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                                        status,
                                        status_text,
                                        body.len()
                                    );
                                    let _ = writer.write_all(http_resp.as_bytes()).await;
                                    let _ = writer.write_all(&body).await;
                                    let _ = writer.flush().await;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(peer = %peer_addr, "ServerProxy mTLS: Handshake failed: {}", e);
                }
            }
        });
    }
}
