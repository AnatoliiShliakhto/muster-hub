use crate::error::LoggerError;
use opentelemetry::{KeyValue, global};
use opentelemetry_sdk::{
    Resource,
    trace::{SdkTracerProvider, TraceError},
};

/// A guard that shuts down the global `OpenTelemetry` tracer provider on a drop.
#[derive(Debug)]
pub struct OpenTelemetryGuard {
    provider: SdkTracerProvider,
}

impl OpenTelemetryGuard {
    /// Explicitly shuts down the tracer provider.
    pub fn shutdown(self) {
        drop(self);
    }
}

impl Drop for OpenTelemetryGuard {
    fn drop(&mut self) {
        let _ = self.provider.shutdown();
    }
}

/// Installs an OTLP tracer provider and sets it as the global tracer provider.
///
/// The exporter respects standard OTEL environment variables such as
/// `OTEL_EXPORTER_OTLP_ENDPOINT`, `OTEL_SERVICE_NAME`, and `OTEL_RESOURCE_ATTRIBUTES`.
///
/// # Errors
/// Returns [`LoggerError::InvalidConfiguration`] if `service_name` is empty.
/// Returns [`LoggerError::OpenTelemetry`] if the OTLP pipeline fails to initialize.
///
/// # Examples
/// ```rust,no_run
/// use mhub_logger::init_otlp_tracer;
///
/// let _otel = init_otlp_tracer("my-app")?;
/// # Ok::<(), mhub_logger::LoggerError>(())
/// ```
pub fn init_otlp_tracer(
    service_name: impl Into<String>,
) -> Result<OpenTelemetryGuard, LoggerError> {
    let service_name = service_name.into();
    if service_name.trim().is_empty() {
        return Err(LoggerError::InvalidConfiguration {
            message: "service_name cannot be empty".into(),
            context: None,
        });
    }

    let resource = Resource::builder_empty()
        .with_attributes([KeyValue::new("service.name", service_name)])
        .build();

    let exporter =
        opentelemetry_otlp::SpanExporter::builder().with_tonic().build().map_err(|source| {
            LoggerError::OpenTelemetry {
                source: TraceError::Other(Box::new(source)),
                context: Some("Failed to build OTLP span exporter".into()),
            }
        })?;

    let provider =
        SdkTracerProvider::builder().with_batch_exporter(exporter).with_resource(resource).build();

    global::set_tracer_provider(provider.clone());

    Ok(OpenTelemetryGuard { provider })
}
