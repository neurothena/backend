use axum::{Router, routing::post};

use crate::{routes::auth::register::create_user, state::AppState};

mod register;

pub fn auth() -> Router<AppState> {
    Router::new().route("/register", post(create_user))
}
