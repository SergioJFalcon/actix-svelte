use actix_web::{
  App,
  HttpServer,
  web::Data,
};
use std::sync::Arc;
use std::net::TcpListener;

mod handlers;
mod utils;

use utils::AppState;
use handlers::{serve_static_files, counter};

#[actix_web::main]
pub async fn actix_server_app() -> std::io::Result<()> {
    let hostname: &str = "localhost";
    let port: i32 = 8090;
    let listener: TcpListener = TcpListener::bind(format!("{hostname}:{port}")).expect("Failed to bind to address");
    let shared_state: Data<Arc<AppState>> = Data::new(AppState::new("Room Condition Status"));
    
    println!("\t🚀 Server started successfully");
    println!("\t🌍 Listening on: http://{}:{}/", hostname, port);

    HttpServer::new(move || {
        App::new()
            .app_data(shared_state.clone())
            .service(serve_static_files)
            .service(counter)
    })
    .listen(listener)?
    .run()
    .await
}