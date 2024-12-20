use actix_web::{
  get,
  post,
  web::{Data, Path},
  HttpResponse, Responder, Result
};
use mime_guess;

use crate::{SharedState, StaticFiles};

#[get("/{filename:.*}")]
async fn serve_static_files(path: Path<String>) -> Result<HttpResponse> {
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

#[post("/api/counter")]
pub async fn counter(data: Data<SharedState>) -> impl Responder {
    let new_count = {
        let mut counter = data.counter.lock().unwrap();
        *counter += 1;
        println!("Counter: {}", *counter);
        counter
    };
    println!("Data: {:?}", data);
    HttpResponse::Ok().body(new_count.to_string())
}