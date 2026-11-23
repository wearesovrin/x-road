pub mod client_proxy;
pub mod mtls;
pub mod server_proxy;

pub use client_proxy::{
    ClientProxyState, client_proxy_app, parse_headers_from_rest_path, run_client_proxy,
};
pub use mtls::{create_server_mtls_config, generate_self_signed_cert, run_server_proxy_mtls};
pub use server_proxy::{ServerProxyState, run_server_proxy, server_proxy_app};

// Backward-compatible alias
pub use client_proxy::ClientProxyState as ProxyState;

#[cfg(test)]
mod tests {
    use super::*;
    use common::headers;
    use identifiers::ClientId;
    use poem::Endpoint;
    use serverconf::InMemoryServerConf;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[test]
    fn test_parse_headers_from_rest_path() {
        let mut header_map = HashMap::new();
        header_map.insert(
            headers::X_ROAD_CLIENT.to_string(),
            "EE/COM/11112222/client-sub".to_string(),
        );
        header_map.insert(headers::X_ROAD_ID.to_string(), "query-uuid-1".to_string());

        // 4 parts: /r1/INSTANCE/CLASS/CODE/SERVICE
        let h4 = parse_headers_from_rest_path("/r1/EE/COM/99998888/echoService", &header_map)
            .expect("Should parse 4-part REST path");
        assert_eq!(h4.service.member_code(), "99998888");
        assert_eq!(h4.service.subsystem_code(), None);
        assert_eq!(h4.service.service_code(), "echoService");
        assert_eq!(h4.service.service_version(), None);
        assert_eq!(h4.id, "query-uuid-1");

        // 5 parts: /r1/INSTANCE/CLASS/CODE/SUBSYSTEM/SERVICE
        let h5 =
            parse_headers_from_rest_path("/r1/EE/COM/99998888/orders/getOrderDetails", &header_map)
                .expect("Should parse 5-part REST path");
        assert_eq!(h5.service.member_code(), "99998888");
        assert_eq!(h5.service.subsystem_code(), Some("orders"));
        assert_eq!(h5.service.service_code(), "getOrderDetails");
        assert_eq!(h5.service.service_version(), None);

        // 6 parts: /r1/INSTANCE/CLASS/CODE/SUBSYSTEM/SERVICE/VERSION
        let h6 = parse_headers_from_rest_path(
            "/r1/EE/COM/99998888/orders/getOrderDetails/v1",
            &header_map,
        )
        .expect("Should parse 6-part REST path");
        assert_eq!(h6.service.member_code(), "99998888");
        assert_eq!(h6.service.subsystem_code(), Some("orders"));
        assert_eq!(h6.service.service_code(), "getOrderDetails");
        assert_eq!(h6.service.service_version(), Some("v1"));

        // Invalid paths
        assert!(parse_headers_from_rest_path("/other/prefix", &header_map).is_none());
        assert!(parse_headers_from_rest_path("/r1/EE/COM", &header_map).is_none());

        // Missing calling client header
        let empty_headers = HashMap::new();
        assert!(
            parse_headers_from_rest_path("/r1/EE/COM/99998888/echoService", &empty_headers)
                .is_none()
        );
    }

    #[test]
    fn test_mtls_cert_generation_and_config() {
        let (cert_der, key_der) =
            generate_self_signed_cert("xroad-test-server").expect("Cert generation should succeed");
        assert!(!cert_der.is_empty());

        let server_config =
            create_server_mtls_config(cert_der, key_der, None).expect("Server config should build");
        assert_eq!(server_config.alpn_protocols.len(), 0);
    }

    #[tokio::test]
    async fn test_server_proxy_metaservices() {
        let mut server_conf = InMemoryServerConf::default();
        let message_log = Arc::new(
            messagelog::MessageLogDb::connect_in_memory()
                .await
                .expect("In-memory messagelog"),
        );

        let _producer = ClientId::member("DEV", "COM", "PRODUCER").unwrap();
        let svc = identifiers::ServiceId::from_parts(
            "DEV",
            "COM",
            "PRODUCER",
            None::<&str>,
            "testService",
            None::<&str>,
        )
        .unwrap();

        server_conf.add_service(svc.clone(), "http://127.0.0.1:8080/test".into());

        let state = ServerProxyState::new(Arc::new(server_conf), message_log);
        let app = server_proxy_app(state);

        // Call listMethods
        let req_xml = r#"<?xml version="1.0" encoding="utf-8"?>
<SOAP-ENV:Envelope xmlns:SOAP-ENV="http://schemas.xmlsoap.org/soap/envelope/"
                   xmlns:xroad="http://x-road.eu/xsd/xroad.xsd"
                   xmlns:id="http://x-road.eu/xsd/identifiers">
    <SOAP-ENV:Header>
        <xroad:client id:objectType="MEMBER">
            <id:xRoadInstance>DEV</id:xRoadInstance>
            <id:memberClass>COM</id:memberClass>
            <id:memberCode>CONSUMER</id:memberCode>
        </xroad:client>
        <xroad:service id:objectType="SERVICE">
            <id:xRoadInstance>DEV</id:xRoadInstance>
            <id:memberClass>COM</id:memberClass>
            <id:memberCode>PRODUCER</id:memberCode>
            <id:serviceCode>listMethods</id:serviceCode>
        </xroad:service>
        <xroad:id>meta-query-123</xroad:id>
        <xroad:protocolVersion>4.0</xroad:protocolVersion>
    </SOAP-ENV:Header>
    <SOAP-ENV:Body/>
</SOAP-ENV:Envelope>"#
            .to_string();

        let req = poem::Request::builder()
            .method(poem::http::Method::POST)
            .content_type("text/xml; charset=utf-8")
            .body(req_xml);

        let resp = app.call(req).await.unwrap();
        assert_eq!(resp.status(), poem::http::StatusCode::OK);

        let body_bytes = resp.into_body().into_bytes().await.unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(body_str.contains("listMethodsResponse"));
        assert!(body_str.contains("testService"));
    }
}
