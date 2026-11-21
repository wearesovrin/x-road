use proto::monitor_service_server::{MonitorService, MonitorServiceServer};
use proto::{StatsReq, StatsResp};
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

/// Launches the environmental monitoring gRPC service on the given address.
pub async fn run_monitor(bind_addr: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!(
        "Starting X-Road Environmental Monitoring Daemon on {}...",
        bind_addr
    );
    let addr = bind_addr.parse()?;
    tonic::transport::Server::builder()
        .add_service(create_monitor_service())
        .serve(addr)
        .await?;
    Ok(())
}
