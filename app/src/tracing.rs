#[cfg(target_arch = "wasm32")]
pub fn init_tracing() {
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::{filter, layer::SubscriberExt, Layer, Registry};
    use tracing_wasm::{WASMLayer, WASMLayerConfigBuilder};

    // Set up panic hook for better error messages in browser
    console_error_panic_hook::set_once();

    // Get package name for filtering
    const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");

    // Create filter that only logs from our app at TRACE level, others at WARN
    let filter_fn = filter::filter_fn(|metadata| {
        let mut level_filter = LevelFilter::WARN;
        if metadata.target().starts_with(PACKAGE_NAME) {
            level_filter = if cfg!(debug_assertions) {
                LevelFilter::DEBUG
            } else {
                LevelFilter::INFO
            };
        }
        metadata.level() <= &level_filter
    });

    // Configure WASM layer - pretty in debug, structured in release
    let config = if cfg!(debug_assertions) {
        // Pretty output for development
        WASMLayerConfigBuilder::new()
            .set_report_logs_in_timings(false)
            .set_console_config(tracing_wasm::ConsoleConfig::ReportWithConsoleColor)
            .build()
    } else {
        // More structured output for release
        WASMLayerConfigBuilder::new()
            .set_report_logs_in_timings(false)
            .set_console_config(tracing_wasm::ConsoleConfig::ReportWithoutConsoleColor)
            .build()
    };

    tracing::subscriber::set_global_default(
        Registry::default().with(WASMLayer::new(config).with_filter(filter_fn)),
    )
    .expect("WASM tracing initialization failed");
}

#[cfg(not(target_arch = "wasm32"))]
pub fn init_tracing() {
    use tracing_subscriber::fmt;
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    // Setup environment filter
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
            EnvFilter::new("debug")
        } else {
            EnvFilter::new("info")
        }
    });

    if cfg!(debug_assertions) {
        // Pretty output for development
        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                fmt::layer()
                    .pretty()
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_file(true)
                    .with_line_number(true),
            )
            .init();
    } else {
        // JSON output for production
        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                fmt::layer()
                    .json()
                    .with_target(true)
                    .with_current_span(true)
                    .with_span_list(true),
            )
            .init();
    }
}
