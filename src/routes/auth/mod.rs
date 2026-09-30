use axum::{Router, routing::post};

use crate::{routes::auth::register::create_user, state::AppState};

mod login;
mod register;
mod token;

pub fn auth() -> Router<AppState> {
    Router::new().route("/register", post(create_user))
}
