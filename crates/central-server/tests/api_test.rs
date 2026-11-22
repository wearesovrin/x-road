use central_server::{CentralServerDb, central_server_app};
use poem::{
    Server,
    listener::{Acceptor, Listener, TcpListener},
};
use serde_json::{Value, json};
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

    // 1. GET /api/v1/initialization/status & POST /api/v1/initialization
    let init_status_resp = http_client
        .get(format!("{base_url}/api/v1/initialization/status"))
        .send()
        .await
        .unwrap();
    assert_eq!(init_status_resp.status(), reqwest::StatusCode::OK);
    let init_status_val: Value = init_status_resp.json().await.unwrap();
    assert_eq!(init_status_val["is_initialized"], false);

    let init_resp = http_client
        .post(format!("{base_url}/api/v1/initialization"))
        .json(&json!({
            "central_server_address": "cs.nordic.fi",
            "software_token_pin": "1234"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(init_resp.status(), reqwest::StatusCode::OK);

    // 2. Member Classes
    let add_mc_resp = http_client
        .post(format!("{base_url}/api/v1/member-classes"))
        .json(&json!({
            "code": "COM",
            "description": "Commercial Enterprises"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(add_mc_resp.status(), reqwest::StatusCode::OK);

    let get_mc_resp = http_client
        .get(format!("{base_url}/api/v1/member-classes/COM"))
        .send()
        .await
        .unwrap();
    assert_eq!(get_mc_resp.status(), reqwest::StatusCode::OK);

    // 3. GET /api/v1/status
    let status_resp = http_client
        .get(format!("{base_url}/api/v1/status"))
        .send()
        .await
        .unwrap();
    assert_eq!(status_resp.status(), reqwest::StatusCode::OK);
    let status_val: Value = status_resp.json().await.unwrap();
    assert_eq!(status_val["instance_identifier"], "DEV");
    assert_eq!(status_val["status"], "READY");

    // 4. POST /api/v1/members
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

    // 5. POST /api/v1/members/:id/subsystems
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

    // 6. GET /api/v1/clients (aggregated clients list)
    let clients_resp = http_client
        .get(format!("{base_url}/api/v1/clients"))
        .send()
        .await
        .unwrap();
    assert_eq!(clients_resp.status(), reqwest::StatusCode::OK);
    let clients_val: Value = clients_resp.json().await.unwrap();
    assert!(clients_val.as_array().unwrap().len() >= 2);

    // 7. POST /api/v1/security-servers
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

    // 8. Security Server Clients and Auth Certs
    let add_client_resp = http_client
        .post(format!(
            "{base_url}/api/v1/security-servers/{ss_id}/clients"
        ))
        .json(&json!({ "client_id_str": "DEV/COM/11223344/SECURE-PAY" }))
        .send()
        .await
        .unwrap();
    assert_eq!(add_client_resp.status(), reqwest::StatusCode::OK);

    let add_cert_resp = http_client
        .post(format!("{base_url}/api/v1/security-servers/{ss_id}/authentication-certificates"))
        .json(&json!({ "cert_pem": "-----BEGIN CERTIFICATE-----\nMIIC...\n-----END CERTIFICATE-----" }))
        .send()
        .await
        .unwrap();
    assert_eq!(add_cert_resp.status(), reqwest::StatusCode::OK);

    // 9. Global Groups
    let add_group_resp = http_client
        .post(format!("{base_url}/api/v1/global-groups"))
        .json(&json!({ "group_code": "banking", "description": "Banking services" }))
        .send()
        .await
        .unwrap();
    assert_eq!(add_group_resp.status(), reqwest::StatusCode::OK);

    // 10. Management Requests
    let mreq_resp = http_client
        .post(format!("{base_url}/api/v1/management-requests"))
        .json(&json!({
            "request_type": "CLIENT_REGISTRATION",
            "origin": "SECURITY_SERVER",
            "security_server_id": ss_id,
            "client_id_str": "DEV/COM/11223344/SECURE-PAY",
            "comments": "Automated test request"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(mreq_resp.status(), reqwest::StatusCode::OK);
    let mreq_val: Value = mreq_resp.json().await.unwrap();
    let mreq_id = mreq_val["id"].as_i64().unwrap();

    let approve_resp = http_client
        .put(format!(
            "{base_url}/api/v1/management-requests/{mreq_id}/approval"
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(approve_resp.status(), reqwest::StatusCode::OK);

    // 11. Configuration Anchor download
    let anchor_resp = http_client
        .get(format!(
            "{base_url}/api/v1/configuration-sources/internal/anchor/download"
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(anchor_resp.status(), reqwest::StatusCode::OK);
    let anchor_xml = anchor_resp.text().await.unwrap();
    assert!(anchor_xml.contains("<instanceIdentifier>DEV</instanceIdentifier>"));

    // 12. GET /api/v1/globalconf/shared-params.xml
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

    // 13. System version and backups
    let ver_resp = http_client
        .get(format!("{base_url}/api/v1/system/version"))
        .send()
        .await
        .unwrap();
    assert_eq!(ver_resp.status(), reqwest::StatusCode::OK);

    let backups_resp = http_client
        .get(format!("{base_url}/api/v1/backups"))
        .send()
        .await
        .unwrap();
    assert_eq!(backups_resp.status(), reqwest::StatusCode::OK);

    // 14. GET /swagger-ui
    let ui_resp = http_client
        .get(format!("{base_url}/swagger-ui"))
        .send()
        .await
        .unwrap();
    assert_eq!(ui_resp.status(), reqwest::StatusCode::OK);
}
