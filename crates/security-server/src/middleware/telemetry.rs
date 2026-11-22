use opentelemetry::{global, trace::TracerProvider as _};
use opentelemetry_sdk::{
    Resource,
    propagation::TraceContextPropagator,
    trace::{SdkTracer, SdkTracerProvider},
};

/// Initializes the OpenTelemetry tracer provider and propagator for the service.
pub fn init_tracer(service_name: &str) -> SdkTracer {
    global::set_text_map_propagator(TraceContextPropagator::new());
    let mut builder = SdkTracerProvider::builder().with_resource(
        Resource::builder()
            .with_service_name(service_name.to_string())
            .build(),
    );

    if let Ok(exporter) = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
    {
        builder = builder.with_batch_exporter(exporter);
    }

    let provider = builder.build();
    let tracer = provider.tracer(service_name.to_string());
    global::set_tracer_provider(provider);
    tracer
}
