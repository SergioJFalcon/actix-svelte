mod server;

use std::net::TcpListener;
use server::build_server_app;
use tokio::runtime::Runtime;
use tokio_util::sync::CancellationToken;
use std::sync::mpsc;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use windows_service::{
  define_windows_service,
  service::{
      ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType,
  },
  service_control_handler::{self, ServiceControlHandlerResult},
  service_dispatcher,
};

// Service name
const SERVICE_NAME: &str = "actix_example";
// Global flag for shutdown coordination
pub static PAUSED: AtomicBool = AtomicBool::new(false);

// Main service logic
fn run_service_server(rx: mpsc::Receiver<ServiceControl>) -> Result<(), Box<dyn std::error::Error>> {
    // Create a new tokio runtime
    let rt: Runtime = Runtime::new()?;
    
    // Create a shutdown flag
    let running: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    let running_clone: Arc<AtomicBool> = running.clone();

    // Spawn the Actix web server
    rt.block_on(async move {
        let hostname: &str = env!("WINDOW_SERVICE_HOST");
        let port: i32 = env!("WINDOW_SERVICE_PORT").parse().expect("WINDOW_SERVICE_PORT must be a number");
        let listener: TcpListener = TcpListener::bind(format!("{}:{}", hostname, port)).expect("Failed to bind address");
        let cancel_token: CancellationToken = CancellationToken::new();
        let server_app: actix_web::dev::Server = build_server_app(listener, cancel_token.clone()).await?;

        let srv: actix_web::dev::ServerHandle = server_app.handle();

        // Spawn control message handler
        tokio::spawn(async move {
          while let Ok(control) = rx.recv() {
              match control {
                  ServiceControl::Stop => {
                      running_clone.store(false, Ordering::SeqCst);
                      srv.stop(true).await;
                      break;
                  }
                  ServiceControl::Pause => {
                      PAUSED.store(true, Ordering::SeqCst);
                      // You could also pause accepting new connections here
                      srv.pause().await;
                  }
                  ServiceControl::Continue => {
                      PAUSED.store(false, Ordering::SeqCst);
                      // You could resume accepting connections here
                      srv.resume().await;
                  }
                  _ => {}
              }
          }
        });

        server_app.await?;
        Ok::<(), std::io::Error>(())
    })?;

    Ok(())
}

// Windows service implementation
define_windows_service!(ffi_service_main, service_main);

fn service_main(_arguments: Vec<std::ffi::OsString>) {
    // Create a channel to coordinate shutdown
    let (control_tx, control_rx) = mpsc::channel();

    // Define the service control handler
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        let _ = control_tx.send(control_event);
        match control_event {
            ServiceControl::Stop => {
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Pause => {
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Continue => {
                ServiceControlHandlerResult::NoError
            }
            _ => ServiceControlHandlerResult::NoError,
        }
    };

    // Register the service control handler
    let status_handle = service_control_handler::register(
        SERVICE_NAME,
        event_handler
    ).unwrap();

    // Tell the system that the service is running
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    }).unwrap();

    // Run the server
    if let Err(e) = run_service_server(control_rx) {
        // Log the error
        eprintln!("Service error: {}", e);
    }

    // Tell the system that the service is stopped
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    }).unwrap();
}

#[cfg(all(windows, not(debug_assertions)))]
fn main() -> Result<(), windows_service::Error> {
    // Start the service dispatcher
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;

    Ok(())
}

#[cfg(debug_assertions)]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
  let hostname: String = dotenvy::var("WINDOW_SERVICE_HOST").expect("HOST must be set");
  let port: u16 = dotenvy::var("WINDOW_SERVICE_PORT")
    .unwrap_or_else(|_| "5000".to_string()) // Default to 5000 if nothing is set
    .parse::<u16>().expect("WINDOW_SERVICE_PORT must be a number");
  let listener: TcpListener = TcpListener::bind(format!("{hostname}:{port}")).expect("Failed to bind to address");
  let cancel_token: CancellationToken = CancellationToken::new();
  let server_app: server::Application = server::Application::build(hostname, port, None).await?;

  server_app.run_until_stopped().await
}