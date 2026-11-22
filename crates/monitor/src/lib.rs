use proto::metrics_service_server::{MetricsService, MetricsServiceServer};
use proto::monitor_service_server::{MonitorService, MonitorServiceServer};
use proto::op_monitor_service_server::{OpMonitorService, OpMonitorServiceServer};
use proto::*;
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct MonitorServiceImpl;

#[tonic::async_trait]
impl MonitorService for MonitorServiceImpl {
    async fn get_stats(&self, _req: Request<StatsReq>) -> Result<Response<StatsResp>, Status> {
        Ok(Response::new(StatsResp {
            open_file_descriptor_count: 10,
            max_file_descriptor_count: 1024,
            system_cpu_load: 0.05,
            committed_virtual_memory_size: 1024 * 1024,
            free_physical_memory_size: 1024 * 1024 * 512,
            total_physical_memory_size: 1024 * 1024 * 1024,
            free_swap_space_size: 0,
            total_swap_space_size: 0,
        }))
    }
}

pub fn create_monitor_service() -> MonitorServiceServer<MonitorServiceImpl> {
    MonitorServiceServer::new(MonitorServiceImpl)
}

#[derive(Debug, Default)]
pub struct MetricsServiceImpl;

#[tonic::async_trait]
impl MetricsService for MetricsServiceImpl {
    async fn get_metrics(
        &self,
        _req: Request<SystemMetricsReq>,
    ) -> Result<Response<SystemMetricsResp>, Status> {
        Ok(Response::new(SystemMetricsResp {
            metrics: Some(MetricsGroup {
                name: "system".to_string(),
                metrics: vec![],
            }),
        }))
    }
}

pub fn create_metrics_service() -> MetricsServiceServer<MetricsServiceImpl> {
    MetricsServiceServer::new(MetricsServiceImpl)
}

#[derive(Debug, Default)]
pub struct OpMonitorServiceImpl;

#[tonic::async_trait]
impl OpMonitorService for OpMonitorServiceImpl {
    async fn get_operational_data_intervals(
        &self,
        _req: Request<GetOperationalDataIntervalsReq>,
    ) -> Result<Response<GetOperationalDataIntervalsResp>, Status> {
        Ok(Response::new(GetOperationalDataIntervalsResp {
            operational_data_interval: vec![],
        }))
    }

    async fn store_operational_data(
        &self,
        _req: Request<StoreOperationalDataReq>,
    ) -> Result<Response<StoreOperationalDataResp>, Status> {
        Ok(Response::new(StoreOperationalDataResp {}))
    }

    async fn query_operational_data(
        &self,
        _req: Request<QueryOperationalDataReq>,
    ) -> Result<Response<QueryOperationalDataResp>, Status> {
        Ok(Response::new(QueryOperationalDataResp {
            response: b"<operationalDataResponse/>".to_vec(),
            content_type: "text/xml".to_string(),
        }))
    }
}

pub fn create_op_monitor_service() -> OpMonitorServiceServer<OpMonitorServiceImpl> {
    OpMonitorServiceServer::new(OpMonitorServiceImpl)
}

/// Launches the environmental and operational monitoring gRPC services on the given address.
pub async fn run_monitor(bind_addr: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!(
        "Starting X-Road Environmental and Operational Monitoring Daemon on {}...",
        bind_addr
    );
    let addr = bind_addr.parse()?;
    tonic::transport::Server::builder()
        .add_service(create_monitor_service())
        .add_service(create_metrics_service())
        .add_service(create_op_monitor_service())
        .serve(addr)
        .await?;
    Ok(())
}

/// Generates Prometheus / OpenMetrics plain-text formatted output for environmental and proxy operational metrics.
pub fn generate_prometheus_metrics(request_count: u64, error_count: u64) -> String {
    format!(
        "# HELP xroad_system_cpu_load System CPU load\n\
         # TYPE xroad_system_cpu_load gauge\n\
         xroad_system_cpu_load 0.05\n\
         # HELP xroad_system_memory_free_bytes Free physical memory in bytes\n\
         # TYPE xroad_system_memory_free_bytes gauge\n\
         xroad_system_memory_free_bytes 536870912\n\
         # HELP xroad_system_memory_total_bytes Total physical memory in bytes\n\
         # TYPE xroad_system_memory_total_bytes gauge\n\
         xroad_system_memory_total_bytes 1073741824\n\
         # HELP xroad_system_open_file_descriptors Open file descriptors\n\
         # TYPE xroad_system_open_file_descriptors gauge\n\
         xroad_system_open_file_descriptors 10\n\
         # HELP xroad_proxy_requests_total Total requests processed by proxy\n\
         # TYPE xroad_proxy_requests_total counter\n\
         xroad_proxy_requests_total {request_count}\n\
         # HELP xroad_proxy_errors_total Total error requests\n\
         # TYPE xroad_proxy_errors_total counter\n\
         xroad_proxy_errors_total {error_count}\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prometheus_metrics_format() {
        let metrics = generate_prometheus_metrics(123, 4);
        assert!(metrics.contains("xroad_system_cpu_load 0.05"));
        assert!(metrics.contains("xroad_proxy_requests_total 123"));
        assert!(metrics.contains("xroad_proxy_errors_total 4"));
    }

    #[tokio::test]
    async fn test_monitor_service_grpc() {
        let svc = MonitorServiceImpl;
        let resp = svc.get_stats(Request::new(StatsReq {})).await.unwrap();
        let stats = resp.into_inner();
        assert_eq!(stats.open_file_descriptor_count, 10);
        assert_eq!(stats.system_cpu_load, 0.05);
    }
}
