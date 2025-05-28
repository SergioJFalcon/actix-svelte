use actix_web::web::{self, ServiceConfig};

use super::handlers;

/// #### Base App Services
/// These services are used for the base application
pub fn app_services(cfg: &mut ServiceConfig) {
  cfg.service(handlers::get_app_state);
  cfg.service(handlers::health_check);
  cfg.service(handlers::counter);
  cfg.service(handlers::pause_service);
  cfg.service(handlers::unpause_service);
}

/// #### Authentication Services
/// These services are used for user authentication
pub fn auth_services(cfg: &mut ServiceConfig) {
  cfg.service(
    web::scope("/auth")
      .service(handlers::auth::register_user)
      .service(handlers::auth::login)
      .service(handlers::auth::protected)
      // .service(handlers::login_user)
      // .service(handlers::logout_user),
  );
}