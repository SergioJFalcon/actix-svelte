pub mod server;
pub mod telemetry;

use std::sync::atomic::AtomicBool;

use tracing_appender::non_blocking::WorkerGuard;

// Global flag for shutdown coordination
pub static RUNNING: AtomicBool = AtomicBool::new(true);
pub static PAUSED: AtomicBool = AtomicBool::new(false);

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Load the .env file
    dotenvy::dotenv().expect("Error loading env file");
    // Initialize the logging system first thing
    let _log_guard: WorkerGuard = telemetry::setup_logging().expect("Failed to set up logging");
    tracing::info!("Logging initialized successfully");

    let hostname: String = dotenvy::var("HOST").expect("HOST must be set");
    let port: u16 = dotenvy::var("ASPNETCORE_PORT")
        .or_else(|_| dotenvy::var("PORT"))
        .unwrap_or_else(|_| "5000".to_string()) // Default to 5000 if nothing is set
        .parse::<u16>()
        .expect("PORT must be a number");
    
    tracing::info!("WHAT IS THE HOST: {}", hostname);
    tracing::info!("WHAT IS THE PORT: {}", port);

    let application: server::Application = server::Application::build(hostname, port).await?;
    tracing::event!(target: "backend", tracing::Level::INFO, "Listening on http://127.0.0.1:{}/", application.port());

    application.run_until_stopped().await?;

    Ok(())
}