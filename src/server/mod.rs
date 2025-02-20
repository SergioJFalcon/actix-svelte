mod handlers;
mod utils;

use actix_web::{
  App,
  dev::Server,
  HttpServer,
  web::Data,
};
use std::io::Result;
use std::net::TcpListener;

use handlers::{counter, get_app_state, health_check, serve_static_files};
use utils::AppState;
use crate::settings::Settings;

pub struct Application {
  host: String,
  port: u16,
  server: Server,
}
impl Application {
  pub async fn build(
      settings: Settings,
  ) -> Result<Self> {
      let address: String = format!(
          "{}:{}",
          settings.application.host, settings.application.port
      );

      let listener: TcpListener = std::net::TcpListener::bind(&address)?;
      let local_addr: std::net::SocketAddr = listener.local_addr().unwrap();
      let host: String = if local_addr.ip().to_string() == "::1" { format!("localhost") } else { local_addr.ip().to_string() };
      let port: u16 = listener.local_addr().unwrap().port();

      println!("###SERVER BUILD: {:?}", settings);

      let server: Server = actix_server_app(listener).await?;

      Ok(Self { host, port, server })
  }

  pub fn host(&self) -> &str {
      &self.host
  }

  pub fn port(&self) -> u16 {
      self.port
  }

  pub async fn run_until_stopped(self) -> Result<()> {
      println!("\t🚀 Server started successfully");
      println!("\t🌍 Listening on: http://{}:{}/", self.host, self.port);
      println!("\t🛑 Press <Ctrl-C> to stop");

      self.server.await
  }
}


pub async fn actix_server_app(listener: TcpListener) -> Result<Server> {
    let shared_state: AppState = AppState::new("Actix Svelte Template Server");

    let server_app: Server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(shared_state.clone()))
            .service(health_check)
            .service(get_app_state)
            .service(counter)
            .service(serve_static_files)
            // Logging middleware
            .wrap(actix_web::middleware::Logger::default())
    })
    .listen(listener).expect("Failed to listen on address")
    .shutdown_timeout(5) // Give 5 seconds for graceful shutdown
    .workers(1) 
    .run();

    Ok(server_app)
}
