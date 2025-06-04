
use actix_web::{
  get, post, web::{Data, Json, Path, Payload}, HttpRequest, HttpResponse, Responder, Result
};
use anyhow::Error;
use mime_guess;
use tokio::time::sleep;
use std::{sync::atomic::Ordering, time::Duration};

use crate::{server::{SerializableAppState, SharedState, StaticFiles}};
use actix_svelte::{HEALTH_CHECK_HITS, PAUSED};

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

// #[utoipa::path(
// 	get,
// 	path = "/api/health",
// 	responses(
// 		(status = 200, description="Returns the health status of the application"),
// 	)
// )]
// #[get("/health")]
// pub async fn health_check() -> impl Responder {
//     tracing::event!(target: "backend", tracing::Level::INFO, "Accessing health-check endpoint.");
//     if PAUSED.load(Ordering::SeqCst) {
//         HttpResponse::ServiceUnavailable().body("Service is paused")
//     } else {
//         // Sleep for 15 seconds to simulate a long-running health check
//         tracing::event!(target: "backend", tracing::Level::INFO, "Simulating long-running health check. Sleeping for 5 secs.");
//         tokio::time::sleep(std::time::Duration::from_secs(5)).await;

//         HttpResponse::Ok().body("Service is running")
//     }
// }

#[utoipa::path(
    post,
    path = "/api/pause",
    responses(
        (status = 200, description="Service paused successfully"),
    )
)]
#[post("/pause")]
pub async fn pause_service() -> impl Responder {
    println!("Pausing service...");
    PAUSED.store(true, Ordering::SeqCst);
    tracing::event!(target: "backend", tracing::Level::INFO, "Service has been PAUSED.");
    HttpResponse::Ok().body("Service paused")
}

#[utoipa::path(
    post,
    path = "/api/unpause",
    responses(
        (status = 200, description="Service unpaused successfully"),
    )
)]
#[post("/unpause")]
pub async fn unpause_service() -> impl Responder {
    println!("Unpausing service...");
    PAUSED.store(false, Ordering::SeqCst);
    // Reset the health check hits counter when unpausing, so you can re-test the retry logic
    HEALTH_CHECK_HITS.store(0, Ordering::SeqCst);
    tracing::event!(target: "backend", tracing::Level::INFO, "Service has been UNPAUSED and health check counter reset.");
    HttpResponse::Ok().body("Service unpaused")
}

#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200, description="Returns the health status of the application"),
        (status = 503, description="Service is paused or temporarily unavailable"),
        (status = 500, description="Internal Server Error for testing retries"),
    )
)]
#[get("/health")]
pub async fn health_check(_data: Data<SharedState>) -> impl Responder {
    println!("Health check endpoint hit");
    tracing::event!(target: "backend", tracing::Level::INFO, "Accessing health-check endpoint.");
    println!("Health check hit count: {}", HEALTH_CHECK_HITS.load(Ordering::SeqCst));
    // Increment the global health check hit counter
    let hits: usize = HEALTH_CHECK_HITS.fetch_add(1, Ordering::SeqCst);
    println!("Health check hit count after increment: {}", hits);

    // Scenario 1: Service is paused
    if PAUSED.load(Ordering::SeqCst) {
        tracing::event!(target: "backend", tracing::Level::WARN, "Service is paused, returning 503.");
        return HttpResponse::ServiceUnavailable().body("Service is paused");
    }

    // Scenario 2: Simulate transient errors for the first few requests
    match hits {
        0 => { // First hit
            tracing::event!(target: "backend", tracing::Level::WARN, "First hit: Simulating 503 Service Unavailable.");
            // Simulate a delay to mimic a slow service
            println!("Simulating a delay for the first hit...");
            sleep(Duration::from_secs(1)).await; // Add a small delay to simulate network latency
            HttpResponse::ServiceUnavailable().body("Service is temporarily unavailable (first hit)")
        }
        1 => { // Second hit
            tracing::event!(target: "backend", tracing::Level::WARN, "Second hit: Simulating 500 Internal Server Error.");
            // Simulate a delay to mimic a slow service
            println!("Simulating a delay for the second hit...");
            sleep(Duration::from_secs(1)).await; // Add a small delay
            HttpResponse::InternalServerError().body("Internal Server Error (second hit)")
        }
        _ => { // Subsequent hits
            // Scenario 3: Simulate a long-running but successful health check
            tracing::event!(target: "backend", tracing::Level::INFO, "Simulating long-running health check. Sleeping for 5 secs.");
            println!("Simulating a long-running health check for subsequent hits...");
            // Sleep for 5 seconds to simulate a long-running health check
            sleep(Duration::from_secs(3)).await; // This will trigger the Python timeout if less than 5s
            HttpResponse::Ok().body("Service is running (after retries)")
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/reset-health-hits",
    responses(
        (status = 200, description="Health check hits counter reset successfully"),
    )
)]
#[post("/reset-health-hits")]
pub async fn reset_health_check_hits() -> impl Responder {
    println!("Resetting health check hits counter...");
    // Reset the health check hits counter
    HEALTH_CHECK_HITS.store(0, Ordering::SeqCst);
    tracing::event!(target: "backend", tracing::Level::INFO, "Health check hits counter reset.");
    HttpResponse::Ok().body("Health check hits counter reset")
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
    tracing::event!(target: "backend", tracing::Level::INFO, "Accessing application state endpoint.");
    let json: SerializableAppState<'_> = data.to_serializable().await;

    let pkey: &[u8] = data.private_key.as_ref();
    let public_key: &[u8] = data.public_key.as_ref();
    println!("Lets see the app's private key: {}", hex::encode(pkey));
    println!("Lets see the app's public key: {}", hex::encode(public_key));

    // Print out the HEALTH_CHECK_HITS
    println!("Health check hits: {}", HEALTH_CHECK_HITS.load(Ordering::SeqCst));
    // Print out the PAUSED state
    println!("Service paused state: {}", PAUSED.load(Ordering::SeqCst));
    tracing::event!(target: "backend", tracing::Level::INFO, "Returning application state.");

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

// #[utoipa::path(
//   get,
//   path = "/api/ws/echo",
//   responses(
//     (status = 101, description = "WebSocket connection established"),
//     (status = 400, description = "Bad Request"),
//     (status = 500, description = "Internal Server Error")
//   )
// )]
// #[get("/ws/echo")]
// async fn echo(req: HttpRequest, stream: Payload) -> Result<HttpResponse, Error> {
//     let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

//     let mut stream = stream
//         .aggregate_continuations()
//         // aggregate continuation frames up to 1MiB
//         .max_continuation_size(2_usize.pow(20));

//     // start task but don't wait for it
//     actix_web::rt::spawn(async move {
//         // receive messsges from websocket
//         while let Some(msg) = stream.next().await {
//             match msg {
//                 Ok(AggregatedMessage::Text(text)) => {
//                     // Echo text message
//                     session.text(text).await.unwrap();
//                 }
//                 Ok(AggregatedMessage::Binary(bin)) => {
//                     // Echo binary message
//                     session.binary(bin).await.unwrap();
//                 }
//                 Ok(AggregatedMessage::Ping(msg)) => {
//                     // Respond to ping with pong
//                     session.pong(&msg).await.unwrap();
//                 }

//                 _ => {
//                     // Handle other message types or errors
//                     tracing::warn!("Received unsupported message type or error: {:?}", msg);
//                 }
//             }
//         }
//     });

//     // respond immediately with response connected to WS session
//     Ok(res)
// }

#[utoipa::path(
	post,
	path = "/api/counter",
  request_body = serde_json::Value,
	responses(
		(status = 200, description="Returns the updated counter value"),
	),
	tag = "counter"
)]
#[post("/test_value")]
pub async fn test_value(body: Json<serde_json::Value>) -> impl Responder {
    // Simulate some processing
    sleep(Duration::from_secs(5)).await;

    // Return a simple response
    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&body).unwrap())
}