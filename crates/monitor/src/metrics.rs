use crate::system::SystemCollector;
use prometheus::{
    Encoder, Gauge, Histogram, HistogramOpts, IntCounter, Opts, Registry, TextEncoder,
};

/// Standard Prometheus metrics container for X-Road security and central servers.
#[derive(Clone)]
pub struct XRoadMetrics {
    registry: Registry,
    pub system_cpu_load: Gauge,
    pub memory_free_bytes: Gauge,
    pub memory_total_bytes: Gauge,
    pub open_file_descriptors: Gauge,
    pub max_file_descriptors: Gauge,
    pub disk_free_bytes: Gauge,
    pub proxy_requests_total: IntCounter,
    pub proxy_errors_total: IntCounter,
    pub proxy_request_duration_seconds: Histogram,
    pub proxy_request_size_bytes: Histogram,
    pub proxy_response_size_bytes: Histogram,
}

impl Default for XRoadMetrics {
    fn default() -> Self {
        Self::new().expect("Failed to initialize X-Road Prometheus metrics")
    }
}

impl XRoadMetrics {
    /// Creates and registers all standard X-Road Prometheus metrics with a new registry.
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();

        let system_cpu_load = Gauge::with_opts(Opts::new(
            "xroad_system_cpu_load",
            "System CPU load (0.0 to 1.0)",
        ))?;
        let memory_free_bytes = Gauge::with_opts(Opts::new(
            "xroad_system_memory_free_bytes",
            "Free physical memory in bytes",
        ))?;
        let memory_total_bytes = Gauge::with_opts(Opts::new(
            "xroad_system_memory_total_bytes",
            "Total physical memory in bytes",
        ))?;
        let open_file_descriptors = Gauge::with_opts(Opts::new(
            "xroad_system_open_file_descriptors",
            "Open file descriptors",
        ))?;
        let max_file_descriptors = Gauge::with_opts(Opts::new(
            "xroad_system_max_file_descriptors",
            "Maximum open file descriptors limit",
        ))?;
        let disk_free_bytes = Gauge::with_opts(Opts::new(
            "xroad_system_disk_free_bytes",
            "Free disk space in bytes",
        ))?;

        let proxy_requests_total = IntCounter::with_opts(Opts::new(
            "xroad_proxy_requests_total",
            "Total requests processed by proxy",
        ))?;
        let proxy_errors_total = IntCounter::with_opts(Opts::new(
            "xroad_proxy_errors_total",
            "Total error requests processed by proxy",
        ))?;

        let proxy_request_duration_seconds = Histogram::with_opts(
            HistogramOpts::new(
                "xroad_proxy_request_duration_seconds",
                "Proxy request duration in seconds",
            )
            .buckets(vec![
                0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ]),
        )?;
        let proxy_request_size_bytes = Histogram::with_opts(
            HistogramOpts::new(
                "xroad_proxy_request_size_bytes",
                "Proxy request payload size in bytes",
            )
            .buckets(vec![
                128.0, 512.0, 1024.0, 4096.0, 16384.0, 65536.0, 262144.0, 1048576.0,
            ]),
        )?;
        let proxy_response_size_bytes = Histogram::with_opts(
            HistogramOpts::new(
                "xroad_proxy_response_size_bytes",
                "Proxy response payload size in bytes",
            )
            .buckets(vec![
                128.0, 512.0, 1024.0, 4096.0, 16384.0, 65536.0, 262144.0, 1048576.0,
            ]),
        )?;

        registry.register(Box::new(system_cpu_load.clone()))?;
        registry.register(Box::new(memory_free_bytes.clone()))?;
        registry.register(Box::new(memory_total_bytes.clone()))?;
        registry.register(Box::new(open_file_descriptors.clone()))?;
        registry.register(Box::new(max_file_descriptors.clone()))?;
        registry.register(Box::new(disk_free_bytes.clone()))?;
        registry.register(Box::new(proxy_requests_total.clone()))?;
        registry.register(Box::new(proxy_errors_total.clone()))?;
        registry.register(Box::new(proxy_request_duration_seconds.clone()))?;
        registry.register(Box::new(proxy_request_size_bytes.clone()))?;
        registry.register(Box::new(proxy_response_size_bytes.clone()))?;

        Ok(Self {
            registry,
            system_cpu_load,
            memory_free_bytes,
            memory_total_bytes,
            open_file_descriptors,
            max_file_descriptors,
            disk_free_bytes,
            proxy_requests_total,
            proxy_errors_total,
            proxy_request_duration_seconds,
            proxy_request_size_bytes,
            proxy_response_size_bytes,
        })
    }

    /// Returns a reference to the underlying Prometheus registry.
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// Refreshes live host system stats into the Prometheus gauges.
    pub fn update_from_system(&self, collector: &SystemCollector) {
        let stats = collector.get_stats_resp();
        self.system_cpu_load.set(stats.system_cpu_load);
        self.memory_free_bytes
            .set(stats.free_physical_memory_size as f64);
        self.memory_total_bytes
            .set(stats.total_physical_memory_size as f64);
        self.open_file_descriptors
            .set(stats.open_file_descriptor_count as f64);
        self.max_file_descriptors
            .set(stats.max_file_descriptor_count as f64);
    }

    /// Records metrics for a single completed proxy query.
    pub fn record_request(
        &self,
        duration_secs: f64,
        request_size: usize,
        response_size: usize,
        is_error: bool,
    ) {
        self.proxy_requests_total.inc();
        if is_error {
            self.proxy_errors_total.inc();
        }
        self.proxy_request_duration_seconds.observe(duration_secs);
        self.proxy_request_size_bytes.observe(request_size as f64);
        self.proxy_response_size_bytes.observe(response_size as f64);
    }

    /// Formats all registered metrics into Prometheus plain text format.
    pub fn gather_text(&self) -> String {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        if encoder.encode(&metric_families, &mut buffer).is_ok() {
            String::from_utf8(buffer).unwrap_or_default()
        } else {
            String::new()
        }
    }
}

/// Backward-compatible metrics generator that produces standard Prometheus format output.
pub fn generate_prometheus_metrics(request_count: u64, error_count: u64) -> String {
    let metrics = XRoadMetrics::default();
    let collector = SystemCollector::new();
    metrics.update_from_system(&collector);
    metrics.proxy_requests_total.inc_by(request_count);
    metrics.proxy_errors_total.inc_by(error_count);
    metrics.gather_text()
}
