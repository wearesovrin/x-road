use crate::operational::OperationalStore;
use crate::system::SystemCollector;
use proto::metrics_service_server::{MetricsService, MetricsServiceServer};
use proto::monitor_service_server::{MonitorService, MonitorServiceServer};
use proto::op_monitor_service_server::{OpMonitorService, OpMonitorServiceServer};
use proto::*;
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct MonitorServiceImpl {
    collector: Arc<SystemCollector>,
}

impl MonitorServiceImpl {
    pub fn new(collector: Arc<SystemCollector>) -> Self {
        Self { collector }
    }
}

#[tonic::async_trait]
impl MonitorService for MonitorServiceImpl {
    async fn get_stats(&self, _req: Request<StatsReq>) -> Result<Response<StatsResp>, Status> {
        Ok(Response::new(self.collector.get_stats_resp()))
    }
}

pub fn create_monitor_service(
    collector: Arc<SystemCollector>,
) -> MonitorServiceServer<MonitorServiceImpl> {
    MonitorServiceServer::new(MonitorServiceImpl::new(collector))
}

#[derive(Clone)]
pub struct MetricsServiceImpl {
    collector: Arc<SystemCollector>,
}

impl MetricsServiceImpl {
    pub fn new(collector: Arc<SystemCollector>) -> Self {
        Self { collector }
    }
}

#[tonic::async_trait]
impl MetricsService for MetricsServiceImpl {
    async fn get_metrics(
        &self,
        req: Request<SystemMetricsReq>,
    ) -> Result<Response<SystemMetricsResp>, Status> {
        let req_data = req.into_inner();
        Ok(Response::new(
            self.collector.get_system_metrics_resp(&req_data),
        ))
    }
}

pub fn create_metrics_service(
    collector: Arc<SystemCollector>,
) -> MetricsServiceServer<MetricsServiceImpl> {
    MetricsServiceServer::new(MetricsServiceImpl::new(collector))
}

#[derive(Clone)]
pub struct OpMonitorServiceImpl {
    store: Arc<OperationalStore>,
}

impl OpMonitorServiceImpl {
    pub fn new(store: Arc<OperationalStore>) -> Self {
        Self { store }
    }
}

#[tonic::async_trait]
impl OpMonitorService for OpMonitorServiceImpl {
    async fn get_operational_data_intervals(
        &self,
        req: Request<GetOperationalDataIntervalsReq>,
    ) -> Result<Response<GetOperationalDataIntervalsResp>, Status> {
        let req_data = req.into_inner();
        let intervals = self.store.get_intervals(&req_data);
        Ok(Response::new(GetOperationalDataIntervalsResp {
            operational_data_interval: intervals,
        }))
    }

    async fn store_operational_data(
        &self,
        req: Request<StoreOperationalDataReq>,
    ) -> Result<Response<StoreOperationalDataResp>, Status> {
        let req_data = req.into_inner();
        self.store.store_records(req_data.records);
        Ok(Response::new(StoreOperationalDataResp {}))
    }

    async fn query_operational_data(
        &self,
        req: Request<QueryOperationalDataReq>,
    ) -> Result<Response<QueryOperationalDataResp>, Status> {
        let req_data = req.into_inner();
        Ok(Response::new(self.store.query_records(&req_data)))
    }
}

pub fn create_op_monitor_service(
    store: Arc<OperationalStore>,
) -> OpMonitorServiceServer<OpMonitorServiceImpl> {
    OpMonitorServiceServer::new(OpMonitorServiceImpl::new(store))
}

/// Runs the Tonic gRPC monitoring services on the given address.
pub async fn run_grpc_server(
    bind_addr: &str,
    collector: Arc<SystemCollector>,
    store: Arc<OperationalStore>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!(
        "Starting X-Road Environmental and Operational Monitoring gRPC Server on {}...",
        bind_addr
    );
    let addr = bind_addr.parse()?;
    tonic::transport::Server::builder()
        .add_service(create_monitor_service(collector.clone()))
        .add_service(create_metrics_service(collector))
        .add_service(create_op_monitor_service(store))
        .serve(addr)
        .await?;
    Ok(())
}
