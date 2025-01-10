use std::sync::atomic::Ordering;

use actix_web::{
  get,
  post,
  web::{Data, Path},
  HttpResponse, Responder, Result
};
use mime_guess;

use crate::{server::utils::{SharedState, StaticFiles}, PAUSED};

#[get("/{filename:.*}")]
pub async fn serve_static_files(path: Path<String>) -> Result<HttpResponse> {
    let filename = path.into_inner();
    
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

#[get("/api/health")]
pub async fn health_check() -> impl Responder {
  if PAUSED.load(Ordering::SeqCst) {
      HttpResponse::ServiceUnavailable().body("Service is paused")
  } else {
      HttpResponse::Ok().body("Service is running")
  }
}

#[get("/api/state")]
pub async fn get_app_state(data: Data<SharedState>) -> impl Responder {
    match data.to_pretty_json() {
        Ok(json_data) => HttpResponse::Ok().body(json_data),
        Err(_) => {
            HttpResponse::InternalServerError().body("Error serializing app state")
        }
    }
}

#[post("/api/counter")]
pub async fn counter(data: Data<SharedState>) -> impl Responder {
    data.counter.fetch_add(1, Ordering::Relaxed);

    let local_count: usize = data.local_count.get();
    data.local_count.set(local_count + 1);

    data.global_count.fetch_add(1, Ordering::Relaxed);
    
    HttpResponse::Ok().body(
      data.counter.load(Ordering::Relaxed).to_string()
    )
}