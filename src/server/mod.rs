mod handlers;
mod utils;

use actix_web::{
  App,
  HttpServer,
  web::Data,
};
use std::sync::Arc;
use std::net::TcpListener;

use handlers::{counter, get_app_state, health_check, serve_static_files};
use utils::AppState;

pub async fn actix_server_app(listener: TcpListener) -> actix_web::dev::Server {
    let local_addr: std::net::SocketAddr = listener.local_addr().unwrap();
    let normalized_addr: String = if local_addr.ip().to_string() == "::1" { format!("localhost") } else { local_addr.ip().to_string() };
    let shared_state: Data<Arc<AppState>> = Data::new(AppState::new("Room Condition Status"));
    
    println!("\t🚀 Server started successfully");
    println!("\t🌍 Listening on: http://{}/{}", normalized_addr, local_addr.port());

    let server_app: actix_web::dev::Server = HttpServer::new(move || {
        App::new()
            .app_data(shared_state.clone())
            .service(health_check)
            .service(get_app_state)
            .service(counter)
            .service(serve_static_files)
    })
    .listen(listener).expect("Failed to listen on address")
    .shutdown_timeout(5) // Give 5 seconds for graceful shutdown
    .run();

    server_app
}
