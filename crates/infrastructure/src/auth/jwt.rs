use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: i64,
    pub iat: i64,
    pub role: String,
}

pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    expiry_hours: i64,
}

impl JwtManager {
    pub fn new(secret: &str, expiry_hours: i64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            expiry_hours,
        }
    }

    pub fn generate_token(&self, user_id: Uuid, role: &str) -> Result<String, String> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id,
            exp: (now + Duration::hours(self.expiry_hours)).timestamp(),
            iat: now.timestamp(),
            role: role.to_string(),
        };
        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| format!("JWT encode error: {e}"))
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, String> {
        let data = decode::<Claims>(token, &self.decoding_key, &Validation::default())
            .map_err(|e| format!("JWT decode error: {e}"))?;
        Ok(data.claims)
    }
}
