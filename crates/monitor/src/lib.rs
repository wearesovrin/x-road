pub mod grpc;
pub mod http;
pub mod metrics;
pub mod operational;
pub mod system;

use std::sync::Arc;

pub use grpc::{
    MetricsServiceImpl, MonitorServiceImpl, OpMonitorServiceImpl, create_metrics_service,
    create_monitor_service, create_op_monitor_service, run_grpc_server,
};
pub use http::{HealthStatusResponse, create_monitor_app, run_http_metrics_server};
pub use metrics::{XRoadMetrics, generate_prometheus_metrics};
pub use operational::OperationalStore;
pub use system::SystemCollector;

/// Launches the X-Road Environmental and Operational Monitoring services (gRPC and Poem HTTP metrics).
pub async fn run_monitor(bind_addr: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!(
        "Starting X-Road Environmental and Operational Monitoring Daemon on {}...",
        bind_addr
    );

    let collector = Arc::new(SystemCollector::new());
    let op_store = Arc::new(OperationalStore::default());
    let metrics = Arc::new(XRoadMetrics::default());

    // Optional HTTP metrics server port (defaults to 9090 or environment variable)
    let http_bind =
        std::env::var("XROAD_METRICS_HTTP_BIND").unwrap_or_else(|_| "0.0.0.0:9090".to_string());
    let http_collector = collector.clone();
    let http_op_store = op_store.clone();
    let http_metrics = metrics.clone();

    tokio::spawn(async move {
        if let Err(e) =
            run_http_metrics_server(&http_bind, http_metrics, http_op_store, http_collector).await
        {
            tracing::warn!("Poem HTTP metrics server terminated: {}", e);
        }
    });

    run_grpc_server(bind_addr, collector, op_store).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use poem::Endpoint;
    use proto::monitor_service_server::MonitorService;
    use proto::op_monitor_service_server::OpMonitorService;
    use proto::*;
    use tonic::Request;

    #[test]
    fn test_system_collector_stats() {
        let collector = SystemCollector::new();
        let stats = collector.get_stats_resp();
        assert!(stats.total_physical_memory_size > 0);
        assert!(stats.open_file_descriptor_count > 0);
        assert!(stats.max_file_descriptor_count > 0);

        let sys_metrics = collector.get_system_metrics_resp(&SystemMetricsReq {
            metric_names: vec![],
            is_client_owner: true,
        });
        assert!(sys_metrics.metrics.is_some());
        let group = sys_metrics.metrics.unwrap();
        assert_eq!(group.name, "system");
        assert!(!group.metrics.is_empty());
    }

    #[test]
    fn test_operational_store_intervals_and_query() {
        let store = OperationalStore::new(100);
        assert_eq!(store.len(), 0);

        let now = 1_700_000_000_000i64;
        let records = vec![
            OperationalDataRecordProto {
                security_server_internal_ip: Some("10.0.0.1".to_string()),
                request_in_ts: Some(now),
                response_out_ts: Some(now + 45),
                client_xroad_instance: Some("DEV".to_string()),
                client_member_class: Some("GOV".to_string()),
                client_member_code: Some("12345".to_string()),
                service_code: Some("testService".to_string()),
                succeeded: Some(true),
                ..Default::default()
            },
            OperationalDataRecordProto {
                security_server_internal_ip: Some("10.0.0.1".to_string()),
                request_in_ts: Some(now + 100),
                response_out_ts: Some(now + 200),
                client_xroad_instance: Some("DEV".to_string()),
                client_member_class: Some("GOV".to_string()),
                client_member_code: Some("12345".to_string()),
                service_code: Some("testService".to_string()),
                succeeded: Some(false),
                ..Default::default()
            },
        ];

        store.store_records(records);
        assert_eq!(store.len(), 2);

        let intervals = store.get_intervals(&GetOperationalDataIntervalsReq {
            records_from: now - 1000,
            records_to: now + 5000,
            interval_in_minutes: 5,
            security_server_type: None,
            member_id: None,
            service_id: None,
        });
        assert!(!intervals.is_empty());
        assert_eq!(intervals[0].success_count, 1);
        assert_eq!(intervals[0].failure_count, 1);

        let query_resp = store.query_records(&QueryOperationalDataReq {
            soap_query: vec![],
            content_type: "text/xml".to_string(),
            access_scope: AccessScope::FullView as i32,
        });
        let xml = String::from_utf8(query_resp.response).unwrap();
        assert!(xml.contains("<om:recordsCount>2</om:recordsCount>"));
        assert!(
            xml.contains("<om:securityServerInternalIp>10.0.0.1</om:securityServerInternalIp>")
        );
    }

    #[test]
    fn test_prometheus_metrics_format() {
        let metrics = generate_prometheus_metrics(123, 4);
        assert!(metrics.contains("xroad_system_cpu_load"));
        assert!(metrics.contains("xroad_system_memory_free_bytes"));
        assert!(metrics.contains("xroad_system_open_file_descriptors"));
        assert!(metrics.contains("xroad_proxy_requests_total 123"));
        assert!(metrics.contains("xroad_proxy_errors_total 4"));
    }

    #[tokio::test]
    async fn test_poem_http_metrics_and_health_endpoints() {
        let collector = Arc::new(SystemCollector::new());
        let op_store = Arc::new(OperationalStore::default());
        let metrics = Arc::new(XRoadMetrics::default());

        metrics.record_request(0.042, 1024, 2048, false);

        let app = create_monitor_app(metrics, op_store, collector);

        // Test /health endpoint
        let req = poem::Request::builder()
            .uri("/health".parse().unwrap())
            .finish();
        let resp = app.call(req).await.unwrap();
        assert_eq!(resp.status(), poem::http::StatusCode::OK);

        // Test /metrics endpoint
        let req = poem::Request::builder()
            .uri("/metrics".parse().unwrap())
            .finish();
        let resp = app.call(req).await.unwrap();
        assert_eq!(resp.status(), poem::http::StatusCode::OK);
        let body_bytes = resp.into_body().into_bytes().await.unwrap();
        let body_text = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(body_text.contains("xroad_system_cpu_load"));
        assert!(body_text.contains("xroad_proxy_requests_total 1"));
    }

    #[tokio::test]
    async fn test_grpc_services() {
        let collector = Arc::new(SystemCollector::new());
        let op_store = Arc::new(OperationalStore::default());

        let mon_svc = MonitorServiceImpl::new(collector.clone());
        let stats_resp = mon_svc.get_stats(Request::new(StatsReq {})).await.unwrap();
        let stats = stats_resp.into_inner();
        assert!(stats.total_physical_memory_size > 0);

        let op_svc = OpMonitorServiceImpl::new(op_store.clone());
        let store_resp = op_svc
            .store_operational_data(Request::new(StoreOperationalDataReq {
                records: vec![OperationalDataRecordProto {
                    succeeded: Some(true),
                    ..Default::default()
                }],
            }))
            .await;
        assert!(store_resp.is_ok());
        assert_eq!(op_store.len(), 1);
    }
}
