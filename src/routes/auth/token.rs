use std::sync::Arc;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use axum_responses::JsonResponse;
use time::Duration;

use crate::{
    error::AppError,
    infra::{
        database::repositories::{Repositories, RepositoryProvider},
        jwt::{error::JwtError, service::JwtService},
    },
    response::EndpointResult,
};

fn get_refresh_token(jar: &CookieJar) -> Result<&str, JwtError> {
    Ok(jar
        .get("jwt_refresh")
        .ok_or(JwtError::InvalidRefreshError)?
        .value())
}

fn get_access_token(jar: &CookieJar) -> Result<&str, JwtError> {
    Ok(jar
        .get("jwt_access")
        .ok_or(JwtError::InvalidAccessError)?
        .value())
}

pub(super) fn create_token_cookies(
    jar: CookieJar,
    (access, refresh): (String, String),
) -> CookieJar {
    jar.add(
        Cookie::build(("jwt_access", access))
            .path("/")
            .same_site(SameSite::Strict)
            .http_only(true)
            .secure(true)
            .max_age(Duration::minutes(15))
            .build(),
    )
    .add(
        Cookie::build(("jwt_refresh", refresh))
            .path("/")
            .same_site(SameSite::Strict)
            .http_only(true)
            .secure(true)
            .max_age(Duration::days(30))
            .build(),
    )
}

pub(super) async fn refresh<P: RepositoryProvider>(
    State(jwt_service): State<JwtService>,
    State(repositories): State<Arc<Repositories<P>>>,
    jar: CookieJar,
) -> EndpointResult {
    let previous_refresh_token = get_refresh_token(&jar)?;

    let (access_token, refresh_token) = jwt_service
        .regenerate_tokens(previous_refresh_token, &repositories.user_jti)
        .await?;

    let jar = create_token_cookies(jar, (access_token, refresh_token));

    Ok((jar, JsonResponse::Ok()).into())
}

pub(super) async fn authorize(
    State(jwt_service): State<JwtService>,
    jar: CookieJar,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let access_token = get_access_token(&jar)?;

    jwt_service.verify(access_token)?;

    Ok(next.run(req).await)
}
