use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(RustEmbed)]
#[folder = "client/build"]
pub struct StaticFiles;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct AppState {
    pub app_name: String,
    pub app_version: String,
    pub counter: Mutex<i32>,
}

// Serializable version of the struct
#[derive(Serialize)]
struct SerializableAppState<'a> {
    app_name: &'a str,
    app_version: &'a str,
    counter: i32,
}
impl AppState {
  pub fn new(app_name: &str) -> SharedState {
      Arc::new(AppState {
          app_name: app_name.to_string(),
          app_version: env!("CARGO_PKG_VERSION").to_string(),
          counter: Mutex::new(0),
      })
  }
  pub fn _to_pretty_json(&self) -> Result<Vec<u8>, serde_json::Error> {
      let serializable = SerializableAppState {
          app_name: &self.app_name,
          app_version: &self.app_version,
          counter: *self.counter.lock().unwrap(),
      };

      serde_json::to_vec_pretty(&serializable)
  }
}

pub type SharedState = Arc<AppState>;
