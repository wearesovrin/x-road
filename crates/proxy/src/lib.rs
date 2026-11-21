use axum::{
    Router,
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
};
use identifiers::ClientId;
use message::XRoadHeaders;
use serverconf::ServerConfProvider;
use std::collections::HashMap;
use std::sync::Arc;

/// Shared application state for X-Road Proxy.
#[derive(Clone)]
pub struct ProxyState {
    pub server_conf: Arc<dyn ServerConfProvider>,
    pub client_id: ClientId,
}

/// Client Proxy router handling inbound requests from local Information Systems.
pub fn client_proxy_router(state: ProxyState) -> Router {
    Router::new()
        .route("/", post(handle_client_request))
        .with_state(state)
}

async fn handle_client_request(
    State(state): State<ProxyState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let mut header_map = HashMap::new();
    for (k, v) in headers.iter() {
        if let Ok(v_str) = v.to_str() {
            header_map.insert(k.as_str().to_string(), v_str.to_string());
        }
    }

    let parsed_headers = match XRoadHeaders::from_http_headers(&header_map) {
        Ok(h) => h,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, format!("Protocol error: {}", e)).into_response();
        }
    };

    // Verify client permission in local serverconf
    match state
        .server_conf
        .is_client_allowed(&parsed_headers.client)
        .await
    {
        Ok(true) => {}
        Ok(false) => return (StatusCode::FORBIDDEN, "Client not authorized").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }

    tracing::info!(
        query_id = %parsed_headers.id,
        client = %parsed_headers.client,
        service = %parsed_headers.service,
        payload_bytes = body.len(),
        "Processing X-Road request"
    );

    (StatusCode::OK, "X-Road Response OK").into_response()
}

/// Launches the client proxy service on the given address.
pub async fn run_client_proxy(
    bind_addr: &str,
    client_id: ClientId,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!(
        "Starting X-Road Proxy Service on {} (Client Proxy)...",
        bind_addr
    );
    let mut server_conf = serverconf::InMemoryServerConf::default();
    server_conf.allowed_clients.insert(client_id.clone());

    let state = ProxyState {
        server_conf: Arc::new(server_conf),
        client_id,
    };

    let app = client_proxy_router(state);
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
