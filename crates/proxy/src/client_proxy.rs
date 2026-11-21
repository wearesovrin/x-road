use common::headers;
use globalconf::GlobalConf;
use identifiers::{ClientId, ServiceId};
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

/// Shared application state for X-Road Client Proxy.
#[derive(Clone)]
pub struct ClientProxyState {
    pub server_conf: Arc<dyn ServerConfProvider>,
    pub message_log: Arc<MessageLogDb>,
    pub global_conf: Arc<GlobalConf>,
    pub http_client: reqwest::Client,
    pub default_server_proxy_url: Option<String>,
}

impl ClientProxyState {
    pub fn new(
        server_conf: Arc<dyn ServerConfProvider>,
        message_log: Arc<MessageLogDb>,
        global_conf: Arc<GlobalConf>,
    ) -> Self {
        Self {
            server_conf,
            message_log,
            global_conf,
            http_client: reqwest::Client::new(),
            default_server_proxy_url: None,
        }
    }

    pub fn with_default_server_proxy(mut self, url: impl Into<String>) -> Self {
        self.default_server_proxy_url = Some(url.into());
        self
    }
}

/// Parses X-Road headers from REST gateway path format `/r1/{instance}/{class}/{code}/{subsystem}/{service}`
fn parse_headers_from_rest_path(path: &str, header_map: &HashMap<String, String>) -> Option<XRoadHeaders> {
    let trimmed = path.strip_prefix("/r1/").or_else(|| path.strip_prefix("r1/"))?;
    let parts: Vec<&str> = trimmed.split('/').filter(|s| !s.is_empty()).collect();

    // We expect: [instance, class, code, (subsystem), service, (version)]
    let (client_target, service_code, service_version) = match parts.len() {
        4 => {
            let client = ClientId::member_unchecked(parts[0], parts[1], parts[2]);
            (client, parts[3], None)
        }
        5 => {
            let client = ClientId::subsystem_unchecked(parts[0], parts[1], parts[2], parts[3]);
            (client, parts[4], None)
        }
        6.. => {
            let client = ClientId::subsystem_unchecked(parts[0], parts[1], parts[2], parts[3]);
            (client, parts[4], Some(parts[5]))
        }
        _ => return None,
    };

    let target_service = ServiceId::new_unchecked(client_target, service_code, service_version);

    // Calling client should be in x-road-client header
    let caller_str = header_map
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(headers::X_ROAD_CLIENT))
        .map(|(_, v)| v.as_str())?;
    let caller_client: ClientId = caller_str.parse().ok()?;

    let query_id = header_map
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(headers::X_ROAD_ID))
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let mut xheaders = XRoadHeaders::new(caller_client, target_service);
    xheaders.id = query_id;
    Some(xheaders)
}

#[handler]
pub async fn handle_client_request(
    state: Data<&ClientProxyState>,
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
            let path = req.uri().path();
            if let Some(h) = parse_headers_from_rest_path(path, &header_map) {
                h
            } else if is_xml {
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

    // 1. Verify caller permission in local serverconf
    match state
        .server_conf
        .is_client_allowed(&parsed_headers.client)
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
                            Some("Client not authorized on this security server"),
                        )
                        .to_xml()
                        .unwrap_or_default(),
                    ));
            }
            return Ok(Response::builder()
                .status(StatusCode::FORBIDDEN)
                .content_type("application/json")
                .body(r#"{"faultCode":"Client.AccessDenied","faultString":"Client not authorized on this security server"}"#));
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

    let message_hash = format!("{:x}", Sha256::digest(&body_bytes));
    let query_id = parsed_headers.id.clone();

    tracing::info!(
        query_id = %query_id,
        client = %parsed_headers.client,
        service = %parsed_headers.service,
        payload_bytes = body_bytes.len(),
        "ClientProxy: Processing outgoing X-Road request"
    );

    // 2. Audit log outgoing request in messagelog
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

    // 3. Resolve destination Server Proxy URL
    let target_url = if let Some(direct) = header_map.get("x-road-server-proxy-url") {
        direct.clone()
    } else if let Some(addr) = state.global_conf.get_security_server_address_for_client(&parsed_headers.service.client) {
        if addr.starts_with("http://") || addr.starts_with("https://") {
            addr.clone()
        } else {
            format!("http://{addr}")
        }
    } else if let Some(ref def) = state.default_server_proxy_url {
        def.clone()
    } else {
        return Ok(Response::builder()
            .status(StatusCode::BAD_GATEWAY)
            .content_type("application/json")
            .body(r#"{"faultCode":"Server.ServerProxyNotFound","faultString":"Cannot resolve destination Server Proxy URL"}"#));
    };

    // 4. Forward to Server Proxy over HTTP
    let mut forward_req = state.http_client.request(req.method().clone(), &target_url);
    for (k, v) in parsed_headers.to_http_headers() {
        forward_req = forward_req.header(k, v);
    }
    if let Some(ct) = req.content_type() {
        forward_req = forward_req.header("content-type", ct);
    }
    if !body_bytes.is_empty() {
        forward_req = forward_req.body(body_bytes);
    }

    let upstream_resp = match forward_req.send().await {
        Ok(r) => r,
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .content_type("application/json")
                .body(format!(
                    r#"{{"faultCode":"Server.NetworkError","faultString":"Failed to communicate with Server Proxy: {e}"}}"#
                )));
        }
    };

    // 5. Capture upstream response and log in messagelog
    let status = StatusCode::from_u16(upstream_resp.status().as_u16())
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let resp_content_type = upstream_resp
        .headers()
        .get("content-type")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    let resp_bytes = upstream_resp.bytes().await.unwrap_or_default();
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

    // 6. Return response to the consumer Information System
    let mut builder = Response::builder().status(status);
    if let Some(ct) = resp_content_type {
        builder = builder.content_type(ct);
    }
    builder = builder.header(headers::X_ROAD_ID, query_id);
    Ok(builder.body(resp_bytes))
}

pub fn client_proxy_app(state: ClientProxyState) -> Route {
    Route::new()
        .at("/", handle_client_request.data(state.clone()))
        .at("/r1/*path", handle_client_request.data(state))
}

/// Launches the client proxy service on the given address.
pub async fn run_client_proxy(
    bind_addr: &str,
    state: ClientProxyState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Starting X-Road Client Proxy on {}...", bind_addr);
    let app = client_proxy_app(state);
    Server::new(TcpListener::bind(bind_addr)).run(app).await?;
    Ok(())
}
