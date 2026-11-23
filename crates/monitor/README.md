# X-Road Monitor (`monitor`)

`monitor` implements the Environmental and Operational Monitoring daemon for the Security Server. It gathers live host system statistics, stores operational request records, and exposes metrics via Poem (Prometheus & OpenTelemetry) and Tonic gRPC.

## Architecture & Features

- **Live Environmental Collector (`SystemCollector`)**:
  - Uses `sysinfo` to measure CPU load, physical memory, swap, disk space, and open file descriptors.
  - Implements gRPC `MonitorService` (`GetStats`) and `MetricsService` (`GetMetrics`).
- **Operational Data Store (`OperationalStore`)**:
  - In-memory circular buffer storing `OperationalDataRecordProto` records.
  - Calculates aggregated interval buckets for `GetOperationalDataIntervals`.
  - Serializes standard X-Road SOAP XML operational data responses for `QueryOperationalData`.
- **Poem HTTP Server with Prometheus & OpenTelemetry**:
  - Native `poem::endpoint::PrometheusExporter` at `/metrics` and `/metrics/exporter`.
  - Gauges for `xroad_system_cpu_load`, `xroad_system_memory_free_bytes`, `xroad_proxy_requests_total`, `xroad_proxy_request_duration_seconds`, etc.
  - `poem::middleware::OpenTelemetryMetrics` & `poem::middleware::OpenTelemetryTracing` instrumentation.
  - REST endpoints at `/health` and `/status` returning JSON diagnostic information.
