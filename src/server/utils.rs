use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::{
  cell::Cell, 
  sync::{
    atomic::{AtomicI32, AtomicUsize, Ordering}, 
    Arc
  }
};

#[derive(RustEmbed)]
#[folder = "client/build"]
pub struct StaticFiles;

#[derive(Clone, Debug, Default)]
pub struct AppState {
    pub app_name: String,
    pub app_version: String,
    pub counter: Arc<AtomicUsize>,
    pub local_count: Cell<usize>,
    pub global_count: Arc<AtomicUsize>,
}

// Serializable version of the struct
#[derive(Serialize)]
struct SerializableAppState<'a> {
    app_name: &'a str,
    app_version: &'a str,
    counter: usize,
    local_counter: usize,
    global_counter: usize,
}
impl AppState {
  pub fn new(app_name: &str) -> AppState {
      AppState {
          app_name: app_name.to_string(),
          app_version: env!("CARGO_PKG_VERSION").to_string(),
          counter: Arc::new(AtomicUsize::new(0)),
          local_count: Cell::new(0),
          global_count: Arc::new(AtomicUsize::new(0)),
      }
  }
  pub fn to_pretty_json(&self) -> Result<Vec<u8>, serde_json::Error> {
      let serializable = SerializableAppState {
          app_name: &self.app_name,
          app_version: &self.app_version,
          counter: self.counter.load(Ordering::Relaxed),
          local_counter: self.local_count.get(),
          global_counter: self.global_count.load(Ordering::Relaxed),
      };

      serde_json::to_vec_pretty(&serializable)
  }
}

pub type SharedState = Arc<AppState>;
