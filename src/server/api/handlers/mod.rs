use actix_web::{
  get,
  post,
  web::{Data, Path},
  HttpResponse, Responder, Result
};
use mime_guess;
use std::sync::atomic::Ordering;

use crate::{server::{SerializableAppState, SharedState, StaticFiles}, PAUSED};

pub mod auth;

#[tracing::instrument]
#[get("/{filename:.*}")]
pub async fn serve_static_files(path: Path<String>) -> Result<HttpResponse> {
    let filename: String = path.into_inner();
    
    // Skip if path starts with 'api'
    if filename.starts_with("api/") {
        return Ok(HttpResponse::NotFound().finish());
    }

    // If no specific file is requested, serve index.html
    let requested_path = if filename.is_empty() {
        "index.html".to_string()
    } else {
        filename
    };

    match StaticFiles::get(&requested_path) {
        Some(content) => {
            let mime_type = mime_guess::from_path(&requested_path)
                .first_or_octet_stream();

            Ok(HttpResponse::Ok()
                .content_type(mime_type.as_ref())
                .body(content.data.into_owned()))
        }
        None => {
            // Serve index.html for non-existent paths (SPA routing)
            match StaticFiles::get("index.html") {
                Some(content) => Ok(HttpResponse::Ok()
                    .content_type("text/html")
                    .body(content.data.into_owned())),
                None => Ok(HttpResponse::NotFound().finish()),
            }
        }
    }
}

#[utoipa::path(
	get,
	path = "/api/health",
	responses(
		(status = 200, description="Returns the health status of the application"),
	)
)]
#[get("/health")]
pub async fn health_check() -> impl Responder {
    tracing::event!(target: "backend", tracing::Level::INFO, "Accessing health-check endpoint.");
    if PAUSED.load(Ordering::SeqCst) {
        HttpResponse::ServiceUnavailable().body("Service is paused")
    } else {
        HttpResponse::Ok().body("Service is running")
    }
}

#[utoipa::path(
	get,
	path = "/api/state",
	responses(
		(status = 200, description="Returns the current state of the application"),
	)
)]
#[get("state")]
pub async fn get_app_state(data: Data<SharedState>) -> impl Responder {
    let json: SerializableAppState<'_> = data.to_serializable().await;

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&json).unwrap())
}

#[utoipa::path(
	post,
	path = "/api/counter",
	responses(
		(status = 200, description="Returns the updated counter value"),
	),
	tag = "counter"
)]
#[post("counter")]
pub async fn counter(data: Data<SharedState>) -> impl Responder {
    let new_count: i32 = {
        let mut counter = data.counter.write().await;
        *counter += 1;

        counter.clone()
    };

    HttpResponse::Ok().body(new_count.to_string())
}