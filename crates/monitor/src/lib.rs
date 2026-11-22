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
