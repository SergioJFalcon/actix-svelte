use utoipa::{openapi::security::{ApiKey, ApiKeyValue, SecurityScheme}, Modify, OpenApi};
use super::handlers::{
		__path_counter, __path_get_app_state, __path_health_check,
		auth::{__path_register_user, __path_login, __path_protected}
};

#[derive(OpenApi)]
#[openapi(
		info(
			title="Clean Room Condition Status API",
			version="3.1.0",
			description="This is a simple API for monitoring the condition of a clean room",
		),
		paths(
			get_app_state,
			counter,
			health_check,
			register_user,
			login,
			protected
		),
		components(
			schemas(
			)
		),
		modifiers(&SecurityAddon),
		tags(
			(name="core", description="Operations about core functionality"),
		),
)]
pub struct ApiDocumentation;



struct SecurityAddon;

impl Modify for SecurityAddon {
		fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
				let components: &mut utoipa::openapi::Components = openapi.components.as_mut().unwrap(); // we can unwrap safely since there already is components registered.
				components.add_security_scheme(
						"bearerAuth",
						SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("bearerAuth"))),
				)
		}
}