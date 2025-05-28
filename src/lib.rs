use std::sync::atomic::{AtomicBool, AtomicUsize};

use actix_web::{web, HttpRequest};
use chrono::NaiveDateTime;
// use rusty_paseto::v4::{
//     decode, encode, public::Ed25519KeyPair, public::PublicKey, public::SecretKey, Claims,
//     DefaultFooter, Error as PasetoError, Token,
// };
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

// Global flag for shutdown coordination
pub static HEALTH_CHECK_HITS: AtomicUsize = AtomicUsize::new(0);
pub static PAUSED: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Debug, Deserialize, FromRow, Serialize, ToSchema)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUser {
    pub username: String,
    pub password: String
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginUser {
    pub username: String,
    pub password: String,
    // pub token: Option<String>, // Optional token for authentication
}

// --- Authentication Data ---
// Define a struct for the claims you want to include in the token
#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i64, // Assuming your User model has an ID
    pub username: String,
    // You can add other relevant claims here, like roles, etc.
    pub exp: i64, // Expiration timestamp
}



// --- Token Generation and Decoding ---
// pub fn generate_token(user: &User, config: &AppConfig) -> Result<String, PasetoError> {
//     let now: DateTime<Utc> = Utc::now();
//     let expiration = now + Duration::from_secs(3600); // 1 hour

//     let claims = Claims::builder()
//         .issuer(&config.issuer)
//         .audience(&config.audience)
//         .subject(&user.username)
//         .issued_at(now.timestamp())
//         .expiration(expiration.timestamp())
//         .add_claim("user_id", user.id.into())
//         .build()
//         .expect("Failed to build claims");

//     let token = Token::<Claims<AuthClaims>, DefaultFooter>::new(claims);
//     let secret_key = config.paseto_secret.as_bytes();
//     encode(token, &secret_key)
// }

// pub fn decode_token(token_str: &str, secret: &str) -> Result<Token<Claims<serde_json::Value>, DefaultFooter>, PasetoError> {
//     let secret_key = secret.as_bytes();
//     decode(token_str, &secret_key, None)
// }
