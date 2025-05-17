pub mod api;
pub mod utils;

use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware, web::Data};
use actix_web::{http, web};
use sqlx::{Pool, Sqlite, SqlitePool};
use std::io::Result;
use std::net::TcpListener;
use tokio::signal;
use tokio_util::sync::CancellationToken;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use rust_embed::RustEmbed;
use serde::Serialize;
use std::{
  sync::{
    atomic::{AtomicUsize, Ordering}, 
    Arc
  }
};
use tokio::sync::RwLock;

#[derive(RustEmbed)]
#[folder = "client/build"]
pub struct StaticFiles;

#[derive(Debug)]
pub struct DatabaseState {
    pub pool: Pool<Sqlite>,
}

#[derive(Debug)]
pub struct AppState {
    pub app_name: String,
    pub app_version: String,
    pub counter: RwLock<i32>,
    pub global_count: RwLock<AtomicUsize>,
}

// Serializable version of the struct
#[derive(Serialize)]
pub struct SerializableAppState<'a> {
    app_name: &'a str,
    app_version: &'a str,
    counter: i32,
    global_counter: usize,
}
impl AppState {
  pub fn new(app_name: &str) -> SharedState {
      Arc::new(AppState {
          app_name: app_name.to_string(),
          app_version: env!("CARGO_PKG_VERSION").to_string(),
          // counter: Arc::new(AtomicUsize::new(0)),
          counter: RwLock::new(0),
          global_count: RwLock::new(AtomicUsize::new(0)),
      })
  }
  
  pub async fn to_serializable(&self) -> SerializableAppState {
    SerializableAppState {
        app_name: &self.app_name,
        app_version: &self.app_version,
        counter: *self.counter.read().await,
        global_counter: self.global_count.read().await.load(Ordering::SeqCst),
    }
}
}

pub type SharedState = Arc<AppState>;

pub struct Application {
    pub hostname: String,
    pub port: u16,
    pub server: actix_web::dev::Server,
    pub cancel_token: CancellationToken,
}
impl Application {
    pub async fn build(hostname: String, port: u16, _test_pool: Option<Pool<Sqlite>>) -> Result<Self> {
        let listener: TcpListener =
            TcpListener::bind(format!("{}:{}", hostname, port)).expect("Failed to bind address");
        let cancel_token: CancellationToken = CancellationToken::new();
        let server: actix_web::dev::Server = build_server_app(listener, cancel_token.clone()).await?;

        Ok(Self {
            hostname,
            port,
            server,
            cancel_token,
        })
    }

    pub fn hostname(&self) -> String {
        self.hostname.clone()
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<()> {
        println!("\n\t✅ Database connected successfully");
        println!("\t🚀 Server started successfully");
        println!(
            "\t🌍 Listening on: http://{}:{}",
            self.hostname(),
            self.port()
        );
        println!(
            "\t🔗 Swagger Docs: http://{}:{}/api/swagger-ui/#/\n",
            self.hostname(),
            self.port()
        );

        // Start listening for shutdown
        let shutdown_signal = async {
            signal::ctrl_c().await.expect("Failed to listen for ctrl_c");
            self.cancel_token.cancel(); // Notify all tasks
            println!("Shutdown signal received");
        };

        // Run the server and shutdown signal in parallel
        tokio::select! {
                _ = self.server => Ok(()),
                _ = shutdown_signal => Ok(())
        }

        // self.server.await
        // Ok(())
    }
}

pub async fn build_server_app(
    listener: TcpListener,
    _cancel_token: CancellationToken,
) -> Result<actix_web::dev::Server> {
    let database_url: String = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool: Pool<Sqlite> = SqlitePool::connect(&database_url).await.expect("Failed to connect to database");
    let db_state: Data<DatabaseState> = Data::new(DatabaseState { pool: pool.clone() });
    let app_name: String = dotenvy::var("APP_NAME").unwrap_or_else(|_| "App Template".to_string());
    let shared_state: Data<Arc<AppState>> = Data::new(AppState::new(app_name.as_str()));

    let openapi: utoipa::openapi::OpenApi = api::swagger::ApiDocumentation::openapi();

    let server_app: actix_web::dev::Server = HttpServer::new(move || {
        App::new()
            .app_data(db_state.clone())
            .app_data(shared_state.clone())
            .wrap(middleware::Logger::default())
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:5173")
                    .allowed_methods(vec!["GET"])
                    .allowed_headers(vec![http::header::CONTENT_TYPE])
                    .max_age(3600),
            )
            .service(
                SwaggerUi::new("/api/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi.clone()),
            )
            .service(
                web::scope("/api")
                    .configure(api::routes::app_services)
                    .configure(api::routes::auth_services)
            )
            .service(api::handlers::serve_static_files)
    })
    .listen(listener)
    .expect("Failed to listen on address")
    .workers(1)
    .shutdown_timeout(5)
    .run();

    Ok(server_app)
}
