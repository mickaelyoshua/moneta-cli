use std::fs;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

/// Init telemetry.
/// Terminal: silent/JSON for humans. File: audits for AI.
/// Fallback to warning if log dir creation fails.
pub fn init_telemetry() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn,moneta=debug"));

    // Layer 1: Stderr. Preserves stdout for JSON output.
    let stderr_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_span_events(FmtSpan::NONE)
        .compact()
        .with_filter(env_filter);

    let mut layers = vec![stderr_layer.boxed()];

    // Logs dir.
    let log_dir = dirs::state_dir()
        .map(|p| p.join("moneta-cli").join("logs"))
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp/moneta-cli-logs"));

    // Create dir or warn.
    match fs::create_dir_all(&log_dir) {
        Ok(_) => {
            // Layer 2: File. Daily rotation.
            let file_appender = RollingFileAppender::builder()
                .rotation(Rotation::DAILY)
                .filename_prefix("moneta.app")
                .filename_suffix("log")
                .max_log_files(30)
                .build(log_dir)
                .expect("Failed to initialize rolling file appender");

            // JSON format for automated tools.
            let file_layer = fmt::layer()
                .with_writer(file_appender)
                .json()
                .with_filter(EnvFilter::new("info,sqlx=warn,moneta=debug"));

            layers.push(file_layer.boxed());
        }
        Err(e) => {
            eprintln!("Warn: Log dir creation failed. File logging disabled: {}", e);
        }
    }

    // Init global registry.
    let _ = tracing_subscriber::registry().with(layers).try_init();
}
