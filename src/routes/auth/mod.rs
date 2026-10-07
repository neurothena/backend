use axum::{Router, routing::get, routing::post};

use crate::{
    infra::database::repositories::RepositoryProvider,
    routes::auth::{register::create_user, token::refresh},
    state::AppState,
};

mod login;
mod register;
mod token;

pub fn auth<P: RepositoryProvider>() -> Router<AppState<P>> {
    Router::new()
        .route("/register", post(create_user::<P>))
        .route("/refresh", get(refresh::<P>))
}
