use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::Config;
use crate::error::AppError;
use crate::infra::database::models::user_jti::NewUserJti;
use crate::infra::database::repositories::user_jti::UserJtiRepository;
use crate::infra::jwt::error::JwtError;

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
        user_jti: &impl UserJtiRepository,
    ) -> Result<String, AppError> {
        let refresh_token = encode(&Header::default(), &claims, &self.refresh_encoding_key)
            .map_err(JwtError::from)?;

        user_jti
            .insert(NewUserJti::from_timestamp(
                claims.jti, claims.sub, claims.exp,
            ))
            .await?;

        Ok(refresh_token)
    }

    async fn rotate_refresh_token(
        &self,
        claims: RefreshClaims,
        revoked_jti: Uuid,
        user_jti: &impl UserJtiRepository,
    ) -> Result<String, AppError> {
        let refresh_token = encode(&Header::default(), &claims, &self.refresh_encoding_key)
            .map_err(|_| JwtError::RefreshTokenFailed)?;

        user_jti
            .update(
                revoked_jti,
                NewUserJti::from_timestamp(claims.jti, claims.sub, claims.exp),
            )
            .await?;

        Ok(refresh_token)
    }

    pub async fn create_token_pair(
        &self,
        subject: &str,
        user_jti: &impl UserJtiRepository,
    ) -> Result<(String, String), AppError> {
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
            jti,
        };

        let refresh_token = self
            .generate_refresh_token(refresh_claims, user_jti)
            .await?;

        Ok((access_token, refresh_token))
    }

    pub async fn regenerate_tokens(
        &self,
        refresh_token: &str,
        user_jti: &impl UserJtiRepository,
    ) -> Result<(String, String), AppError> {
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
            .rotate_refresh_token(refresh_claims, old_claims.jti, user_jti)
            .await?;

        Ok((access_token, refresh_token))
    }

    pub fn verify(&self, access_token: &str) -> Result<AccessClaims, AppError> {
        let token = decode::<AccessClaims>(
            &access_token,
            &self.access_decoding_key,
            &Validation::default(),
        )
        .map_err(|_| JwtError::InvalidAccessError)?;

        Ok(token.claims)
    }
}
