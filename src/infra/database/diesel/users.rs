use diesel::prelude::*;
use diesel_async::{RunQueryDsl};

use crate::{infra::database::{diesel::pool::{DbPool}, error::DatabaseError, models::users::{NewUser, User}, repositories::users::UserRepository}, schema::users};

pub struct DieselUserRepository {
    pool: DbPool
}

impl DieselUserRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl From<DbPool> for DieselUserRepository {
    fn from(value: DbPool) -> Self {
        Self { pool: value }
    }
}

#[async_trait::async_trait]
impl UserRepository for DieselUserRepository {
    async fn insert(&self, user: NewUser) -> Result<(), DatabaseError> {
        let mut connection = self.pool.get().await?;

        diesel::insert_into(users::table)
            .values(&user)
            .execute(&mut connection)
            .await?;

        Ok(())
    }

    async fn get_by_email(&self, email: &str) -> Result<Option<User>, DatabaseError> {
        let mut connection = self.pool.get().await?;

        users::table
            .filter(users::email.eq(email))
            .select(User::as_select())
            .first::<User>(&mut connection)
            .await
            .optional()
            .map_err(DatabaseError::from)
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<User>, DatabaseError> {
        let mut connection = self.pool.get().await?;

        users::table
            .filter(users::id.eq(id))
            .select(User::as_select())
            .first::<User>(&mut connection)
            .await
            .optional()
            .map_err(DatabaseError::from)
    }
}