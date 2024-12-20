use actix_files::{Files, NamedFile};
use actix_web::{
  App,
  HttpServer,
  web::Data,
};
use rust_embed::RustEmbed;
use std::sync::Mutex;
use std::net::TcpListener;


mod handlers;


#[derive(RustEmbed)]
#[folder = "client/build"]
pub struct StaticFiles;

#[derive(Debug)]
pub struct AppState {
  pub app_name: String,
  pub counter: Mutex<i32>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let hostname: &str = "localhost";
    let port: i32 = 8090;
    let listener: TcpListener = TcpListener::bind(format!("{hostname}:{port}")).expect("Failed to bind to address");
    let static_files: String = String::from(
          "C:/Users/sfalcon/code/development/rust/aisin-jc-cleanroom/client/build");
    
    println!("\t🚀 Server started successfully");
    println!("\t🌍 Listening on: http://{}:{}/", hostname, port);

    HttpServer::new(|| {
        App::new()
            .app_data(Data::new(AppState {
                app_name: "Actix-web".to_string(),
                counter: Mutex::new(0),
            }))
            .service(handlers::serve_static_files)
            .service(handlers::counter)
            // .service(
            //   Files::new("/", static_files.clone())
            //       .index_file("index.html")
            //       .default_handler(
            //           NamedFile::open(format!("{}/index.html", static_files.clone()))
            //               .expect("index file should exist"),
            //       ),
            // )
    })
    .listen(listener)?
    .run()
    .await
}