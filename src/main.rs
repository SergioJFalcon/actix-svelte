use actix_web::{
  App,
  HttpServer,
  web::Data,
};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::net::TcpListener;

mod handlers;

#[derive(RustEmbed)]
#[folder = "client/build"]
pub struct StaticFiles;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct AppState {
    pub app_name: String,
    pub app_version: String,
    pub counter: Mutex<i32>,
}

// Serializable version of the struct
#[derive(Serialize)]
struct SerializableAppState<'a> {
    app_name: &'a str,
    app_version: &'a str,
    counter: i32,
}

impl AppState {
    pub fn new(app_name: &str) -> SharedState {
        Arc::new(AppState {
            app_name: app_name.to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            counter: Mutex::new(0),
        })
    }
    pub fn to_pretty_json(&self) -> Result<Vec<u8>, serde_json::Error> {
        let serializable = SerializableAppState {
            app_name: &self.app_name,
            app_version: &self.app_version,
            counter: *self.counter.lock().unwrap(),
        };

        serde_json::to_vec_pretty(&serializable)
    }
}

pub type SharedState = Arc<AppState>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let hostname: &str = "localhost";
    let port: i32 = 8090;
    let listener: TcpListener = TcpListener::bind(format!("{hostname}:{port}")).expect("Failed to bind to address");
    let shared_state: Data<Arc<AppState>> = Data::new(AppState::new("Room Condition Status"));
    
    println!("\t🚀 Server started successfully");
    println!("\t🌍 Listening on: http://{}:{}/", hostname, port);

    HttpServer::new(move || {
        App::new()
            .app_data(shared_state.clone())
            .service(handlers::serve_static_files)
            .service(handlers::counter)
    })
    .listen(listener)?
    .run()
    .await
}