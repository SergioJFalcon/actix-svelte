use std::sync::atomic::Ordering;

use actix_web::{
  get,
  post,
  web::{Data, Path},
  HttpResponse, Responder, Result
};
use mime_guess;

use crate::{server::utils::{SharedState, StaticFiles}, PAUSED};

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

#[tracing::instrument]
#[get("/api/health")]
pub async fn health_check() -> impl Responder {
    tracing::event!(target: "backend", tracing::Level::INFO, "Accessing health-check endpoint.");
    if PAUSED.load(Ordering::SeqCst) {
        HttpResponse::ServiceUnavailable().body("Service is paused")
    } else {
        HttpResponse::Ok().body("Service is running")
    }
}

#[tracing::instrument(
  name = "Get app state",
  skip(data),
  fields(
    counter = data.counter.load(Ordering::Relaxed),
    local_count = data.local_count.get(),
    global_count = data.global_count.load(Ordering::Relaxed)
  )
)]
#[get("/api/state")]
pub async fn get_app_state(data: Data<SharedState>) -> impl Responder {
    tracing::event!(target: "backend", tracing::Level::INFO, "Accessing app state endpoint.");
    match data.to_pretty_json() {
        Ok(json_data) => HttpResponse::Ok().body(json_data),
        Err(_) => {
            HttpResponse::InternalServerError().body("Error serializing app state")
        }
    }
}

#[post("/api/counter")]
pub async fn counter(data: Data<SharedState>) -> impl Responder {
    println!("###############################################################################");
    tracing::event!(target: "backend", tracing::Level::INFO, "Accessing counter endpoint.");
    
		let new_count = {
      let mut counter = data.counter.lock().unwrap();
      *counter += 1;
      counter
  };

  HttpResponse::Ok().body(new_count.to_string())
    // data.counter.fetch_add(1, Ordering::Relaxed);

    // let local_count: usize = data.local_count.get();
    // data.local_count.set(local_count + 1);

    // data.global_count.fetch_add(1, Ordering::Relaxed);
    // println!("Global count: {}", data.global_count.load(Ordering::Relaxed));
    // println!("###############################################################################");
    // HttpResponse::Ok().body(
    //   data.counter.load(Ordering::Relaxed).to_string()
    // )
}