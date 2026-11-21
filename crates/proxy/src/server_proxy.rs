use common::headers;
use message::{SoapEnvelopeXml, XRoadHeaders};
use messagelog::{MessageLogDb, NewLogRecord};
use poem::{
    EndpointExt, Request, Response, Route, handler,
    http::StatusCode,
    listener::TcpListener,
    web::Data,
    Server,
};
use serverconf::ServerConfProvider;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;

/// Shared application state for X-Road Server Proxy.
#[derive(Clone)]
pub struct ServerProxyState {
    pub server_conf: Arc<dyn ServerConfProvider>,
    pub message_log: Arc<MessageLogDb>,
    pub http_client: reqwest::Client,
}

impl ServerProxyState {
    pub fn new(
        server_conf: Arc<dyn ServerConfProvider>,
        message_log: Arc<MessageLogDb>,
    ) -> Self {
        Self {
            server_conf,
            message_log,
            http_client: reqwest::Client::new(),
        }
    }
}

#[handler]
pub async fn handle_server_request(
    state: Data<&ServerProxyState>,
    req: &Request,
    body: poem::Body,
) -> poem::Result<Response> {
    let mut header_map = HashMap::new();
    for (k, v) in req.headers().iter() {
        if let Ok(v_str) = v.to_str() {
            header_map.insert(k.as_str().to_string(), v_str.to_string());
        }
    }

    let body_bytes = body.into_bytes().await.map_err(poem::error::BadRequest)?;
    let content_type = req
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let is_xml = content_type.contains("xml")
        || body_bytes.starts_with(b"<?xml")
        || body_bytes.starts_with(b"<SOAP-ENV:")
        || body_bytes.starts_with(b"<soapenv:");

    let parsed_headers = match XRoadHeaders::from_http_headers(&header_map) {
        Ok(h) => h,
        Err(e) => {
            if is_xml {
                if let Ok(xml_str) = std::str::from_utf8(&body_bytes) {
                    if let Ok(soap_envelope) = SoapEnvelopeXml::from_xml(xml_str) {
                        match soap_envelope.to_xroad_headers() {
                            Ok(h) => h,
                            Err(e2) => {
                                return Ok(Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .content_type("text/xml; charset=utf-8")
                                    .body(
                                        SoapEnvelopeXml::new_fault(
                                            None,
                                            "SOAP-ENV:Client",
                                            "Client.InvalidHeaders",
                                            Some(&e2.to_string()),
                                        )
                                        .to_xml()
                                        .unwrap_or_default(),
                                    ));
                            }
                        }
                    } else {
                        return Ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .content_type("text/xml; charset=utf-8")
                            .body(
                                SoapEnvelopeXml::new_fault(
                                    None,
                                    "SOAP-ENV:Client",
                                    "Client.InvalidMessage",
                                    Some("Malformed SOAP XML message"),
                                )
                                .to_xml()
                                .unwrap_or_default(),
                            ));
                    }
                } else {
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .content_type("text/xml; charset=utf-8")
                        .body(
                            SoapEnvelopeXml::new_fault(
                                None,
                                "SOAP-ENV:Client",
                                "Client.InvalidMessage",
                                Some("Invalid UTF-8 payload"),
                            )
                            .to_xml()
                            .unwrap_or_default(),
                        ));
                }
            } else {
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .content_type("application/json")
                    .body(format!(
                        r#"{{"faultCode":"Client.InvalidHeaders","faultString":"Protocol error: {e}"}}"#
                    )));
            }
        }
    };

    let query_id = parsed_headers.id.clone();

    tracing::info!(
        query_id = %query_id,
        client = %parsed_headers.client,
        service = %parsed_headers.service,
        "ServerProxy: Incoming request received"
    );

    // 1. Verify access permissions for client calling service
    match state
        .server_conf
        .is_service_allowed(&parsed_headers.client, &parsed_headers.service)
        .await
    {
        Ok(true) => {}
        Ok(false) => {
            if is_xml {
                return Ok(Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .content_type("text/xml; charset=utf-8")
                    .body(
                        SoapEnvelopeXml::new_fault(
                            Some(&parsed_headers),
                            "SOAP-ENV:Client",
                            "Client.AccessDenied",
                            Some("Access to requested service denied"),
                        )
                        .to_xml()
                        .unwrap_or_default(),
                    ));
            }
            return Ok(Response::builder()
                .status(StatusCode::FORBIDDEN)
                .content_type("application/json")
                .body(r#"{"faultCode":"Client.AccessDenied","faultString":"Access to requested service denied"}"#));
        }
        Err(e) => {
            if is_xml {
                return Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("text/xml; charset=utf-8")
                    .body(
                        SoapEnvelopeXml::new_fault(
                            Some(&parsed_headers),
                            "SOAP-ENV:Server",
                            "Server.InternalError",
                            Some(&e.to_string()),
                        )
                        .to_xml()
                        .unwrap_or_default(),
                    ));
            }
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .content_type("application/json")
                .body(format!(
                    r#"{{"faultCode":"Server.InternalError","faultString":"{e}"}}"#
                )));
        }
    }

    // 2. Resolve internal backend URL for this service
    let service_url = match state.server_conf.get_service_url(&parsed_headers.service).await {
        Ok(Some(url)) => url,
        Ok(None) => {
            if is_xml {
                return Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .content_type("text/xml; charset=utf-8")
                    .body(
                        SoapEnvelopeXml::new_fault(
                            Some(&parsed_headers),
                            "SOAP-ENV:Server",
                            "Server.ServerProxy.ServiceNotFound",
                            Some("Service not configured on this security server"),
                        )
                        .to_xml()
                        .unwrap_or_default(),
                    ));
            }
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .content_type("application/json")
                .body(r#"{"faultCode":"Client.ServiceNotFound","faultString":"Service not configured on this security server"}"#));
        }
        Err(e) => {
            if is_xml {
                return Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("text/xml; charset=utf-8")
                    .body(
                        SoapEnvelopeXml::new_fault(
                            Some(&parsed_headers),
                            "SOAP-ENV:Server",
                            "Server.InternalError",
                            Some(&e.to_string()),
                        )
                        .to_xml()
                        .unwrap_or_default(),
                    ));
            }
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .content_type("application/json")
                .body(format!(
                    r#"{{"faultCode":"Server.InternalError","faultString":"{e}"}}"#
                )));
        }
    };

    let message_hash = format!("{:x}", Sha256::digest(&body_bytes));

    // 3. Audit log request in messagelog
    let _ = state
        .message_log
        .log_message(NewLogRecord {
            query_id: query_id.clone(),
            x_request_id: header_map.get("x-request-id").cloned(),
            client_id_str: Some(parsed_headers.client.to_string()),
            service_id_str: Some(parsed_headers.service.to_string()),
            discriminator: if is_xml { "SOAP".to_string() } else { "REST".to_string() },
            message_hash,
            signature_hash: None,
            message_body: String::from_utf8(body_bytes.to_vec()).ok(),
            signature_data: None,
            response: false,
        })
        .await;

    // 4. Forward to internal backend service
    let mut backend_req = state.http_client.request(req.method().clone(), &service_url);
    if let Some(ct) = req.content_type() {
        backend_req = backend_req.header("content-type", ct);
    }
    for (k, v) in req.headers() {
        let name = k.as_str();
        if !name.starts_with("x-road-") && name != "host" {
            backend_req = backend_req.header(name, v.as_bytes());
        }
    }
    if !body_bytes.is_empty() {
        backend_req = backend_req.body(body_bytes);
    }

    let backend_resp = match backend_req.send().await {
        Ok(r) => r,
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .content_type("application/json")
                .body(format!(
                    r#"{{"faultCode":"Server.BackendServiceError","faultString":"Failed to contact backend service: {e}"}}"#
                )));
        }
    };

    // 5. Capture backend response and log in messagelog
    let status = StatusCode::from_u16(backend_resp.status().as_u16())
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let resp_content_type = backend_resp
        .headers()
        .get("content-type")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    let resp_bytes = backend_resp.bytes().await.unwrap_or_default();
    let resp_hash = format!("{:x}", Sha256::digest(&resp_bytes));

    let _ = state
        .message_log
        .log_message(NewLogRecord {
            query_id: query_id.clone(),
            x_request_id: header_map.get("x-request-id").cloned(),
            client_id_str: Some(parsed_headers.client.to_string()),
            service_id_str: Some(parsed_headers.service.to_string()),
            discriminator: if is_xml { "SOAP".to_string() } else { "REST".to_string() },
            message_hash: resp_hash,
            signature_hash: None,
            message_body: String::from_utf8(resp_bytes.to_vec()).ok(),
            signature_data: None,
            response: true,
        })
        .await;

    // 6. Return response to calling Security Server
    let mut builder = Response::builder().status(status);
    if let Some(ct) = resp_content_type {
        builder = builder.content_type(ct);
    }
    builder = builder.header(headers::X_ROAD_ID, query_id);
    Ok(builder.body(resp_bytes))
}

pub fn server_proxy_app(state: ServerProxyState) -> Route {
    Route::new()
        .at("/*path", handle_server_request.data(state.clone()))
        .at("/", handle_server_request.data(state))
}

/// Launches the server proxy service on the given address.
pub async fn run_server_proxy(
    bind_addr: &str,
    state: ServerProxyState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Starting X-Road Server Proxy on {}...", bind_addr);
    let app = server_proxy_app(state);
    Server::new(TcpListener::bind(bind_addr)).run(app).await?;
    Ok(())
}
