use std::sync::Arc;

use crate::{
    config::Config,
    db::DbPool,
    jwt::service::{JwtError, JwtService},
};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub config: Arc<Config>,
    pub jwt_service: JwtService,
}

impl AppState {
    pub fn new(db_pool: DbPool, config: Arc<Config>) -> Self {
        Self {
            db_pool,
            jwt_service: JwtService::new(&config),
            config,
        }
    }

    pub async fn create_token_pair(&self, subject: &str) -> Result<(String, String), JwtError> {
        self.jwt_service.create_token_pair(subject, self).await
    }

    pub async fn regenerate_tokens(
        &self,
        refresh_token: &str,
    ) -> Result<(String, String), JwtError> {
        self.jwt_service
            .regenerate_tokens(refresh_token, self)
            .await
    }
}
