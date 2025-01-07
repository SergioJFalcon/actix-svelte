use actix_web::{
  App,
  HttpServer,
  web::Data,
};
use actix_rt::signal;
use std::sync::Arc;
use std::net::TcpListener;

mod handlers;
mod utils;

use utils::AppState;
use handlers::{serve_static_files, counter};

pub async fn actix_server_app() -> actix_web::dev::Server {
    let hostname: &str = "localhost";
    let port: i32 = 8090;
    let listener: TcpListener = TcpListener::bind(format!("{hostname}:{port}")).expect("Failed to bind to address");
    let shared_state: Data<Arc<AppState>> = Data::new(AppState::new("Room Condition Status"));
    
    println!("\t🚀 Server started successfully");
    println!("\t🌍 Listening on: http://{}:{}/", hostname, port);

    let server_app: actix_web::dev::Server = HttpServer::new(move || {
        App::new()
            .app_data(shared_state.clone())
            .service(serve_static_files)
            .service(counter)
    })
    .listen(listener).expect("Failed to listen on address")
    .run();

    server_app
}

pub async fn actix_server_handle(server_app: &actix_web::dev::Server) -> actix_web::dev::ServerHandle {
    server_app.handle()
}

pub async fn start_actix_server(server_app: actix_web::dev::Server) -> std::io::Result<()> {
    // Run the server
    server_app.await
}

// Spawn signal handlers
// actix_rt::spawn(async move {
//   // Wait for SIGTERM
//   signal::ctrl_c().await.expect("Failed to listen for SIGTERM");
//   println!("SIGTERM received, shutting down server");

//   // Start graceful server shutdown
//   srv.stop(true).await;
// });