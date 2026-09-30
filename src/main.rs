use std::sync::Arc;

use axum::{Router, routing::get};
use axum_responses::JsonResponse;
use backend::{config::Config, db::create_pool, routes::auth::auth, state::AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Arc::new(Config::from_env()?);

    let pool = create_pool(&config).await?;

    let state = AppState {
        db_pool: pool,
        config: config,
    };

    let api = Router::new()
        .route("/health", get(|| async { JsonResponse::Ok() }))
        .nest("/auth", auth());

    let app = Router::new().nest("/api", api).with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
