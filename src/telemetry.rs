use std::path::PathBuf;
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, time::LocalTime},
};

fn get_log_directory() -> PathBuf {
    // if not in IIS context, use a logs folder int he current directory
    let log_dir: PathBuf = PathBuf::from("./logs");
    if !log_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&log_dir) {
            tracing::error!("Failed to create log directory: {}", e);
        }
    }

    log_dir
}

pub fn setup_logging() -> Result<WorkerGuard, Box<dyn std::error::Error>> {
    let log_dir: PathBuf = get_log_directory();

    // Create file appender with daily reotation (generates files like: 'app-2025-05-02.log')
    let file_appender: RollingFileAppender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_suffix("log")
        .build(log_dir)
        .expect("Failed to create file appender");

    // Create a non-blocking writer to avoid performance issues
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Create a custom time format for the log messages
    let time_format: Vec<time::format_description::BorrowedFormatItem<'_>> =
        time::format_description::parse("[hour]:[minute]:[second]")
            .expect("format string should be valid!");
    let timer: LocalTime<Vec<time::format_description::BorrowedFormatItem<'_>>> =
        LocalTime::new(time_format);

    // Create environment filter
    let env_filter: EnvFilter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // File layer - no ANSI colors
    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_timer(timer.clone())
        .with_level(true)
        .with_target(true)
        .with_file(true)
        .with_line_number(true);

    // Combine both layers with the filter
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    // Create a registry and add the layers to it, only adding the file layer as we add the console later
    let registry = tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer);

    // Only add console layer in debug builds
    #[cfg(debug_assertions)]
    let registry = {
        // Console layer - only in debug/development builds
        let console_layer = fmt::layer()
            .with_writer(std::io::stdout)
            .with_ansi(true) // Enable colors for console
            .with_timer(timer)
            .with_level(true)
            .with_target(true);

        registry.with(console_layer)
    };

    // Initialize the registry
    registry.init();

    // log the startup info with with service details
    tracing::info!("Service started with PID: {}", std::process::id());
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "Service version: {} starting...",
        env!("CARGO_PKG_VERSION")
    );

    tracing::debug!("This is a debug message I want displayed in the log file.");
    tracing::info!("This is an info message I want displayed in the log file.");
    tracing::warn!("This is a warning message I want displayed in the log file.");
    tracing::error!("This is an error message I want displayed in the log file.");

    Ok(guard)
}
