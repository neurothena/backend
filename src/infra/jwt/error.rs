use axum::response::{IntoResponse, Response};
use axum_responses::JsonResponse;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JwtError {
    #[error("Internal token error")]
    CryptoError(#[from] jsonwebtoken::errors::Error),

    #[error("Invalid refresh token")]
    InvalidRefreshError,

    #[error("Invalid access token")]
    InvalidAccessError,

    #[error("Unable to rotate refresh token")]
    RefreshRotationFailed,

    #[error("Failed to encode an access token")]
    AccessTokenFailed,

    #[error("Failed to encode a refresh token")]
    RefreshTokenFailed,
}

impl IntoResponse for JwtError {
    fn into_response(self) -> Response {
        match self {
            JwtError::InvalidAccessError |
            JwtError::InvalidRefreshError => {
                JsonResponse::Unauthorized()
                    .message(self.to_string())
                    .into_response()
            }

            JwtError::AccessTokenFailed |
            JwtError::RefreshRotationFailed |
            JwtError::RefreshTokenFailed |
            JwtError::CryptoError(_) => {
                JsonResponse::InternalServerError()
                    .message(self.to_string())
                    .into_response()
            }
        }
    }
}