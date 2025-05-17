use actix_web::{post, web::{Data, Json}, HttpResponse, Responder};

use actix_svelte::{CreateUser, LoginUser, User};
use argon2::{
		password_hash::{
				rand_core::OsRng,
				PasswordHash, PasswordHasher, PasswordVerifier, SaltString
		},
		Argon2
};

use crate::server::{DatabaseState, SharedState};

#[utoipa::path(
	post,
	path = "/api/auth/register",
	request_body = CreateUser,
	responses(
		(status = 200, description="Returns the registered user state"),
		(status = 500, description="Failed to register user")
	),
	tag = "auth",
)]
#[post("register")]
pub async fn register_user(data: Data<SharedState>, db_pool: Data<DatabaseState>, user: Json<CreateUser>) -> impl Responder {
		tracing::event!(target: "backend", tracing::Level::INFO, "Accessing register endpoint.");
		// Generate a random salt for hashing the password using today's datetime, username, and a random number
		// let salt = format!("{}-{}-{}", chrono::Utc::now().timestamp(), user.username, rand::random::<u32>()).as_bytes();
		let salt: SaltString = SaltString::generate(&mut OsRng);
		// Argon2 with default params (Argon2id v19)
		let argon2: Argon2<'_> = Argon2::default();

		// Hash the password using Argon2
		let password_hash: String = argon2.hash_password(user.password.as_bytes(), &salt).expect("Failed to hash password").to_string();
		// Parse the hashed password into a PasswordHash object
		let parsed_hash: PasswordHash<'_> = PasswordHash::new(&password_hash).expect("Failed to parse password hash");
		let result: Result<(), argon2::password_hash::Error> = argon2.verify_password(user.password.as_bytes(), &parsed_hash);
		println!("Password verification result: {:?}", result);
		// Lets insert it into our user table
		let query_result = sqlx::query!(
			"INSERT INTO users (username, password_hash) VALUES (?, ?)",
			user.username,
			password_hash
		)
		.execute(&db_pool.pool)
		.await;

		match query_result {
				Ok(_) => {
						tracing::event!(target: "backend", tracing::Level::INFO, "User registered successfully.");
						HttpResponse::Ok()
								.content_type("application/json")
								.body(r#"{"message": "User registered successfully"}"#)
				},
				Err(e) => {
						tracing::event!(target: "backend", tracing::Level::ERROR, "Failed to register user: {}", e);
						HttpResponse::InternalServerError()
								.content_type("application/json")
								.body(r#"{"error": "Failed to register user"}"#)
				}
		}
}

#[utoipa::path(
	post,
	path = "/api/auth/login",
	request_body = LoginUser,
	responses(
		(status = 200, description="Logins the user successfully"),
		(status = 500, description="Failed to login the user")
	),
	tag = "auth",
)]
#[post("login")]
pub async fn login(data: Data<SharedState>, db_pool: Data<DatabaseState>, user: Json<LoginUser>) -> impl Responder {
		tracing::event!(target: "backend", tracing::Level::INFO, "Accessing login endpoint.");
		// Fetch the user from the database
		let query_result = sqlx::query_as!(User, "SELECT * FROM users WHERE username = ?", user.username)
				.fetch_one(&db_pool.pool)
				.await;


		match query_result {
				Ok(fetched_user) => {
						let entered_password: &String = &user.password; // Assuming your login input has a password field
						let stored_password_hash: &String = &fetched_user.password_hash;

						// Parse the stored password hash string
						match PasswordHash::new(stored_password_hash) {
								Ok(parsed_hash) => {
										// Verify the entered password against the parsed hash
										let verification_result: Result<(), argon2::password_hash::Error> = Argon2::default().verify_password(entered_password.as_bytes(), &parsed_hash);

										match verification_result {
												Ok(_) => {
														// Password is correct, return success response
														HttpResponse::Ok()
																.content_type("application/json")
																.json(fetched_user)
												},
												Err(_) => {
														// Password is incorrect
														HttpResponse::Unauthorized()
																.content_type("application/json")
																.body(r#"{"error": "Invalid username or password"}"#)
												}
										}
								},
								Err(_) => {
										// Password is incorrect
										HttpResponse::Unauthorized()
												.content_type("application/json")
												.body(r#"{"error": "Invalid username or password"}"#)
								}
						}
				},
				Err(e) => {
						// User not found or other error
						tracing::event!(target: "backend", tracing::Level::ERROR, "Failed to login user: {}", e);
						HttpResponse::InternalServerError()
								.content_type("application/json")
								.body(r#"{"error": "Failed to login user"}"#)
				}
		}
}