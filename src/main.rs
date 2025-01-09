mod server;

use std::env;
use std::net::TcpListener;
use std::sync::atomic::AtomicBool;

// Global flag for shutdown coordination
pub static RUNNING: AtomicBool = AtomicBool::new(true);
pub static PAUSED: AtomicBool = AtomicBool::new(false);

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  std::env::set_var("RUST_LOG", "actix_web=debug");
  std::env::set_var("RUST_BACKTRACE", "1");

  let hostname: &str = env!("HOST");
  // let port: i32 = env!("PORT").parse().expect("PORT must be a number");
  let port: String = match std::env::var("ASPNETCORE_PORT") {
    Ok(port) => port,
    Err(_) => {
        println!("Unable to match env var setting to default port :4000");
        env!("PORT").parse().expect("PORT must be a number")
    }
  };
  
  let listener: TcpListener = TcpListener::bind(format!("{hostname}:{port}")).expect("Failed to bind to address");

  let server_app: actix_web::dev::Server = server::actix_server_app(listener).await;
  
  server_app.await
}