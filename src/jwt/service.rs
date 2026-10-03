use std::sync::Arc;

use chrono::{DateTime, NaiveDateTime};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::config::{self, Config};
use crate::schema::user_jti;
use crate::state::AppState;

#[derive(Debug, Error)]
pub enum JwtError {
    #[error("Internal token error")]
    CryptoError(#[from] jsonwebtoken::errors::Error),

    #[error("Invalid refresh token")]
    InvalidRefreshError,

    #[error("Invalid access token")]
    InvalidAccessError,

    #[error("Database error")]
    DBError,

    #[error("Unable to rotate refresh token")]
    RefreshRotationFailed,

    #[error("Failed to encode an access token")]
    AccessTokenFailed,

    #[error("Failed to encode a refresh token")]
    RefreshTokenFailed,
}

#[derive(Clone)]
pub struct JwtService {
    access_encoding_key: EncodingKey,
    access_decoding_key: DecodingKey,

    refresh_encoding_key: EncodingKey,
    refresh_decoding_key: DecodingKey,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessClaims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub jti: Uuid,
}

impl JwtService {
    pub fn new(config: &Config) -> Self {
        Self {
            access_encoding_key: EncodingKey::from_secret(
                config.jwt_access_secret.clone().as_bytes(),
            ),
            access_decoding_key: DecodingKey::from_secret(
                config.jwt_access_secret.clone().as_bytes(),
            ),
            refresh_encoding_key: EncodingKey::from_secret(
                config.jwt_refresh_secret.clone().as_bytes(),
            ),
            refresh_decoding_key: DecodingKey::from_secret(
                config.jwt_refresh_secret.clone().as_bytes(),
            ),
        }
    }

    async fn generate_access_token(&self, claims: AccessClaims) -> Result<String, JwtError> {
        let access_token = encode(&Header::default(), &claims, &self.access_encoding_key)
            .map_err(|_| JwtError::AccessTokenFailed)?;

        Ok(access_token)
    }

    async fn generate_refresh_token(
        &self,
        claims: RefreshClaims,
        state: &AppState,
    ) -> Result<String, JwtError> {
        let refresh_token = encode(&Header::default(), &claims, &self.refresh_encoding_key)
            .map_err(|_| JwtError::RefreshTokenFailed)?;

        let mut conn = match state.db_pool.get().await {
            Ok(connecton) => connecton,
            Err(_) => return Err(JwtError::DBError),
        };

        let expires_at_naive: NaiveDateTime = DateTime::from_timestamp(claims.exp, 0)
            .map(|dt| dt.naive_utc())
            .unwrap_or_else(|| chrono::Utc::now().naive_utc());

        match diesel::insert_into(user_jti::table)
            .values((
                user_jti::user_email.eq(&claims.sub),
                user_jti::jti.eq(&claims.jti),
                user_jti::expires_at.eq(&expires_at_naive),
            ))
            .execute(&mut conn)
            .await
        {
            Ok(_) => return Ok(refresh_token),
            Err(_) => return Err(JwtError::DBError),
        };
    }

    async fn rotate_refresh_token(
        &self,
        claims: RefreshClaims,
        revoked_jti: Uuid,
        state: &AppState,
    ) -> Result<String, JwtError> {
        let refresh_token = encode(&Header::default(), &claims, &self.refresh_encoding_key)
            .map_err(|_| JwtError::RefreshTokenFailed)?;

        let mut conn = match state.db_pool.get().await {
            Ok(connecton) => connecton,
            Err(_) => return Err(JwtError::DBError),
        };

        let expires_at_naive: NaiveDateTime = DateTime::from_timestamp(claims.exp, 0)
            .map(|dt| dt.naive_utc())
            .unwrap_or_else(|| chrono::Utc::now().naive_utc());

        let delete_result = diesel::delete(
            user_jti::table.filter(
                user_jti::jti
                    .eq(&revoked_jti)
                    .and(user_jti::user_email.eq(&claims.sub)),
            ),
        )
        .execute(&mut conn)
        .await;

        if let Err(_) = delete_result {
            return Err(JwtError::DBError);
        }

        match diesel::insert_into(user_jti::table)
            .values((
                user_jti::jti.eq(&claims.jti),
                user_jti::user_email.eq(&claims.sub),
                user_jti::expires_at.eq(&expires_at_naive),
            ))
            .execute(&mut conn)
            .await
        {
            Ok(_) => return Ok(refresh_token),
            Err(_) => return Err(JwtError::DBError),
        };
    }

    pub async fn create_token_pair(
        &self,
        subject: &str,
        state: &AppState,
    ) -> Result<(String, String), JwtError> {
        let now = Utc::now();

        let access_claims = AccessClaims {
            sub: subject.to_string(),
            exp: (now + Duration::minutes(15)).timestamp(),
            iat: now.timestamp(),
        };

        let access_token = self.generate_access_token(access_claims).await?;

        let jti = Uuid::new_v4();

        let refresh_claims = RefreshClaims {
            sub: subject.to_string(),
            exp: (now + Duration::days(30)).timestamp(),
            iat: now.timestamp(),
            jti: jti,
        };

        let refresh_token = self.generate_refresh_token(refresh_claims, state).await?;

        Ok((access_token, refresh_token))
    }

    pub async fn regenerate_tokens(
        &self,
        refresh_token: &str,
        state: &AppState,
    ) -> Result<(String, String), JwtError> {
        let token_data = decode(
            refresh_token,
            &self.refresh_decoding_key,
            &Validation::default(),
        )
        .map_err(|_| JwtError::InvalidRefreshError)?;

        let old_claims: RefreshClaims = token_data.claims;

        let now = Utc::now();

        let access_claims = AccessClaims {
            sub: old_claims.sub.clone(),
            exp: (now + Duration::minutes(15)).timestamp(),
            iat: now.timestamp(),
        };

        let access_token = self.generate_access_token(access_claims).await?;

        let jti = Uuid::new_v4();

        let refresh_claims = RefreshClaims {
            sub: old_claims.sub,
            exp: (now + Duration::days(30)).timestamp(),
            iat: now.timestamp(),
            jti,
        };

        let refresh_token = self
            .rotate_refresh_token(refresh_claims, old_claims.jti, state)
            .await?;

        Ok((access_token, refresh_token))
    }

    pub fn verify(&self, access_token: &str) -> Result<(), JwtError> {
        match decode::<AccessClaims>(&access_token, &self.access_decoding_key, &Validation::default()) {
            Ok(_) => Ok(()),
            Err(_) => Err(JwtError::InvalidAccessError)
        }
    }

    pub fn get_access_claims(&self, access_token: &str) -> Result<AccessClaims, JwtError> {
        match decode::<AccessClaims>(&access_token, &self.access_decoding_key, &Validation::default()) {
            Ok(token) => Ok(token.claims),
            Err(_) => Err(JwtError::InvalidAccessError)
        }
    }
}
