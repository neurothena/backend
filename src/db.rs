use diesel_async::{AsyncPgConnection, pooled_connection::{AsyncDieselConnectionManager, bb8::Pool}};
use crate::{config::Config, error::DatabaseError};

pub type DbPool = Pool<AsyncPgConnection>;

pub async fn create_pool(config: &Config) -> Result<DbPool, DatabaseError> {
    let connection_url = &config.database_url;

    let diesel_config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(connection_url);

    Pool::builder()
        .build(diesel_config)
        .await
        .map_err(|_| DatabaseError::ConnectionError { db_url: connection_url.to_string() })
}