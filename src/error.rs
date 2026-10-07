use axum::response::{IntoResponse};
use axum_responses::JsonResponse;
use thiserror::Error;
use tokio::task::JoinError;

use crate::infra::{database::error::DatabaseError, jwt::error::JwtError};

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    DBError(#[from] DatabaseError),
    
    #[error("Authentication error: {0}")]
    AuthError(#[from] JwtError),

    #[error("Thread error: {0}")]
    ThreadError(#[from] JoinError),

    #[error("Hash error: {0}")]
    HashError(#[from] argon2::password_hash::Error)
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::AuthError(error) => error.into_response(),
            
            AppError::ThreadError(_) |
            AppError::DBError(_) |
            AppError::HashError(_) => {
                JsonResponse::InternalServerError()
                    .message("Internal server error")
                    .into_response()
            }
        }
    }
}