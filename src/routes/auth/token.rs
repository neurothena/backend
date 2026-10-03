use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use axum_responses::JsonResponse;
use diesel::{Selectable, deserialize::Queryable};
use serde::{Deserialize, Serialize};
use time::Duration;

use crate::{state::AppState};

pub(super) async fn refresh(
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, JsonResponse) {
    let previous_refresh_token = match jar.get("jwt_refresh") {
        Some(token) => token.value(),
        None => {
            return (
                jar,
                JsonResponse::Unauthorized().message("Missing refresh token"),
            );
        }
    };

    let (access_token, refresh_token) = match state.regenerate_tokens(previous_refresh_token).await
    {
        Ok(pair) => pair,
        Err(err) => {
            return (jar, JsonResponse::Unauthorized().error(err.to_string()));
        }
    };

    let jar = jar
        .add(
            Cookie::build(("jwt_access", access_token))
                .path("/")
                .same_site(SameSite::Strict)
                .http_only(true)
                .secure(true)
                .max_age(Duration::minutes(15))
                .build(),
        )
        .add(
            Cookie::build(("jwt_refresh", refresh_token))
                .path("/")
                .same_site(SameSite::Strict)
                .http_only(true)
                .secure(true)
                .max_age(Duration::days(30))
                .build(),
        );

    (jar, JsonResponse::Ok())
}

pub(super) async fn authorize(
    State(state): State<AppState>,
    jar: CookieJar,
    req: Request,
    next: Next,
) -> Response {
    let access_token = match jar.get("jwt_access") {
        Some(cookie) => cookie.value(),
        None => {
            return (
                jar,
                JsonResponse::Unauthorized().message("Missing JWT access token"),
            )
            .into_response();
        }
    };

    if let Err(_) = &state.jwt_service.verify(access_token) {
        return (jar, JsonResponse::Unauthorized().message("Missing JWT access token")).into_response();
    }
    else {
        return next.run(req).await;
    }
}
