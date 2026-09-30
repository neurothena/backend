use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use strum::Display;
use thiserror::Error;

use crate::config::Config;

#[derive(Debug, Error)]
pub(super) enum JwtError {
    #[error("Internal token error")]
    CryptoError(#[from] jsonwebtoken::errors::Error),

    #[error("Invalid refresh token")]
    InvalidRefreshError,

    #[error("Invalid token type `{token_type}`, expected `{expected}`")]
    InvalidTokenType {
        token_type: TokenType,
        expected: TokenType,
    },
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

#[derive(Debug, Serialize, Deserialize, Display, PartialEq, Eq)]
enum TokenType {
    Refresh,
    Access,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub token_type: TokenType,
}

impl JwtService {
    pub fn new(config: &Config) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(config.jwt_secret.clone().as_bytes()),
            decoding_key: DecodingKey::from_secret(config.jwt_secret.clone().as_bytes()),
        }
    }

    pub fn create_token(&self, subject: &str) -> Result<(String, String), JwtError> {
        let now = Utc::now();

        let access_claims = Claims {
            sub: subject.to_string(),
            exp: (now + Duration::minutes(15)).timestamp(),
            iat: now.timestamp(),
            token_type: TokenType::Access,
        };

        let access_token = encode(&Header::default(), &access_claims, &self.encoding_key)?;

        let refresh_claims = Claims {
            sub: subject.to_string(),
            exp: (now + Duration::days(7)).timestamp(),
            iat: now.timestamp(),
            token_type: TokenType::Refresh,
        };

        let refresh_token = encode(&Header::default(), &refresh_claims, &self.encoding_key)?;

        Ok((access_token, refresh_token))
    }

    pub fn regenerate_access_token(&self, refresh_token: &str) -> Result<String, JwtError> {
        let token_data = decode(refresh_token, &self.decoding_key, &Validation::default())
            .map_err(|_| JwtError::InvalidRefreshError)?;

        let claims: Claims = token_data.claims;

        if (claims.token_type != TokenType::Refresh) {
            return Err(JwtError::InvalidTokenType {
                token_type: claims.token_type,
                expected: TokenType::Refresh,
            });
        }

        let now = Utc::now();

        let access_claims = Claims {
            sub: claims.sub,
            exp: (now + Duration::minutes(15)).timestamp(),
            iat: now.timestamp(),
            token_type: TokenType::Access,
        };

        let access_token = encode(&Header::default(), &access_claims, &self.encoding_key)?;

        Ok(access_token)
    }
}
