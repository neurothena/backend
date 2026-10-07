use std::sync::Arc;

use crate::{
    config::Config,
    error::AppError,
    infra::{
        database::{
            models::users::NewUser,
            repositories::{Repositories, RepositoryProvider, users::UserRepository},
        },
        jwt::service::JwtService,
    },
    response::EndpointResult,
    routes::auth::token::create_token_cookies,
};
use argon2::{Algorithm, Argon2, Params, PasswordHasher, Version};
use axum::{Json, extract::State};
use axum_extra::extract::CookieJar;
use axum_responses::JsonResponse;
use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct CreateUserDTO {
    pub email: String,
    pub username: String,
    pub password: String,
}

async fn hash_password(config: Arc<Config>, password: String) -> Result<String, AppError> {
    Ok(
        tokio::task::spawn_blocking(move || -> Result<String, argon2::password_hash::Error> {
            let argon2 = Argon2::new_with_secret(
                config.argon2_pepper.as_bytes(),
                Algorithm::default(),
                Version::default(),
                Params::default(),
            )?;

            let result = argon2.hash_password(password.as_bytes())?.to_string();

            Ok(result)
        })
        .await??,
    )
}

pub(super) async fn create_user<P: RepositoryProvider>(
    State(config): State<Arc<Config>>,
    State(repositories): State<Arc<Repositories<P>>>,
    State(jwt_service): State<JwtService>,
    jar: CookieJar,
    Json(body): Json<CreateUserDTO>,
) -> EndpointResult {
    let password_hash = hash_password(config.clone(), body.password.clone()).await?;

    repositories
        .users
        .insert(NewUser {
            email: body.email.clone(),
            username: body.username,
            password_hash,
        })
        .await?;

    let tokens = jwt_service
        .create_token_pair(&body.email, &repositories.user_jti)
        .await?;

    let jar = create_token_cookies(jar, tokens);

    Ok((jar, JsonResponse::Created()).into())
}
