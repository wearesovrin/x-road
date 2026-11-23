use parking_lot::Mutex;
use proto::metrics::Value;
use proto::*;
use sysinfo::{Disks, System};

/// Cross-platform system and environmental metrics collector.
pub struct SystemCollector {
    sys: Mutex<System>,
    disks: Mutex<Disks>,
}

impl Default for SystemCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemCollector {
    /// Creates a new `SystemCollector` with refreshed system and disk state.
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let disks = Disks::new_with_refreshed_list();
        Self {
            sys: Mutex::new(sys),
            disks: Mutex::new(disks),
        }
    }

    /// Refreshes CPU, memory, and disk usage statistics.
    pub fn refresh(&self) {
        let mut sys = self.sys.lock();
        sys.refresh_cpu_all();
        sys.refresh_memory();
        drop(sys);

        let mut disks = self.disks.lock();
        disks.refresh(true);
    }

    /// Obtains open file descriptor count for the current process.
    pub fn open_file_descriptor_count() -> i64 {
        #[cfg(target_os = "macos")]
        {
            if let Ok(entries) = std::fs::read_dir("/dev/fd") {
                return entries.count() as i64;
            }
        }
        #[cfg(target_os = "linux")]
        {
            if let Ok(entries) = std::fs::read_dir("/proc/self/fd") {
                return entries.count() as i64;
            }
        }
        10
    }

    /// Obtains maximum allowed file descriptor count for the current process.
    pub fn max_file_descriptor_count() -> i64 {
        #[cfg(unix)]
        unsafe {
            let mut rlim = libc::rlimit {
                rlim_cur: 0,
                rlim_max: 0,
            };
            if libc::getrlimit(libc::RLIMIT_NOFILE, &mut rlim) == 0 {
                return rlim.rlim_cur as i64;
            }
        }
        1024
    }

    /// Returns the system stats response according to UnixOperatingSystemMXBean / X-Road specs.
    pub fn get_stats_resp(&self) -> StatsResp {
        self.refresh();
        let sys = self.sys.lock();

        let cpu_load = (sys.global_cpu_usage() as f64) / 100.0;
        let total_mem = sys.total_memory() as i64;
        let free_mem = sys.available_memory() as i64;
        let used_mem = sys.used_memory() as i64;
        let total_swap = sys.total_swap() as i64;
        let free_swap = sys.free_swap() as i64;
        let used_swap = sys.used_swap() as i64;
        let committed_vm = used_mem + used_swap;

        StatsResp {
            open_file_descriptor_count: Self::open_file_descriptor_count(),
            max_file_descriptor_count: Self::max_file_descriptor_count(),
            system_cpu_load: cpu_load,
            committed_virtual_memory_size: committed_vm,
            free_physical_memory_size: free_mem,
            total_physical_memory_size: total_mem,
            free_swap_space_size: free_swap,
            total_swap_space_size: total_swap,
        }
    }

    /// Returns structured system metrics response for the MetricsService gRPC API.
    pub fn get_system_metrics_resp(&self, _req: &SystemMetricsReq) -> SystemMetricsResp {
        let stats = self.get_stats_resp();
        let disks = self.disks.lock();

        let total_disk_space: u64 = disks.list().iter().map(|d| d.total_space()).sum();
        let free_disk_space: u64 = disks.list().iter().map(|d| d.available_space()).sum();

        let mut sys_metrics = Vec::new();
        sys_metrics.push(Metrics {
            value: Some(Value::SingleMetrics(SingleMetrics {
                name: "system_cpu_load".to_string(),
                value: Some(format!("{:.4}", stats.system_cpu_load)),
            })),
        });
        sys_metrics.push(Metrics {
            value: Some(Value::SingleMetrics(SingleMetrics {
                name: "open_file_descriptor_count".to_string(),
                value: Some(stats.open_file_descriptor_count.to_string()),
            })),
        });
        sys_metrics.push(Metrics {
            value: Some(Value::SingleMetrics(SingleMetrics {
                name: "max_file_descriptor_count".to_string(),
                value: Some(stats.max_file_descriptor_count.to_string()),
            })),
        });
        sys_metrics.push(Metrics {
            value: Some(Value::SingleMetrics(SingleMetrics {
                name: "free_physical_memory_size".to_string(),
                value: Some(stats.free_physical_memory_size.to_string()),
            })),
        });
        sys_metrics.push(Metrics {
            value: Some(Value::SingleMetrics(SingleMetrics {
                name: "total_physical_memory_size".to_string(),
                value: Some(stats.total_physical_memory_size.to_string()),
            })),
        });
        sys_metrics.push(Metrics {
            value: Some(Value::SingleMetrics(SingleMetrics {
                name: "free_swap_space_size".to_string(),
                value: Some(stats.free_swap_space_size.to_string()),
            })),
        });
        sys_metrics.push(Metrics {
            value: Some(Value::SingleMetrics(SingleMetrics {
                name: "total_swap_space_size".to_string(),
                value: Some(stats.total_swap_space_size.to_string()),
            })),
        });
        sys_metrics.push(Metrics {
            value: Some(Value::SingleMetrics(SingleMetrics {
                name: "total_disk_space_bytes".to_string(),
                value: Some(total_disk_space.to_string()),
            })),
        });
        sys_metrics.push(Metrics {
            value: Some(Value::SingleMetrics(SingleMetrics {
                name: "free_disk_space_bytes".to_string(),
                value: Some(free_disk_space.to_string()),
            })),
        });

        SystemMetricsResp {
            metrics: Some(MetricsGroup {
                name: "system".to_string(),
                metrics: sys_metrics,
            }),
        }
    }
}
