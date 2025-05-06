use utoipa::OpenApi;
use super::handlers::{
		__path_counter, __path_get_app_state, __path_health_check
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
			health_check
		),
		components(
			schemas(
			)
		),
		tags(
			(name="core", description="Operations about core functionality"),
		),
)]
pub struct ApiDocumentation;
