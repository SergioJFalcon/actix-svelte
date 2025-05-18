pub mod api;
pub mod utils;

use actix_cors::Cors;
use actix_web::Error;
use actix_web::{App, HttpServer, middleware, web::Data};
use actix_web::{http, web, FromRequest, HttpRequest};
use chrono::{DateTime, Utc};
use futures::future::{ready, Ready};
use sqlx::{Pool, Sqlite, SqlitePool};
use std::env;
use std::io::Result;
use std::net::TcpListener;
use tokio::signal;
use tokio_util::sync::CancellationToken;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use rust_embed::RustEmbed;
use rusty_paseto::prelude::*;
use serde::{Deserialize, Serialize};
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

pub struct AppState {
    pub app_name: String,
    pub app_version: String,
    pub counter: RwLock<i32>,
    pub global_count: RwLock<AtomicUsize>,
    pub secret_key_string: String,
    pub private_key: Key<64>,
    pub public_key: Key<32>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("app_name", &self.app_name)
            .field("app_version", &self.app_version)
            .field("counter", &self.counter)
            .field("global_count", &self.global_count)
            .finish_non_exhaustive()
    }
}

// Serializable version of the struct
#[derive(Serialize)]
pub struct SerializableAppState<'a> {
    app_name: &'a str,
    app_version: &'a str,
    counter: i32,
    global_counter: usize,
    // private_key: &'a str,
    // public_key: &'a str,
}
impl AppState {
  pub fn new(app_name: &str) -> SharedState {
      let secret_key_string: String = env::var("SECRET_KEY").expect("SECRET_KEY must be set");
      let private_key: Key<64> = Key::<64>::try_from(secret_key_string.clone().as_str()).expect("Failed to parse PASETO secret key");
      // let pk: &[u8] = private_key.to_owned().as_slice();
      // let private_key = PasetoAsymmetricPrivateKey::<V4, Public>::from(pk);
      let public_key: Key<32> = Key::<32>::try_from("1eb9dbbbbc047c03fd70604e0071f0987e16b28b757225c11f00415d0e20b1a2").expect("Failed to parse public key");
      // let public_key = PasetoAsymmetricPublicKey::<V4, Public>::from(&public_key);


      Arc::new(AppState {
          app_name: app_name.to_string(),
          app_version: env!("CARGO_PKG_VERSION").to_string(),
          counter: RwLock::new(0),
          global_count: RwLock::new(AtomicUsize::new(0)),
          secret_key_string,
          private_key,
          public_key 
      })
  }
  
  pub async fn to_serializable(&self) -> SerializableAppState {
    SerializableAppState {
        app_name: &self.app_name,
        app_version: &self.app_version,
        counter: *self.counter.read().await,
        global_counter: self.global_count.read().await.load(Ordering::SeqCst),
        // private_key: hex::encode(self.private_key.as_ref()).as_str(),
        // public_key: hex::encode(self.public_key.as_ref()).as_str(),
    }
}
}

pub type SharedState = Arc<AppState>;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub token: String,
    pub expiration: DateTime<Utc>
}

impl FromRequest for AuthenticatedUser {
    type Error = actix_web::Error;
    type Future = Ready<std::result::Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        if let Some(cookie) = req.cookie("auth_token") { // Assuming you named your cookie "auth_token"
            let token = cookie.value();
            let p_key = req.app_data::<web::Data<SharedState>>()
                .map(|data| data.public_key.clone())
                .unwrap_or_default();
            let public_key: PasetoAsymmetricPublicKey<'_, V4, Public> = PasetoAsymmetricPublicKey::<V4, Public>::from(&p_key);

            tracing::info!("Extracted token from cookie: {}", token);
            let parsed_token_json = PasetoParser::<V4, Public>::default()
            .parse(&token, &public_key)
            .expect("Failed to parse token");
            // .set_footer(Footer::from("Footer example"))

            // print the parsed token for debugging
            tracing::info!("Parsed token: {:?}", parsed_token_json);
            // check if the expiration in the token is valid
            let expiration: &serde_json::Value = &parsed_token_json["exp"];
            println!("Expiration value: {:?}", expiration);
            // Check if the expiration field exists and is valid

            if expiration.is_null() {
                return ready(Err(actix_web::error::ErrorUnauthorized("Token does not have an expiration")));
            }
            // Parsed token: Object {"aud": String("custoemrs"), "data": String("this is a secret message"), "exp": String("2025-05-19T06:34:46.993882200+00:00"), "iat": String("2025-05-18T06:34:46.994121400+00:00"), "iss": String("me"), "jti": String("me"), "nbf": String("2025-05-18T06:34:46.994173900+00:00"), "sub": String("my local subjects"), "username": String("pass123")}
            // We are expecting expiration to be a String(DateTime<Utc>), so we need to parse it
            let expiration_str: &str = expiration.as_str().unwrap_or_default();
            let expiration: DateTime<Utc> = match DateTime::parse_from_rfc3339(expiration_str) {
                Ok(dt) => dt.with_timezone(&Utc),
                Err(_) => return ready(Err(actix_web::error::ErrorUnauthorized("Invalid expiration format"))),
            };
            tracing::info!("Token expiration: {}", expiration);
            // check if its expired, i.e., if the expiration is in the past
            if expiration < Utc::now() {
                return ready(Err(actix_web::error::ErrorUnauthorized("Token has expired")));
            } else {
                tracing::info!("Token is valid and not expired");
                return ready(Ok(AuthenticatedUser {
                    token: token.to_string(),
                    expiration,
                }));
            }
        } else {
            tracing::warn!("No auth_token cookie found in request");
            return ready(Err(actix_web::error::ErrorUnauthorized("Missing auth_token cookie")));
        }

        // ready(Err(actix_web::error::ErrorUnauthorized("Missing or invalid token")))
    }
    
    fn extract(req: &HttpRequest) -> Self::Future {
        Self::from_request(req, &mut actix_web::dev::Payload::None)
    }
}

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
