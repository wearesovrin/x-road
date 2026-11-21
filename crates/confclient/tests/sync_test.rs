use central_server::{central_server_app, CentralServerDb};
use confclient::ConfClient;
use globalconf::ConfigurationAnchor;
use identifiers::ClientId;
use poem::Server;
use poem::listener::{Acceptor, Listener};
use std::time::Duration;

#[tokio::test]
async fn test_confclient_sync_e2e() {
    // 1. Setup in-memory Central Server database
    let db = CentralServerDb::connect_in_memory()
        .await
        .expect("Failed to create in-memory CentralServerDb");

    let member = db
        .add_member("DEV", "COM", "12345678", "E2E Test Member", Some("admin@test.ee"))
        .await
        .unwrap();

    let _sub = db.add_subsystem(member.id, "MANAGEMENT").await.unwrap();

    let sec_server = db
        .register_security_server(member.id, "SS1", "https://ss1.test.ee:8443")
        .await
        .unwrap();

    let client_sub = ClientId::subsystem_unchecked("DEV", "COM", "12345678", "MANAGEMENT");
    db.add_security_server_client(sec_server.id, &client_sub)
        .await
        .unwrap();

    let _ca = db
        .add_approved_ca("TestCA", b"fake-ca-cert-bytes", false, None)
        .await
        .unwrap();

    let _tsa = db
        .add_approved_tsa("TestTSA", "http://tsa.test.ee:8899", b"fake-tsa-cert-bytes")
        .await
        .unwrap();

    // 2. Launch Central Server on ephemeral port
    let app = central_server_app("DEV".to_string(), db);
    let listener = poem::listener::TcpListener::bind("127.0.0.1:0");
    let acceptor = listener.into_acceptor().await.expect("Failed to bind ephemeral listener");
    let addrs = acceptor.local_addr();
    let port = addrs[0].as_socket_addr().expect("Failed to get socket addr").port();

    tokio::spawn(async move {
        let server = Server::new_with_acceptor(acceptor);
        if let Err(e) = server.run(app).await {
            eprintln!("Central server error: {e}");
        }
    });

    // Brief yield for server to start accepting
    tokio::time::sleep(Duration::from_millis(50)).await;

    // 3. Initialize ConfClient pointing to the running Central Server
    let anchor = ConfigurationAnchor {
        instance_identifier: "DEV".to_string(),
        generated_at: chrono::Utc::now(),
        download_urls: vec![
            format!("http://127.0.0.1:{port}/internalconf/shared-params.xml"),
        ],
        verification_cert_hashes: vec![],
    };

    let client = ConfClient::new(anchor);

    // 4. Perform synchronization
    let conf = client.sync_once().await.expect("ConfClient sync failed");

    // 5. Verify cached configuration
    assert_eq!(conf.instance_identifier, "DEV");

    let member_id = ClientId::member_unchecked("DEV", "COM", "12345678");
    assert!(conf.is_member_registered(&member_id));
    assert!(conf.is_member_registered(&client_sub));

    assert_eq!(
        conf.get_security_server_address_for_client(&client_sub),
        Some(&"https://ss1.test.ee:8443".to_string())
    );

    assert_eq!(conf.approved_cas.len(), 1);
    assert_eq!(conf.approved_cas[0].name, "TestCA");
    assert_eq!(conf.approved_cas[0].top_ca_cert, b"fake-ca-cert-bytes");

    assert_eq!(conf.approved_tsps.len(), 1);
    assert_eq!(conf.approved_tsps[0].name, "TestTSA");
    assert_eq!(conf.approved_tsps[0].url, "http://tsa.test.ee:8899");

    // 6. Verify client.get_global_conf() returns cached conf
    let cached = client.get_global_conf().await.expect("Expected cached globalconf");
    assert_eq!(cached.instance_identifier, "DEV");
}
