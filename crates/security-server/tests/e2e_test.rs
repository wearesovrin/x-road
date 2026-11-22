use identifiers::{ClientId, ServiceId};
use poem::{
    Response, Route, Server, handler, http::StatusCode, listener::TcpListener, post,
    web::Json as PoemJson,
};
use security_server::{SecurityServerConfig, SecurityServerNode};
use serde_json::{Value, json};
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

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename = "SOAP-ENV:Envelope")]
struct MockSoapEnvelope {
    #[serde(rename = "@xmlns:SOAP-ENV")]
    xmlns: String,
    #[serde(rename = "SOAP-ENV:Body")]
    body: MockSoapBody,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct MockSoapBody {
    #[serde(rename = "getUserResponse")]
    response: MockGetUserResponse,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct MockGetUserResponse {
    result: String,
    #[serde(rename = "echoQuery")]
    echo_query: usize,
}

#[handler]
async fn mock_soap_backend(body: String) -> Response {
    let envelope = MockSoapEnvelope {
        xmlns: "http://schemas.xmlsoap.org/soap/envelope/".to_string(),
        body: MockSoapBody {
            response: MockGetUserResponse {
                result: "SUCCESS".to_string(),
                echo_query: body.len(),
            },
        },
    };
    let xml = quick_xml::se::to_string(&envelope).unwrap_or_default();
    let resp = format!("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n{xml}");
    Response::builder()
        .status(StatusCode::OK)
        .content_type("text/xml; charset=utf-8")
        .body(resp)
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
    let backend_app = Route::new()
        .at("/api/user-profile", post(mock_user_backend))
        .at("/soap/user-service", post(mock_soap_backend));
    tokio::spawn(async move {
        let _ = Server::new(TcpListener::bind(&backend_addr))
            .run(backend_app)
            .await;
    });

    // 2. Configure Security Server Node
    let provider = ClientId::subsystem_unchecked("DEV", "COM", "12345678", "PROVIDER");
    let consumer = ClientId::subsystem_unchecked("DEV", "COM", "87654321", "CONSUMER");
    let rogue = ClientId::subsystem_unchecked("DEV", "COM", "99999999", "ROGUE");

    let config = SecurityServerConfig {
        admin_bind: admin_bind.clone(),
        client_proxy_bind: client_proxy_bind.clone(),
        server_proxy_bind: server_proxy_bind.clone(),
        database_url: None,             // in-memory SQLite
        messagelog_database_url: None,  // in-memory SQLite
        default_server_proxy_url: None, // tests dynamic GlobalConf resolution!
    };

    let mut global_conf = globalconf::GlobalConf::new("DEV");
    let sec_server_id =
        identifiers::SecurityServerId::new("DEV", "COM", "12345678", "SS-TEST-NODE").unwrap();
    global_conf.register_security_server(
        sec_server_id.clone(),
        format!("127.0.0.1:{server_proxy_port}"),
    );
    global_conf.link_client_to_security_server(provider.clone(), sec_server_id);

    let server_conf = serverconf::DatabaseServerConf::connect_in_memory()
        .await
        .unwrap();
    let message_log = messagelog::MessageLogDb::connect_in_memory().await.unwrap();
    let node = SecurityServerNode::new(config, server_conf, message_log, global_conf);

    // 3. Register Clients, Services, and Access Rights in ServerConf
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
    node.server_conf
        .grant_access(&service, &consumer)
        .await
        .unwrap();

    let soap_backend_url = format!("http://127.0.0.1:{backend_port}/soap/user-service");
    node.server_conf
        .add_service(
            &provider,
            "getSoapUser",
            None,
            &soap_backend_url,
            ServiceType::Soap,
        )
        .await
        .unwrap();

    let soap_service = ServiceId::new(provider.clone(), "getSoapUser", None::<&str>).unwrap();
    node.server_conf
        .grant_access(&soap_service, &consumer)
        .await
        .unwrap();

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

    let resp_body: Value = resp
        .json()
        .await
        .expect("Failed to deserialize JSON response");
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
    let rest_gateway_url =
        format!("http://{client_proxy_bind}/r1/DEV/COM/12345678/PROVIDER/getUserProfile");
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
    node.server_conf
        .grant_access(&unknown_service, &consumer)
        .await
        .unwrap();

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

    // -------------------------------------------------------------------------
    // E2E Test 6: End-to-End SOAP Flow & Fault Handling via quick-xml serde
    // -------------------------------------------------------------------------
    let soap_request_envelope = message::SoapEnvelopeXml::new_request(
        consumer.clone(),
        soap_service.clone(),
        Some("<getSoapUser><id>123</id></getSoapUser>".to_string()),
    );
    let soap_xml = soap_request_envelope.to_xml().unwrap();

    let soap_resp = http_client
        .post(&client_proxy_url)
        .header("content-type", "text/xml; charset=utf-8")
        .body(soap_xml)
        .send()
        .await
        .expect("Failed to send SOAP request to Client Proxy");

    assert_eq!(soap_resp.status(), reqwest::StatusCode::OK);
    let soap_resp_xml = soap_resp.text().await.unwrap();
    assert!(soap_resp_xml.contains("getUserResponse"));
    assert!(soap_resp_xml.contains("SUCCESS"));

    // Verify SOAP transaction logged in MessageLogDb with discriminator "SOAP"
    let soap_query_id = soap_request_envelope.header.id;
    let soap_log_record = node
        .message_log
        .get_by_query_id(&soap_query_id)
        .await
        .unwrap()
        .expect("SOAP query was not logged in MessageLogDb!");
    assert_eq!(soap_log_record.discriminator, "SOAP");

    // Test unauthorized client sending SOAP query returns standard SOAP Fault XML
    let unauth_soap_envelope = message::SoapEnvelopeXml::new_request(
        rogue.clone(),
        soap_service.clone(),
        Some("<getSoapUser><id>999</id></getSoapUser>".to_string()),
    );
    let unauth_soap_resp = http_client
        .post(&client_proxy_url)
        .header("content-type", "text/xml; charset=utf-8")
        .body(unauth_soap_envelope.to_xml().unwrap())
        .send()
        .await
        .unwrap();

    assert_eq!(unauth_soap_resp.status(), reqwest::StatusCode::FORBIDDEN);
    let fault_xml = unauth_soap_resp.text().await.unwrap();
    let parsed_fault =
        message::SoapEnvelopeXml::from_xml(&fault_xml).expect("Fault must be parseable SOAP XML");
    assert!(parsed_fault.is_fault());
    let fault = parsed_fault.body.fault.unwrap();
    assert_eq!(fault.fault_code, "SOAP-ENV:Client");
    assert_eq!(fault.fault_string, "Client.AccessDenied");

    // -------------------------------------------------------------------------
    // E2E Test 7: Full Admin Management OpenAPI Parity with Java
    // -------------------------------------------------------------------------
    // A) Initialization & Status
    let init_status_url = format!("http://{admin_bind}/api/v1/initialization/status");
    let init_resp = http_client.get(&init_status_url).send().await.unwrap();
    assert_eq!(init_resp.status(), reqwest::StatusCode::OK);

    let init_url = format!("http://{admin_bind}/api/v1/initialization");
    let init_req = json!({
        "instance": "DEV",
        "member_class": "COM",
        "member_code": "12345678",
        "subsystem_code": "PROVIDER",
        "software_token_pin": "5678"
    });
    let init_post_resp = http_client
        .post(&init_url)
        .json(&init_req)
        .send()
        .await
        .unwrap();
    assert_eq!(init_post_resp.status(), reqwest::StatusCode::OK);

    // B) Client Enable / Disable & Details
    let provider_str = provider.to_string();
    let provider_encoded = provider_str.replace('/', "%2F");
    let client_get_url = format!("http://{admin_bind}/api/v1/clients/{provider_encoded}");
    let client_get_resp = http_client.get(&client_get_url).send().await.unwrap();
    assert_eq!(client_get_resp.status(), reqwest::StatusCode::OK);
    let client_dto: Value = client_get_resp.json().await.unwrap();
    assert_eq!(client_dto["member_code"], "12345678");

    let client_disable_url =
        format!("http://{admin_bind}/api/v1/clients/{provider_encoded}/disable");
    let disable_resp = http_client.put(&client_disable_url).send().await.unwrap();
    assert_eq!(disable_resp.status(), reqwest::StatusCode::OK);

    let client_enable_url = format!("http://{admin_bind}/api/v1/clients/{provider_encoded}/enable");
    let enable_resp = http_client.put(&client_enable_url).send().await.unwrap();
    assert_eq!(enable_resp.status(), reqwest::StatusCode::OK);

    // C) Service & Custom Endpoint Management
    let services_list = node
        .server_conf
        .list_services_for_client(&provider)
        .await
        .unwrap();
    assert!(!services_list.is_empty());
    let svc_id = services_list[0].id;

    let add_ep_url = format!("http://{admin_bind}/api/v1/services/{svc_id}/endpoints");
    let add_ep_req = json!({
        "method": "GET",
        "path": "/users/profile"
    });
    let add_ep_resp = http_client
        .post(&add_ep_url)
        .json(&add_ep_req)
        .send()
        .await
        .unwrap();
    assert_eq!(add_ep_resp.status(), reqwest::StatusCode::OK);
    let ep_dto: Value = add_ep_resp.json().await.unwrap();
    let ep_id = ep_dto["id"].as_i64().unwrap();
    assert_eq!(ep_dto["method"], "GET");
    assert_eq!(ep_dto["path"], "/users/profile");

    // Endpoint ACL: grant & list
    let ep_clients_url = format!("http://{admin_bind}/api/v1/endpoints/{ep_id}/service-clients");
    let grant_ep_req = json!({ "member_id": consumer.to_string() });
    let grant_ep_resp = http_client
        .post(&ep_clients_url)
        .json(&grant_ep_req)
        .send()
        .await
        .unwrap();
    assert_eq!(grant_ep_resp.status(), reqwest::StatusCode::OK);

    let list_ep_resp = http_client.get(&ep_clients_url).send().await.unwrap();
    assert_eq!(list_ep_resp.status(), reqwest::StatusCode::OK);
    let ep_client_list: Vec<String> = list_ep_resp.json().await.unwrap();
    assert!(ep_client_list.contains(&consumer.to_string()));

    // D) Local Groups & Membership
    let groups_url = format!("http://{admin_bind}/api/v1/clients/{provider_encoded}/local-groups");
    let create_group_req = json!({
        "group_code": "FINANCE-AUDITORS",
        "description": "Internal and external financial auditors"
    });
    let create_group_resp = http_client
        .post(&groups_url)
        .json(&create_group_req)
        .send()
        .await
        .unwrap();
    assert_eq!(create_group_resp.status(), reqwest::StatusCode::OK);
    let group_dto: Value = create_group_resp.json().await.unwrap();
    let group_id = group_dto["id"].as_i64().unwrap();
    assert_eq!(group_dto["group_code"], "FINANCE-AUDITORS");

    let group_members_url = format!("http://{admin_bind}/api/v1/local-groups/{group_id}/members");
    let add_member_req = json!({ "member_id": consumer.to_string() });
    let add_member_resp = http_client
        .post(&group_members_url)
        .json(&add_member_req)
        .send()
        .await
        .unwrap();
    assert_eq!(add_member_resp.status(), reqwest::StatusCode::OK);

    let list_members_resp = http_client.get(&group_members_url).send().await.unwrap();
    assert_eq!(list_members_resp.status(), reqwest::StatusCode::OK);
    let members_list: Vec<String> = list_members_resp.json().await.unwrap();
    assert!(members_list.contains(&consumer.to_string()));

    // E) Token & Keys & CSRs (Signer Engine)
    let tokens_url = format!("http://{admin_bind}/api/v1/tokens");
    let tokens_resp = http_client.get(&tokens_url).send().await.unwrap();
    assert_eq!(tokens_resp.status(), reqwest::StatusCode::OK);
    let tokens_list: Vec<Value> = tokens_resp.json().await.unwrap();
    assert!(!tokens_list.is_empty());
    assert_eq!(tokens_list[0]["token_type"], "SOFTWARE");

    let generate_key_url = format!("http://{admin_bind}/api/v1/tokens/0/keys");
    let gen_key_req = json!({ "label": "e2e-signing-key" });
    let gen_key_resp = http_client
        .post(&generate_key_url)
        .json(&gen_key_req)
        .send()
        .await
        .unwrap();
    assert_eq!(gen_key_resp.status(), reqwest::StatusCode::OK);
    let key_dto: Value = gen_key_resp.json().await.unwrap();
    let key_id = key_dto["id"].as_str().unwrap();
    assert_eq!(key_dto["label"], "e2e-signing-key");
    assert!(
        key_dto["public_key"]
            .as_str()
            .unwrap()
            .contains("BEGIN PUBLIC KEY")
    );

    let csr_url = format!("http://{admin_bind}/api/v1/keys/{key_id}/csrs");
    let csr_req = json!({ "subject_cn": "Security Server SS1 Signer" });
    let csr_resp = http_client
        .post(&csr_url)
        .json(&csr_req)
        .send()
        .await
        .unwrap();
    assert_eq!(csr_resp.status(), reqwest::StatusCode::OK);
    let csr_dto: Value = csr_resp.json().await.unwrap();
    assert!(
        csr_dto["csr_pem"]
            .as_str()
            .unwrap()
            .contains("BEGIN CERTIFICATE REQUEST")
    );

    // F) System Settings, Maintenance Mode & Diagnostics
    let version_url = format!("http://{admin_bind}/api/v1/system/version");
    let version_resp = http_client.get(&version_url).send().await.unwrap();
    assert_eq!(version_resp.status(), reqwest::StatusCode::OK);
    let version_body: Value = version_resp.json().await.unwrap();
    assert_eq!(version_body["edition"], "2024");

    let server_addr_url = format!("http://{admin_bind}/api/v1/system/server-address");
    let set_addr_req = json!({ "server_address": "https://ss.test.ee:8443" });
    let set_addr_resp = http_client
        .put(&server_addr_url)
        .json(&set_addr_req)
        .send()
        .await
        .unwrap();
    assert_eq!(set_addr_resp.status(), reqwest::StatusCode::OK);

    let get_addr_resp = http_client.get(&server_addr_url).send().await.unwrap();
    let get_addr_body: Value = get_addr_resp.json().await.unwrap();
    assert_eq!(get_addr_body["server_address"], "https://ss.test.ee:8443");

    let maint_url = format!("http://{admin_bind}/api/v1/system/maintenance-mode");
    let maint_enable_url = format!("http://{admin_bind}/api/v1/system/maintenance-mode/enable");
    let maint_resp = http_client.put(&maint_enable_url).send().await.unwrap();
    assert_eq!(maint_resp.status(), reqwest::StatusCode::OK);

    let maint_status_resp = http_client.get(&maint_url).send().await.unwrap();
    let maint_status_body: Value = maint_status_resp.json().await.unwrap();
    assert_eq!(maint_status_body["maintenance_mode"], true);

    let maint_disable_url = format!("http://{admin_bind}/api/v1/system/maintenance-mode/disable");
    let _ = http_client.put(&maint_disable_url).send().await.unwrap();

    let diag_globalconf_url = format!("http://{admin_bind}/api/v1/diagnostics/globalconf");
    let diag_resp = http_client.get(&diag_globalconf_url).send().await.unwrap();
    assert_eq!(diag_resp.status(), reqwest::StatusCode::OK);
    let diag_body: Value = diag_resp.json().await.unwrap();
    assert_eq!(diag_body["status"], "OK");

    // G) Backups Management
    let backups_url = format!("http://{admin_bind}/api/v1/backups");
    let create_backup_resp = http_client.post(&backups_url).send().await.unwrap();
    assert_eq!(create_backup_resp.status(), reqwest::StatusCode::OK);
    let backup_dto: Value = create_backup_resp.json().await.unwrap();
    let backup_file = backup_dto["filename"].as_str().unwrap();

    let list_backups_resp = http_client.get(&backups_url).send().await.unwrap();
    assert_eq!(list_backups_resp.status(), reqwest::StatusCode::OK);
    let backups_list: Vec<Value> = list_backups_resp.json().await.unwrap();
    assert_eq!(backups_list.len(), 1);
    assert_eq!(backups_list[0]["filename"], backup_file);

    let del_backup_url = format!("http://{admin_bind}/api/v1/backups/{backup_file}");
    let del_resp = http_client.delete(&del_backup_url).send().await.unwrap();
    assert_eq!(del_resp.status(), reqwest::StatusCode::OK);

    // -------------------------------------------------------------------------
    // E2E Test 8: Dynamic Global Configuration Address Resolution
    // -------------------------------------------------------------------------
    // Provider subsystem was linked to SS-TEST-NODE in GlobalConf at initialization.
    // Client Proxy dynamically resolves target ServerProxy via GlobalConf!
    let dynamic_resp = http_client
        .post(&client_proxy_url)
        .header("x-road-client", consumer.to_string())
        .header("x-road-service", service.to_string())
        .header("content-type", "application/json")
        .json(&json!({"query": "dynamic-resolution-test"}))
        .send()
        .await
        .expect("Failed to send dynamically resolved request");

    assert_eq!(dynamic_resp.status(), reqwest::StatusCode::OK);
    let dynamic_body: Value = dynamic_resp.json().await.unwrap();
    assert_eq!(dynamic_body["userId"], 1001);
    assert_eq!(dynamic_body["echo"]["query"], "dynamic-resolution-test");

    // -------------------------------------------------------------------------
    // 9. On-Demand MessageLog Archival & Verification
    // -------------------------------------------------------------------------
    let archive_resp = http_client
        .post(format!("http://{admin_bind}/api/v1/messagelog/archive"))
        .send()
        .await
        .expect("Failed to invoke messagelog archive endpoint");

    assert_eq!(archive_resp.status(), reqwest::StatusCode::OK);
    let archive_json: Value = archive_resp.json().await.unwrap();
    assert_eq!(archive_json["success"], true);
    let archived_count = archive_json["records_archived"].as_u64().unwrap();
    assert!(
        archived_count > 0,
        "Expected at least 1 record archived, got {}",
        archived_count
    );

    let file_path = archive_json["file_path"]
        .as_str()
        .expect("Expected file_path in archive response");
    assert!(
        std::path::Path::new(file_path).exists(),
        "Archive file {} must exist",
        file_path
    );

    // Verify cryptographic hash chain & ASiC-E containers in the generated archive
    let verify_result = messagelog::verify_archive(file_path, None).expect("verify_archive failed");
    assert!(verify_result.valid);
    assert_eq!(verify_result.records_verified as u64, archived_count);
    assert_eq!(
        verify_result.last_digest,
        archive_json["last_digest"].as_str().unwrap()
    );

    // -------------------------------------------------------------------------
    // 10. Metaservices Verification (listMethods and allowedMethods)
    // -------------------------------------------------------------------------
    // REST listMethods
    let list_methods_url = format!("http://{client_proxy_bind}/r1/{provider_str}/listMethods");
    let list_resp = http_client
        .get(&list_methods_url)
        .header("x-road-client", consumer.to_string())
        .send()
        .await
        .expect("Failed to call listMethods metaservice");

    assert_eq!(list_resp.status(), reqwest::StatusCode::OK);
    let list_json: Value = list_resp.json().await.unwrap();
    assert!(list_json.is_array());
    let methods: Vec<&str> = list_json
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|m| m["service_code"].as_str())
        .collect();
    assert!(
        methods.contains(&"getUserProfile"),
        "listMethods should contain getUserProfile"
    );
    assert!(
        methods.contains(&"getSoapUser"),
        "listMethods should contain getSoapUser"
    );

    // REST allowedMethods
    let allowed_methods_url =
        format!("http://{client_proxy_bind}/r1/{provider_str}/allowedMethods");
    let allowed_resp = http_client
        .get(&allowed_methods_url)
        .header("x-road-client", consumer.to_string())
        .send()
        .await
        .expect("Failed to call allowedMethods metaservice");

    assert_eq!(allowed_resp.status(), reqwest::StatusCode::OK);
    let allowed_json: Value = allowed_resp.json().await.unwrap();
    let allowed_methods: Vec<&str> = allowed_json
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|m| m["service_code"].as_str())
        .collect();
    assert!(allowed_methods.contains(&"getUserProfile"));

    // SOAP listMethods
    let list_methods_service =
        ServiceId::new(provider.clone(), "listMethods", None::<&str>).unwrap();
    let soap_list_envelope =
        message::SoapEnvelopeXml::new_request(consumer.clone(), list_methods_service, None);
    let soap_list_resp = http_client
        .post(&client_proxy_url)
        .header("content-type", "text/xml; charset=utf-8")
        .body(soap_list_envelope.to_xml().unwrap())
        .send()
        .await
        .expect("Failed to call SOAP listMethods");

    assert_eq!(soap_list_resp.status(), reqwest::StatusCode::OK);
    let soap_list_body = soap_list_resp.text().await.unwrap();
    assert!(soap_list_body.contains("listMethodsResponse"));
    assert!(soap_list_body.contains("getUserProfile"));
}
