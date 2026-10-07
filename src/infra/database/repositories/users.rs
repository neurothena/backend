use crate::infra::database::{error::DatabaseError, models::users::{NewUser, User}};

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn get_by_id(&self, id: i32) -> Result<Option<User>, DatabaseError>;
    async fn get_by_email(&self, email: &str) -> Result<Option<User>, DatabaseError>;
    async fn insert(&self, user: NewUser) -> Result<(), DatabaseError>;
}