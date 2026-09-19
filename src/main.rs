use axum::{Router, routing::get};
use backend::{config::Config, db::{DbPool, create_pool}};

#[derive(Clone)]
pub struct AppState {
    db_pool: DbPool
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;

    let pool = create_pool(&config).await?;

    let state = AppState { db_pool: pool };

    let app = Router::new()
        .route("/api/health", get(|| async { "lgtm!\n" }))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    axum::serve(listener, app).await.unwrap();

    Ok(())
}