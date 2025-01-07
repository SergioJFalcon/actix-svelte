mod server;

fn main() {
    println!("Hello, world!");
    server::actix_server_app().unwrap();
}