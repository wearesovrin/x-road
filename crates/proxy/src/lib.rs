use identifiers::ClientId;
use message::XRoadHeaders;
use poem::{
    EndpointExt, Request, Response, Route, Server, handler,
    http::StatusCode,
    listener::TcpListener,
    web::Data,
};
use serverconf::ServerConfProvider;
use std::collections::HashMap;
use std::sync::Arc;

/// Shared application state for X-Road Proxy.
#[derive(Clone)]
pub struct ProxyState {
    pub server_conf: Arc<dyn ServerConfProvider>,
    pub client_id: ClientId,
}

#[handler]
async fn handle_client_request(
    state: Data<&ProxyState>,
    req: &Request,
    body: poem::Body,
) -> poem::Result<Response> {
    let mut header_map = HashMap::new();
    for (k, v) in req.headers().iter() {
        if let Ok(v_str) = v.to_str() {
            header_map.insert(k.as_str().to_string(), v_str.to_string());
        }
    }

    let parsed_headers = match XRoadHeaders::from_http_headers(&header_map) {
        Ok(h) => h,
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(format!("Protocol error: {}", e)));
        }
    };

    // Verify client permission in local serverconf
    match state
        .server_conf
        .is_client_allowed(&parsed_headers.client)
        .await
    {
        Ok(true) => {}
        Ok(false) => {
            return Ok(Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body("Client not authorized"));
        }
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(e.to_string()));
        }
    }

    let body_bytes = body.into_bytes().await.map_err(poem::error::BadRequest)?;

    tracing::info!(
        query_id = %parsed_headers.id,
        client = %parsed_headers.client,
        service = %parsed_headers.service,
        payload_bytes = body_bytes.len(),
        "Processing X-Road request"
    );

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body("X-Road Response OK"))
}

pub fn client_proxy_app(state: ProxyState) -> Route {
    Route::new().at("/", poem::post(handle_client_request).data(state))
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

    let app = client_proxy_app(state);
    Server::new(TcpListener::bind(bind_addr)).run(app).await?;
    Ok(())
}
