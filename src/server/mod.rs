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
