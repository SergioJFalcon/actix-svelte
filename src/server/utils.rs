use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::{
  cell::Cell, 
  sync::{
    atomic::{AtomicI32, AtomicUsize, Ordering}, 
    Arc
  }
};
use tokio::sync::RwLock;

#[derive(RustEmbed)]
#[folder = "client/build"]
pub struct StaticFiles;

#[derive(Debug)]
pub struct AppState {
    pub app_name: String,
    pub app_version: String,
    pub counter: RwLock<i32>,
    pub global_count: RwLock<AtomicUsize>,
}

// Serializable version of the struct
#[derive(Serialize)]
pub struct SerializableAppState<'a> {
    app_name: &'a str,
    app_version: &'a str,
    counter: i32,
    global_counter: usize,
}
impl AppState {
  pub fn new(app_name: &str) -> SharedState {
      Arc::new(AppState {
          app_name: app_name.to_string(),
          app_version: env!("CARGO_PKG_VERSION").to_string(),
          // counter: Arc::new(AtomicUsize::new(0)),
          counter: RwLock::new(0),
          global_count: RwLock::new(AtomicUsize::new(0)),
      })
  }
  
  pub async fn to_serializable(&self) -> SerializableAppState {
    SerializableAppState {
        app_name: &self.app_name,
        app_version: &self.app_version,
        counter: *self.counter.read().await,
        global_counter: self.global_count.read().await.load(Ordering::SeqCst),
    }
}
}

pub type SharedState = Arc<AppState>;
