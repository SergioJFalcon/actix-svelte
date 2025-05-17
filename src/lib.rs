use chrono::NaiveDateTime;
// use rusty_paseto::v4::{
//     decode, encode, public::Ed25519KeyPair, public::PublicKey, public::SecretKey, Claims,
//     DefaultFooter, Error as PasetoError, Token,
// };
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

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
    pub password: String
}

// --- Authentication Data ---

#[derive(Debug, Serialize, Deserialize)]
struct AuthClaims {
    user_id: i64,
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
