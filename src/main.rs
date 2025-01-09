mod server;

use std::sync::atomic::AtomicBool;

// Global flag for shutdown coordination
pub static RUNNING: AtomicBool = AtomicBool::new(true);
pub static PAUSED: AtomicBool = AtomicBool::new(false);

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  let server_app: actix_web::dev::Server = server::actix_server_app().await;
  
  server_app.await
}