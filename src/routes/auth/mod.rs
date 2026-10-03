use axum::{Router, routing::get, routing::post};

use crate::{
    routes::auth::{register::create_user, token::refresh},
    state::AppState,
};

mod login;
mod register;
mod token;

pub fn auth() -> Router<AppState> {
    Router::new()
        .route("/register", post(create_user))
        .route("/refresh", get(refresh))
}
