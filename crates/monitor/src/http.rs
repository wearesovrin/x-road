use crate::metrics::XRoadMetrics;
use crate::operational::OperationalStore;
use crate::system::SystemCollector;
use poem::{
    EndpointExt, Response, Route, Server,
    endpoint::PrometheusExporter,
    get, handler,
    http::StatusCode,
    listener::TcpListener,
    middleware::{OpenTelemetryMetrics, OpenTelemetryTracing},
    post,
    web::{Data, Json},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthStatusResponse {
    pub status: String,
    pub system_cpu_load: f64,
    pub free_memory_bytes: i64,
    pub total_memory_bytes: i64,
    pub open_file_descriptors: i64,
    pub operational_records_count: usize,
}

#[handler]
async fn health_handler(
    collector: Data<&Arc<SystemCollector>>,
    op_store: Data<&Arc<OperationalStore>>,
) -> Json<HealthStatusResponse> {
    let stats = collector.get_stats_resp();
    Json(HealthStatusResponse {
        status: "UP".to_string(),
        system_cpu_load: stats.system_cpu_load,
        free_memory_bytes: stats.free_physical_memory_size,
        total_memory_bytes: stats.total_physical_memory_size,
        open_file_descriptors: stats.open_file_descriptor_count,
        operational_records_count: op_store.len(),
    })
}

#[handler]
async fn metrics_handler(
    metrics: Data<&Arc<XRoadMetrics>>,
    collector: Data<&Arc<SystemCollector>>,
) -> Response {
    metrics.update_from_system(&collector);
    let text = metrics.gather_text();
    Response::builder()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(text)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreRecordDto {
    pub security_server_internal_ip: Option<String>,
    pub request_in_ts: Option<i64>,
    pub response_out_ts: Option<i64>,
    pub client_xroad_instance: Option<String>,
    pub client_member_class: Option<String>,
    pub client_member_code: Option<String>,
    pub service_code: Option<String>,
    pub succeeded: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct StoreRecordRequest {
    pub records: Vec<StoreRecordDto>,
}

#[handler]
async fn store_operational_data_handler(
    req: Json<StoreRecordRequest>,
    op_store: Data<&Arc<OperationalStore>>,
) -> Response {
    let protos = req
        .0
        .records
        .into_iter()
        .map(|r| proto::OperationalDataRecordProto {
            security_server_internal_ip: r.security_server_internal_ip,
            request_in_ts: r.request_in_ts,
            response_out_ts: r.response_out_ts,
            client_xroad_instance: r.client_xroad_instance,
            client_member_class: r.client_member_class,
            client_member_code: r.client_member_code,
            service_code: r.service_code,
            succeeded: r.succeeded,
            ..Default::default()
        })
        .collect();
    op_store.store_records(protos);
    Response::builder()
        .status(StatusCode::OK)
        .content_type("application/json")
        .body(r#"{"status":"STORED"}"#)
}

/// Constructs the Poem monitoring and metrics route configured with Prometheus and OpenTelemetry.
pub fn create_monitor_app(
    metrics: Arc<XRoadMetrics>,
    op_store: Arc<OperationalStore>,
    collector: Arc<SystemCollector>,
) -> impl poem::Endpoint<Output = Response> {
    let prometheus_exporter = PrometheusExporter::new(metrics.registry().clone());

    Route::new()
        .at("/metrics", get(metrics_handler))
        .at("/metrics/exporter", prometheus_exporter)
        .at("/health", get(health_handler))
        .at("/status", get(health_handler))
        .at("/op-monitor/store", post(store_operational_data_handler))
        .data(metrics)
        .data(op_store)
        .data(collector)
        .with(OpenTelemetryMetrics::new())
        .with(OpenTelemetryTracing::new(opentelemetry::global::tracer(
            "xroad-monitor",
        )))
}

/// Runs the Poem HTTP monitoring server on the given address.
pub async fn run_http_metrics_server(
    bind_addr: &str,
    metrics: Arc<XRoadMetrics>,
    op_store: Arc<OperationalStore>,
    collector: Arc<SystemCollector>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!(
        "Starting X-Road Prometheus & OpenTelemetry HTTP Metrics Server on {}...",
        bind_addr
    );
    let app = create_monitor_app(metrics, op_store, collector);
    Server::new(TcpListener::bind(bind_addr)).run(app).await?;
    Ok(())
}
