use actix_web::{cookie::{Cookie, SameSite}, get, post, web::{Data, Json}, HttpResponse, Responder};

use actix_svelte::{CreateUser, LoginUser, User};
use argon2::{
		password_hash::{
				rand_core::OsRng,
				PasswordHash, PasswordHasher, PasswordVerifier, SaltString
		},
		Argon2
};
use chrono::{DateTime, Duration, Utc};
use rusty_paseto::prelude::*;
use rusty_paseto::prelude::PasetoBuilder;

use crate::server::{DatabaseState, SharedState, AuthenticatedUser};


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
						// let private_key_string = data.secret_key_string.clone();
						// let app_private_key = data.private_key.clone();

						// let private_key: Key<64> = Key::<64>::try_from(private_key_string.as_str()).expect("Failed to parse PASETO secret key");
						// let pk: &[u8] = private_key.as_slice();
						let pk_slice = data.private_key.as_ref();
						let private_key = PasetoAsymmetricPrivateKey::<V4, Public>::from(pk_slice);
						// let public_key = Key::<32>::try_from("1eb9dbbbbc047c03fd70604e0071f0987e16b28b757225c11f00415d0e20b1a2").expect("Failed to parse public key");
						let public_key = PasetoAsymmetricPublicKey::<V4, Public>::from(&data.public_key);

						let footer = Footer::from("some footer");
						let expiration: DateTime<Utc> = Utc::now() + Duration::hours(24); // 24 hour expiration

						// Build the PASETO token with claims
						let token: String = PasetoBuilder::<V4, Public>::default()
								.set_claim(AudienceClaim::from("custoemrs"))
								.set_claim(SubjectClaim::from("my local subjects"))
								.set_claim(IssuerClaim::from("me"))
								.set_claim(TokenIdentifierClaim::from("me"))
								.set_claim(IssuedAtClaim::try_from(Utc::now().to_rfc3339()).expect("Failed to set issued at claim"))
								.set_claim(NotBeforeClaim::try_from(Utc::now().to_rfc3339()).expect("Failed to set not before claim"))
								.set_claim(ExpirationClaim::try_from(expiration.to_rfc3339()).expect("Failed to set expiration claim"))
								.set_claim(CustomClaim::try_from(("username", user.username.clone())).expect("Failed to set custom claim"))
								.set_claim(CustomClaim::try_from(("data", "this is a secret message")).expect("Failed to set custom claim"))
								.build(&private_key)
								.expect("Failed to sign PASETO token");
							// .set_footer(Footer::from("Footer example"))
							// .set_implicit_assertion(ImplicitAssertion::from("example-implicit-assertion"))
						
						let cookie = Cookie::build("auth_token", token.clone()) // Choose a name for your cookie
								.http_only(true)
								.same_site(SameSite::Strict) // Recommended for security against CSRF
								.path("/") // Adjust the path as needed
								// .secure(true) // Uncomment if your site is served over HTTPS
								.finish();

						// Then allow the user to login with this token
						tracing::event!(target: "backend", tracing::Level::INFO, "User registered successfully.");
						HttpResponse::Ok()
								.cookie(cookie)
								.content_type("application/json")
								.json(AuthenticatedUser {
										token: token.to_string(),
										expiration: expiration,
								})
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

		// On login, we need to check if the user has a token already in their session, if they do, then use that to login, if not
		// use the username and password to login.
		// Check if the user has a token in their session

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
														// Password is correct, now generate a PASETO re==token and return success response
														tracing::event!(target: "backend", tracing::Level::INFO, "User {} authenticated correctly, will create token now.", fetched_user.username);
														let pk_slice = data.private_key.as_ref();
														let private_key = PasetoAsymmetricPrivateKey::<V4, Public>::from(pk_slice);
														// let public_key: PasetoAsymmetricPublicKey<'_, V4, Public> = PasetoAsymmetricPublicKey::<V4, Public>::from(&data.public_key);
														let expiration: DateTime<Utc> = Utc::now() + Duration::hours(24); // 24 hour expiration
														// Build the PASETO token with claims
														let token: String = PasetoBuilder::<V4, Public>::default()
																.set_claim(AudienceClaim::from("custoemrs"))
																.set_claim(SubjectClaim::from("my local subjects"))
																.set_claim(IssuerClaim::from("me"))
																.set_claim(TokenIdentifierClaim::from("me"))
																.set_claim(IssuedAtClaim::try_from(Utc::now().to_rfc3339()).expect("Failed to set issued at claim"))
																.set_claim(NotBeforeClaim::try_from(Utc::now().to_rfc3339()).expect("Failed to set not before claim"))
																.set_claim(ExpirationClaim::try_from(expiration.to_rfc3339()).expect("Failed to set expiration claim"))
																.set_claim(CustomClaim::try_from(("username", user.username.clone())).expect("Failed to set custom claim"))
																.set_claim(CustomClaim::try_from(("data", "this is a secret message")).expect("Failed to set custom claim"))
																.build(&private_key)
																.expect("Failed to sign PASETO token");
															// .set_footer(Footer::from("Footer example"))
															// .set_implicit_assertion(ImplicitAssertion::from("example-implicit-assertion"))
														
														let cookie = Cookie::build("auth_token", token.clone()) // Choose a name for your cookie
																.http_only(true)
																.same_site(SameSite::Strict) // Recommended for security against CSRF
																.path("/") // Adjust the path as needed
																// .secure(true) // Uncomment if your site is served over HTTPS
																.finish();
														
														// Log the successful login
														tracing::event!(target: "backend", tracing::Level::INFO, "User logged in successfully: {}", fetched_user.username);
														HttpResponse::Ok()
																.cookie(cookie)
																.content_type("application/json")
																.json(AuthenticatedUser {
																		token: token.to_string(),
																		expiration: expiration,
																})
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

		// security(("bearerAuth" = [])),

// Example protected route
// #[utoipa::path(
// 		get,
// 		path = "/api/auth/protected",
// 		responses(
// 				(status = 200, description="Returns data for authenticated users"),
// 				(status = 401, description="Unauthorized")
// 		),
// 		tag = "protected"
// )]
// #[get("protected")]
// pub async fn protected(_data: Data<SharedState>, _db_pool: Data<DatabaseState>, user: Json<LoginUser>) -> impl Responder {
// 		tracing::event!(target: "backend", tracing::Level::INFO, "Accessing protected endpoint by user: {}", user.username);
// 		// TODO: Implement token verification logic here
// 		// let public_key = PasetoAsymmetricPublicKey::<V4, Public>::from(&data.public_key);

// 		HttpResponse::Ok().json(serde_json::json!({ "message": format!("Hello, token: {}!", user.username)}))
// }

#[utoipa::path(
    get,
    path = "/api/auth/protected",
    security(("bearerAuth" = [])), // Documenting the security requirement for Swagger/OpenAPI
    responses(
        (status = 200, description="Returns data for authenticated users", body = String),
        (status = 401, description="Unauthorized")
    ),
    tag = "protected"
)]
#[get("/protected")]
pub async fn protected(user: AuthenticatedUser) -> impl Responder {
    tracing::event!(target: "backend", tracing::Level::INFO, "Accessing protected endpoint by user: {}", user.token);
		println!("Authenticated user token: {:?}", user);
    HttpResponse::Ok().json(serde_json::json!({ "message": format!("Hello, token: {}!", user.token) }))
}