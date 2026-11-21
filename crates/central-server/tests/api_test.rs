use central_server::{central_server_app, CentralServerDb};
use poem::{
    listener::{Acceptor, Listener, TcpListener},
    Server,
};
use serde_json::{json, Value};
use std::time::Duration;

#[tokio::test]
async fn test_central_server_api_e2e() {
    let db = CentralServerDb::connect_in_memory()
        .await
        .expect("Failed to connect to in-memory central server db");

    let listener = TcpListener::bind("127.0.0.1:0");
    let acceptor = listener.into_acceptor().await.unwrap();
    let port = acceptor.local_addr()[0].as_socket_addr().unwrap().port();

    let app = central_server_app("DEV".to_string(), db);
    tokio::spawn(async move {
        let _ = Server::new_with_acceptor(acceptor).run(app).await;
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let http_client = reqwest::Client::new();
    let base_url = format!("http://127.0.0.1:{port}");

    // 1. GET /api/v1/status
    let status_resp = http_client
        .get(format!("{base_url}/api/v1/status"))
        .send()
        .await
        .unwrap();
    assert_eq!(status_resp.status(), reqwest::StatusCode::OK);
    let status_val: Value = status_resp.json().await.unwrap();
    assert_eq!(status_val["instance_identifier"], "DEV");
    assert_eq!(status_val["status"], "READY");
    assert_eq!(status_val["member_count"], 0);

    // 2. POST /api/v1/members
    let add_member_req = json!({
        "member_class": "COM",
        "member_code": "11223344",
        "name": "Nordic Solutions Ltd",
        "administrative_contact": "admin@nordic.fi"
    });
    let add_resp = http_client
        .post(format!("{base_url}/api/v1/members"))
        .json(&add_member_req)
        .send()
        .await
        .unwrap();
    assert_eq!(add_resp.status(), reqwest::StatusCode::OK);
    let member_val: Value = add_resp.json().await.unwrap();
    let member_id = member_val["id"].as_i64().unwrap();
    assert_eq!(member_val["member_code"], "11223344");

    // 3. POST /api/v1/members/:id/subsystems
    let add_sub_req = json!({ "subsystem_code": "SECURE-PAY" });
    let sub_resp = http_client
        .post(format!("{base_url}/api/v1/members/{member_id}/subsystems"))
        .json(&add_sub_req)
        .send()
        .await
        .unwrap();
    assert_eq!(sub_resp.status(), reqwest::StatusCode::OK);
    let sub_val: Value = sub_resp.json().await.unwrap();
    assert_eq!(sub_val["subsystem_code"], "SECURE-PAY");

    // 4. POST /api/v1/security-servers
    let add_ss_req = json!({
        "owner_id": member_id,
        "server_code": "SS-HELSINKI-01",
        "address": "https://ss.nordic.fi:4000"
    });
    let ss_resp = http_client
        .post(format!("{base_url}/api/v1/security-servers"))
        .json(&add_ss_req)
        .send()
        .await
        .unwrap();
    assert_eq!(ss_resp.status(), reqwest::StatusCode::OK);
    let ss_val: Value = ss_resp.json().await.unwrap();
    let ss_id = ss_val["id"].as_i64().unwrap();
    assert_eq!(ss_val["server_code"], "SS-HELSINKI-01");

    // 5. POST /api/v1/security-servers/client
    let link_req = json!({
        "security_server_id": ss_id,
        "client_id_str": "DEV/COM/11223344/SECURE-PAY"
    });
    let link_resp = http_client
        .post(format!("{base_url}/api/v1/security-servers/client"))
        .json(&link_req)
        .send()
        .await
        .unwrap();
    assert_eq!(link_resp.status(), reqwest::StatusCode::OK);

    // 6. GET /api/v1/globalconf/shared-params.xml
    let xml_resp = http_client
        .get(format!("{base_url}/api/v1/globalconf/shared-params.xml"))
        .send()
        .await
        .unwrap();
    assert_eq!(xml_resp.status(), reqwest::StatusCode::OK);
    let xml_body = xml_resp.text().await.unwrap();
    assert!(xml_body.contains("<instanceIdentifier>DEV</instanceIdentifier>"));
    assert!(xml_body.contains("Nordic Solutions Ltd"));
    assert!(xml_body.contains("SECURE-PAY"));
    assert!(xml_body.contains("SS-HELSINKI-01"));

    // 7. GET /swagger-ui
    let ui_resp = http_client
        .get(format!("{base_url}/swagger-ui"))
        .send()
        .await
        .unwrap();
    assert_eq!(ui_resp.status(), reqwest::StatusCode::OK);
}
