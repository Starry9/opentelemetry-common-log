#[cfg(feature = "otlp")]
use std::time::Duration;

use opentelemetry::global;
use opentelemetry_sdk::trace::Tracer;
#[cfg(any(feature = "otlp", feature = "datadog"))]
use opentelemetry_sdk::trace::{self, RandomIdGenerator, Sampler};
#[cfg(feature = "otlp")]
use opentelemetry_sdk::Resource;
#[cfg(feature = "otlp")]
use opentelemetry::KeyValue;
use opentelemetry_sdk::propagation::TraceContextPropagator;
#[cfg(feature = "datadog")]
use opentelemetry_datadog::DatadogPropagator;
#[cfg(feature = "otlp")]
use opentelemetry_otlp::WithExportConfig;
use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

#[cfg(feature = "jaeger")]
fn get_opentelemetry_tracer(name: &str, opentelemetry_endpoint: &str) -> Tracer {
    opentelemetry_jaeger::new_agent_pipeline()
        .with_endpoint(opentelemetry_endpoint)
        .with_service_name(name)
        .install_batch(opentelemetry::runtime::Tokio)
        .unwrap()
}

#[cfg(feature = "datadog")]
fn get_opentelemetry_tracer(name: &str, opentelemetry_endpoint: &str) -> Tracer {
    let config = trace::config()
        .with_sampler(Sampler::AlwaysOn)
        .with_id_generator(RandomIdGenerator::default());

    opentelemetry_datadog::new_pipeline()
        .with_service_name(name)
        .with_api_version(opentelemetry_datadog::ApiVersion::Version05)
        .with_trace_config(config)
        .with_agent_endpoint(opentelemetry_endpoint)
        .install_batch(opentelemetry::runtime::Tokio)
        .unwrap()
}

#[cfg(feature = "otlp")]
fn get_opentelemetry_tracer(name: &str, opentelemetry_endpoint: &str) -> Tracer {
    // create a OTLP exporter builder. Configure it as you need.
    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(opentelemetry_endpoint)
        .with_timeout(Duration::from_secs(3));
    let server_name = name.to_string();
    let otlp_config = trace::config()
        .with_sampler(Sampler::AlwaysOn)
        .with_id_generator(RandomIdGenerator::default())
        .with_max_events_per_span(64)
        .with_max_attributes_per_span(16)
        .with_resource(Resource::new(vec![KeyValue::new("service.name", server_name)]));
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(otlp_exporter)
        .with_trace_config(otlp_config)
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .unwrap()
}

/// Compose multiple layers into a `tracing`'s subscriber.
///
/// # Implementation Notes
///
/// We are using `impl Subscriber` as return type to avoid having to
/// spell out the actual type of the returned subscriber, which is
/// indeed quite complex.
/// We need to explicitly call out that the returned subscriber is
/// `Send` and `Sync` to make it possible to pass it to `init_subscriber`
/// later on.
fn get_telemetry_subscriber(name: String, log_level: String, opentelemetry_endpoint: String) -> impl Subscriber + Send + Sync {
    // Set up the opentelemetry global context (in order to inject into distributed requests to trace distributed services)
    #[cfg(feature = "datadog")]
    global::set_text_map_propagator(DatadogPropagator::new());
    #[cfg(feature = "otlp")]
    global::set_text_map_propagator(TraceContextPropagator::new());

    let tracer = get_opentelemetry_tracer(&name, &opentelemetry_endpoint);
    // Create a tracing layer with the configured tracer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // We are falling back to printing all spans at info-level or above
    // if the RUST_LOG environment variable has not been set.
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));
    let formatting_layer = BunyanFormattingLayer::new(name, std::io::stdout);

    // The `with` method is provided by `SubscriberExt`, an extension
    // trait for `Subscriber` exposed by `tracing_subscriber`
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
        .with(telemetry)
}

fn get_tracing_subscriber_with_json(name: String, log_level: String) -> impl Subscriber + Send + Sync {
    let formatting_layer = BunyanFormattingLayer::new(name, std::io::stdout);
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

fn get_tracing_subscriber_without_json(_name: String, log_level: String) -> impl Subscriber + Send + Sync {
    let formatting_layer = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true);
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));
    Registry::default()
        .with(env_filter)
        .with(formatting_layer)
}

/// Register a subscriber as global default to process span data.
///
/// It should only be called once!
fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // Redirect all `log`'s events to our subscriber
    LogTracer::init().expect("Failed to set logger");
    // `set_global_default` can be used by applications to specify
    // what subscriber should be used to process spans.
    set_global_default(subscriber).expect("Failed to set subscriber");
}

pub fn init_log(app_name: String, log_level: String, endpoint: Option<String>, with_json: bool) {
    if endpoint.is_some() {
        // opentelemetry endpoint is set, use opentelemetry, and log format must json
        let subscriber = get_telemetry_subscriber(app_name, log_level, endpoint.unwrap());
        init_subscriber(subscriber);
    } else {
        if with_json {
            let subscriber = get_tracing_subscriber_with_json(app_name, log_level);
            init_subscriber(subscriber);
        } else {
            let subscriber = get_tracing_subscriber_without_json(app_name, log_level);
            init_subscriber(subscriber);
        }
    }
}

pub fn shutdown_tracer() {
    global::shutdown_tracer_provider();
}
