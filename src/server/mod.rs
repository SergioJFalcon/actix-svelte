mod api;
mod utils;

use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware, web::Data};
use actix_web::{http, web};
use std::io::Result;
use std::net::TcpListener;
use std::sync::Arc;
use tokio::signal;
use tokio_util::sync::CancellationToken;
use utils::AppState;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub struct Application {
    pub hostname: String,
    pub port: u16,
    pub server: actix_web::dev::Server,
    pub cancel_token: CancellationToken,
}
impl Application {
    pub async fn build(hostname: String, port: u16) -> Result<Self> {
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
    let app_name: String = dotenvy::var("APP_NAME").unwrap_or_else(|_| "Default Room Condition Status".to_string());
    let shared_state: Data<Arc<AppState>> = Data::new(AppState::new(app_name.as_str()));

    let openapi: utoipa::openapi::OpenApi = api::swagger::ApiDocumentation::openapi();

    let server_app: actix_web::dev::Server = HttpServer::new(move || {
        App::new()
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
