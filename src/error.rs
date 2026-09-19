use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("unable to establish connection with database url: `{db_url}`")]
    ConnectionError { db_url: String }
}