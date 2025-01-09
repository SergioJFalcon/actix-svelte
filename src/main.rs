mod server;

use std::env;
use std::net::TcpListener;
use std::sync::atomic::AtomicBool;

// Global flag for shutdown coordination
pub static RUNNING: AtomicBool = AtomicBool::new(true);
pub static PAUSED: AtomicBool = AtomicBool::new(false);

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  let hostname: &str = env!("HOST");
  let port: i32 = env!("PORT").parse().expect("PORT must be a number");
  let listener: TcpListener = TcpListener::bind(format!("{hostname}:{port}")).expect("Failed to bind to address");

  let server_app: actix_web::dev::Server = server::actix_server_app(listener).await;
  
  server_app.await
}