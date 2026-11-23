use opentelemetry::{global, trace::TracerProvider as _};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    Resource,
    propagation::TraceContextPropagator,
    trace::{SdkTracer, SdkTracerProvider},
};

/// Initializes the OpenTelemetry tracer provider and propagator for the service.
///
/// An OTLP trace batch exporter is only registered if an endpoint is explicitly configured
/// via `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT`, `OTEL_EXPORTER_OTLP_ENDPOINT`, or `XROAD_OTEL_ENDPOINT`.
/// If no endpoint is configured or if `OTEL_SDK_DISABLED=true`, an in-process provider without
/// remote export is used, avoiding `BatchSpanProcessor.ExportError` connection failures.
pub fn init_tracer(service_name: &str) -> SdkTracer {
    global::set_text_map_propagator(TraceContextPropagator::new());
    let mut builder = SdkTracerProvider::builder().with_resource(
        Resource::builder()
            .with_service_name(service_name.to_string())
            .build(),
    );

    // Check if OpenTelemetry SDK is disabled
    let is_disabled = std::env::var("OTEL_SDK_DISABLED")
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(false);

    if !is_disabled {
        // Only attach an OTLP batch exporter if an endpoint is explicitly configured.
        let otel_endpoint = std::env::var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT")
            .or_else(|_| std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT"))
            .or_else(|_| std::env::var("XROAD_OTEL_ENDPOINT"))
            .ok()
            .filter(|ep| !ep.trim().is_empty());

        if let Some(endpoint) = otel_endpoint {
            tracing::info!(
                "Configuring OpenTelemetry OTLP trace exporter for {service_name} to {endpoint}"
            );
            if let Ok(exporter) = opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .with_endpoint(endpoint)
                .build()
            {
                builder = builder.with_batch_exporter(exporter);
            }
        } else {
            tracing::debug!(
                "OpenTelemetry OTLP endpoint not configured for {service_name}; running without remote batch exporter"
            );
        }
    }

    let provider = builder.build();
    let tracer = provider.tracer(service_name.to_string());
    global::set_tracer_provider(provider);
    tracer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_tracer_without_endpoint() {
        // When no OTEL endpoint env vars are set, init_tracer must succeed without panicking
        // and without spawning an exporter that attempts to connect to localhost.
        let tracer = init_tracer("test-service");
        use opentelemetry::trace::Tracer;
        let _span = tracer.start("test-span");
    }
}
