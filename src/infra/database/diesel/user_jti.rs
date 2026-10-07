use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{infra::database::{diesel::pool::DbPool, error::DatabaseError, models::user_jti::NewUserJti, repositories::user_jti::UserJtiRepository}, schema::user_jti};

pub struct DieselUserJtiRepository {
    pool: DbPool
}

impl DieselUserJtiRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl From<DbPool> for DieselUserJtiRepository {
    fn from(value: DbPool) -> Self {
        Self { pool: value }
    }
}

#[async_trait::async_trait]
impl UserJtiRepository for DieselUserJtiRepository {
    async fn insert(&self, jti: NewUserJti) -> Result<(), DatabaseError> {
        let mut connection = self.pool.get().await?;

        diesel::insert_into(user_jti::table)
            .values(&jti)
            .execute(&mut connection)
            .await?;

        Ok(())
    }

    async fn revoke(&self, jti: Uuid) -> Result<(), DatabaseError> {
        let mut connection = self.pool.get().await?;

        diesel::delete(
            user_jti::table.filter(user_jti::jti.eq(jti))
        )
            .execute(&mut connection)
            .await?;

        Ok(())
    }

    async fn update(&self, revoked: Uuid, jti: NewUserJti) -> Result<(), DatabaseError> {
        self.revoke(revoked).await?;
        self.insert(jti).await?;

        Ok(())
    }
}