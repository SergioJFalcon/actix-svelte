use actix_web::web::{self, ServiceConfig};

use super::handlers;

/// #### Base App Services
/// These services are used for the base application
pub fn app_services(cfg: &mut ServiceConfig) {
  cfg.service(handlers::get_app_state);
  cfg.service(handlers::health_check);
  cfg.service(handlers::counter);
}
