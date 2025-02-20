pub mod server;
pub mod settings;
pub mod telemetry;

use std::sync::atomic::AtomicBool;

// Global flag for shutdown coordination
pub static RUNNING: AtomicBool = AtomicBool::new(true);
pub static PAUSED: AtomicBool = AtomicBool::new(false);

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let settings: settings::Settings = settings::get_settings().expect("Failed to read settings");
    println!("Settings: {:?}", settings);

    let subscriber = telemetry::get_subscriber(settings.clone().debug);
    telemetry::init_subscriber(subscriber);

    // let hostname: &str = env!("HOST");
    // let port: i32 = env!("PORT").parse().expect("PORT must be a number");
    // let port: String = match std::env::var("ASPNETCORE_PORT") {
    //   Ok(port) => port,
    //   Err(_) => {
    //       println!("Unable to match env var setting to default port :4000");
    //       env!("PORT").parse().expect("PORT must be a number")
    //   }
    // };
    // let listener: TcpListener = TcpListener::bind(format!("{hostname}:{port}")).expect("Failed to bind to address");
    // let server_app: actix_web::dev::Server = server::actix_server_app(listener).await?;
    
    let application: server::Application = server::Application::build(settings).await?;

    tracing::event!(target: "backend", tracing::Level::INFO, "Listening on http://127.0.0.1:{}/", application.port());

    application.run_until_stopped().await?;

    Ok(())
}