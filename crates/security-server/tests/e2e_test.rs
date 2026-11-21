use identifiers::{ClientId, ServiceId};
use poem::{
    handler, http::StatusCode, listener::TcpListener, post, web::Json as PoemJson, Route, Server,
};
use security_server::{SecurityServerConfig, SecurityServerNode};
use serde_json::{json, Value};
use serverconf::ServiceType;
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;

static PORT_COUNTER: AtomicU16 = AtomicU16::new(19100);

fn next_port() -> u16 {
    PORT_COUNTER.fetch_add(10, Ordering::SeqCst)
}

#[handler]
async fn mock_user_backend(body: PoemJson<Value>) -> (StatusCode, PoemJson<Value>) {
    let req_val = body.0;
    let resp = json!({
        "userId": 1001,
        "displayName": "X-Road Test User",
        "status": "ACTIVE",
        "echo": req_val
    });
    (StatusCode::OK, PoemJson(resp))
}

#[tokio::test]
async fn test_security_server_e2e_flow() {
    let base_port = next_port();
    let backend_port = base_port;
    let admin_port = base_port + 1;
    let client_proxy_port = base_port + 2;
    let server_proxy_port = base_port + 3;

    let backend_addr = format!("127.0.0.1:{backend_port}");
    let admin_bind = format!("127.0.0.1:{admin_port}");
    let client_proxy_bind = format!("127.0.0.1:{client_proxy_port}");
    let server_proxy_bind = format!("127.0.0.1:{server_proxy_port}");

    // 1. Start mock backend service
    let backend_app = Route::new().at("/api/user-profile", post(mock_user_backend));
    tokio::spawn(async move {
        let _ = Server::new(TcpListener::bind(&backend_addr))
            .run(backend_app)
            .await;
    });

    // 2. Configure Security Server Node
    let config = SecurityServerConfig {
        admin_bind: admin_bind.clone(),
        client_proxy_bind: client_proxy_bind.clone(),
        server_proxy_bind: server_proxy_bind.clone(),
        database_url: None, // in-memory SQLite
        messagelog_database_url: None, // in-memory SQLite
        default_server_proxy_url: Some(format!("http://{server_proxy_bind}")),
    };

    let node = SecurityServerNode::new_in_memory(config)
        .await
        .expect("Failed to create in-memory Security Server node");

    // 3. Register Clients, Services, and Access Rights in ServerConf
    let provider = ClientId::subsystem_unchecked("DEV", "COM", "12345678", "PROVIDER");
    let consumer = ClientId::subsystem_unchecked("DEV", "COM", "87654321", "CONSUMER");
    let rogue = ClientId::subsystem_unchecked("DEV", "COM", "99999999", "ROGUE");

    node.server_conf.add_client(&provider, false).await.unwrap();
    node.server_conf.add_client(&consumer, false).await.unwrap();

    let backend_url = format!("http://127.0.0.1:{backend_port}/api/user-profile");
    node.server_conf
        .add_service(
            &provider,
            "getUserProfile",
            Some("v1"),
            &backend_url,
            ServiceType::Rest,
        )
        .await
        .unwrap();

    let service = ServiceId::new(provider.clone(), "getUserProfile", Some("v1")).unwrap();
    node.server_conf.grant_access(&service, &consumer).await.unwrap();

    // 4. Launch Security Server Node in background
    let node_clone = node.clone();
    tokio::spawn(async move {
        let _ = node_clone.run().await;
    });

    // Allow servers to bind
    tokio::time::sleep(Duration::from_millis(200)).await;

    let http_client = reqwest::Client::new();

    // -------------------------------------------------------------------------
    // E2E Test 1: Successful Consumer Request through Client Proxy -> Server Proxy -> Backend
    // -------------------------------------------------------------------------
    let client_proxy_url = format!("http://{client_proxy_bind}/");
    let request_payload = json!({"requestId": "test-req-001"});

    let resp = http_client
        .post(&client_proxy_url)
        .header("x-road-client", consumer.to_string())
        .header("x-road-service", service.to_string())
        .header("content-type", "application/json")
        .json(&request_payload)
        .send()
        .await
        .expect("Failed to send request to Client Proxy");

    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let x_road_id = resp
        .headers()
        .get("x-road-id")
        .expect("Missing x-road-id header in response")
        .to_str()
        .unwrap()
        .to_string();
    assert!(!x_road_id.is_empty());

    let resp_body: Value = resp.json().await.expect("Failed to deserialize JSON response");
    assert_eq!(resp_body["userId"], 1001);
    assert_eq!(resp_body["displayName"], "X-Road Test User");
    assert_eq!(resp_body["status"], "ACTIVE");
    assert_eq!(resp_body["echo"]["requestId"], "test-req-001");

    // Verify non-repudiation log in MessageLogDb
    let log_record = node
        .message_log
        .get_by_query_id(&x_road_id)
        .await
        .unwrap()
        .expect("Transaction was not logged in MessageLogDb!");
    assert_eq!(log_record.query_id, x_road_id);
    assert_eq!(log_record.client_id_str, Some(consumer.to_string()));
    assert_eq!(log_record.service_id_str, Some(service.to_string()));

    // -------------------------------------------------------------------------
    // E2E Test 2: REST Gateway Path Convention (`/r1/{instance}/{class}/{code}/{subsystem}/{service}`)
    // -------------------------------------------------------------------------
    let rest_gateway_url = format!("http://{client_proxy_bind}/r1/DEV/COM/12345678/PROVIDER/getUserProfile");
    let resp2 = http_client
        .post(&rest_gateway_url)
        .header("x-road-client", consumer.to_string())
        .header("content-type", "application/json")
        .json(&json!({"requestId": "rest-gateway-002"}))
        .send()
        .await
        .expect("Failed to send REST gateway request");

    assert_eq!(resp2.status(), reqwest::StatusCode::OK);
    let resp2_body: Value = resp2.json().await.unwrap();
    assert_eq!(resp2_body["userId"], 1001);
    assert_eq!(resp2_body["echo"]["requestId"], "rest-gateway-002");

    // -------------------------------------------------------------------------
    // E2E Test 3: Unauthorized Client -> Access Denied (403 Forbidden)
    // -------------------------------------------------------------------------
    // A) Rogue client calling through Client Proxy (Client Proxy rejects unauthorized local client)
    let resp_unauth_client = http_client
        .post(&client_proxy_url)
        .header("x-road-client", rogue.to_string())
        .header("x-road-service", service.to_string())
        .json(&request_payload)
        .send()
        .await
        .unwrap();

    assert_eq!(resp_unauth_client.status(), reqwest::StatusCode::FORBIDDEN);

    // B) Foreign client without access permission reaching Server Proxy directly
    let server_proxy_url = format!("http://{server_proxy_bind}/");
    let foreign_unauth_client = ClientId::subsystem_unchecked("DEV", "COM", "77777777", "FOREIGN");
    let resp_denied = http_client
        .post(&server_proxy_url)
        .header("x-road-client", foreign_unauth_client.to_string())
        .header("x-road-service", service.to_string())
        .header("x-road-id", "unauth-query-id")
        .json(&request_payload)
        .send()
        .await
        .unwrap();

    assert_eq!(resp_denied.status(), reqwest::StatusCode::FORBIDDEN);
    let denied_body: Value = resp_denied.json().await.unwrap();
    assert_eq!(denied_body["faultCode"], "Client.AccessDenied");

    // -------------------------------------------------------------------------
    // E2E Test 4: Unknown Service -> Not Found (404 Not Found)
    // -------------------------------------------------------------------------
    let unknown_service = ServiceId::new(provider.clone(), "secretDatabase", None::<&str>).unwrap();
    // Grant access to non-existent backend to test service resolution
    node.server_conf.grant_access(&unknown_service, &consumer).await.unwrap();

    let resp_not_found = http_client
        .post(&server_proxy_url)
        .header("x-road-client", consumer.to_string())
        .header("x-road-service", unknown_service.to_string())
        .header("x-road-id", "unknown-service-query-id")
        .json(&request_payload)
        .send()
        .await
        .unwrap();

    assert_eq!(resp_not_found.status(), reqwest::StatusCode::NOT_FOUND);
    let not_found_body: Value = resp_not_found.json().await.unwrap();
    assert_eq!(not_found_body["faultCode"], "Client.ServiceNotFound");

    // -------------------------------------------------------------------------
    // E2E Test 5: Admin REST Management API
    // -------------------------------------------------------------------------
    let admin_status_url = format!("http://{admin_bind}/api/v1/status");
    let status_resp = http_client.get(&admin_status_url).send().await.unwrap();
    assert_eq!(status_resp.status(), reqwest::StatusCode::OK);
    let status_body: Value = status_resp.json().await.unwrap();
    assert_eq!(status_body["status"], "OK");
    assert_eq!(status_body["version"], "0.1.0");
    assert_eq!(status_body["client_count"], 2);

    let admin_clients_url = format!("http://{admin_bind}/api/v1/clients");
    let clients_resp = http_client.get(&admin_clients_url).send().await.unwrap();
    assert_eq!(clients_resp.status(), reqwest::StatusCode::OK);
    let clients_list: Vec<Value> = clients_resp.json().await.unwrap();
    assert_eq!(clients_list.len(), 2);

    // Verify recent message logs through Admin API
    let admin_logs_url = format!("http://{admin_bind}/api/v1/messagelog/recent");
    let logs_resp = http_client.get(&admin_logs_url).send().await.unwrap();
    assert_eq!(logs_resp.status(), reqwest::StatusCode::OK);
    let logs_list: Vec<Value> = logs_resp.json().await.unwrap();
    assert!(!logs_list.is_empty());
}
