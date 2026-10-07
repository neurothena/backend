use uuid::Uuid;

use crate::infra::database::{error::DatabaseError, models::user_jti::{NewUserJti}};

#[async_trait::async_trait]
pub trait UserJtiRepository {
    async fn insert(&self, jti: NewUserJti) -> Result<(), DatabaseError>;
    async fn revoke(&self, jti: Uuid) -> Result<(), DatabaseError>;
    async fn update(&self, revoked: Uuid, jti: NewUserJti) -> Result<(), DatabaseError>;
}