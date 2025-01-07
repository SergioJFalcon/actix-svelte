mod server;

use std::ffi::OsString;
use std::time::Duration;
use windows_service::{
  define_windows_service,
  Result,
  service_dispatcher,
  service_control_handler::{self, ServiceControlHandlerResult},
  service::{
      ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType,
  },
};
use actix_rt::signal;

const SERVICE_NAME: &str = "actix_svelte_service";

// Generate the windows service boilerplate.
// The boilerplate contains the low-level service entry function (ffi_service_main) that parses
// incoming service arguments into Vec<OsString> and passes them to user defined service
// entry (my_service_main).
#[cfg(windows)]
define_windows_service!(ffi_service_main, my_service_main);

#[cfg(windows)]
pub fn my_service_main(_arguments: Vec<OsString>) {
}
// Service entry function which is called on background thread by the system with service
// parameters. There is no stdout or stderr at this point so make sure to configure the log
// output to file if needed.

#[cfg(windows)]
pub async fn run_service() -> Result<()>{
    // Create a channel to be able to poll a stop event from the service worker loop.
    let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel();

    let server_app: actix_web::dev::Server = server::actix_server_app().await;
    let server_handle: actix_web::dev::ServerHandle = server::actix_server_handle(&server_app).await;
    let srv: actix_web::dev::ServerHandle = server_handle.clone();
    
    // Set up the service control handler
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            // Notifies a service to report its current status information to the service
            // control manager. Always return NoError even if not implemented.
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,

            ServiceControl::Stop | ServiceControl::Shutdown => {
              let shutdown_tx = shutdown_tx.clone();
              let srv = srv.clone();
              actix_rt::spawn(async move {
                  shutdown_tx.send(()).unwrap();
                  // Handle the shutdown signal with await
                  let _ = signal::ctrl_c().await;
                  let _ = srv.stop(true).await;
              });
              ServiceControlHandlerResult::NoError
          },

            ServiceControl::UserEvent(code) => {
              if code.to_raw() == 130 {
                let shutdown_tx = shutdown_tx.clone();
                let srv = srv.clone();
                actix_rt::spawn(async move {
                    shutdown_tx.send(()).unwrap();
                    // Handle the shutdown signal with await
                    let _ = signal::ctrl_c().await;
                    let _ = srv.stop(true).await;
                });
              }
              ServiceControlHandlerResult::NoError
            },

            _ => ServiceControlHandlerResult::NoError,
        }
    };

    let status_handle: service_control_handler::ServiceStatusHandle = service_control_handler::register(SERVICE_NAME, event_handler)
        .expect("Failed to register service control handler");

    // Tell the system that the service is running
    status_handle
        .set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })
        .expect("Failed to set service status");

      server::start_actix_server(server_app).await.expect("Failed to start server");

    // Tell the system that service has stopped.
    status_handle.set_service_status(ServiceStatus {
      service_type: ServiceType::OWN_PROCESS,
      current_state: ServiceState::Stopped,
      controls_accepted: ServiceControlAccept::empty(),
      exit_code: ServiceExitCode::Win32(0),
      checkpoint: 0,
      wait_hint: Duration::default(),
      process_id: None,
    }).expect("Failed to set service status");

    Ok(())
}

#[cfg(windows)]
pub fn start_service() -> Result<()> {
  // Register generated `ffi_service_main` with the system and start the service, blocking
  // this thread until the service is stopped.
  service_dispatcher::start(SERVICE_NAME, ffi_service_main)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Hello, world!");
    #[cfg(not(debug_assertions))]
    {
        // Running in production/release mode as a windows service
        // Start as Windows service
        // service_dispatcher::start("MyActixService", ffi_service_main)
        //     .expect("Failed to start service dispatcher");
        start_service().expect("Failed to start service dispatcher");

        return Ok(());
    }

    #[cfg(debug_assertions)]{
        // Running in development mode
        // server::actix_server_app().await
        
        let server_app: actix_web::dev::Server = server::actix_server_app().await;
        let server_handle: actix_web::dev::ServerHandle = server::actix_server_handle(&server_app).await;
        
        server_app.await
    }

}