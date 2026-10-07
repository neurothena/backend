use diesel_async::pooled_connection::bb8::RunError;
use thiserror::Error;
use diesel::result::Error as DieselError;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Unable to connect via database url: {db_url}")]
    ConnectionError { db_url: String },
   
    #[error("Unable to get connection pool")]
    PoolError(#[from] RunError),

    #[error("Query failed")]
    QueryError(#[from] DieselError)
}