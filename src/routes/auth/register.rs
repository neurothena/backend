use std::sync::Arc;

use crate::schema::users::{self};
use argon2::{Algorithm, Argon2, Params, PasswordHasher, Version};
use axum::{Json, extract::State};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use axum_responses::JsonResponse;
use diesel::{ExpressionMethods, query_dsl::methods::FilterDsl};
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use time::Duration;

use crate::state::AppState;

#[derive(Deserialize)]
pub(super) struct CreateUserDTO {
    pub email: String,
    pub username: String,
    pub password: String,
}

pub(super) async fn create_user(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<CreateUserDTO>,
) -> (CookieJar, JsonResponse) {
    let mut connection = match state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            return (
                jar,
                JsonResponse::InternalServerError().error(e.to_string()),
            );
        }
    };

    let email_exists: bool = match diesel::select(diesel::dsl::exists(
        users::table.filter(users::email.eq(&body.email)),
    ))
    .get_result(&mut connection)
    .await
    {
        Ok(exists) => exists,
        Err(e) => {
            return (
                jar,
                JsonResponse::InternalServerError().error(e.to_string()),
            );
        }
    };

    if email_exists {
        return (
            jar,
            JsonResponse::Conflict().message("Email already registered"),
        );
    }

    let config = Arc::clone(&state.config);
    let password_to_hash = body.password.clone();
    let password_hash_result = match tokio::task::spawn_blocking(
        move || -> Result<String, argon2::password_hash::Error> {
            let argon2 = Argon2::new_with_secret(
                config.argon2_pepper.as_bytes(),
                Algorithm::default(),
                Version::default(),
                Params::default(),
            )?;

            let result = argon2
                .hash_password(password_to_hash.as_bytes())?
                .to_string();

            Ok(result)
        },
    )
    .await
    {
        Ok(result) => result,
        Err(_) => return (jar, JsonResponse::InternalServerError()),
    };

    let password_hash = match password_hash_result {
        Ok(hash) => hash,
        Err(_) => return (jar, JsonResponse::InternalServerError()),
    };

    let result = diesel::insert_into(users::table)
        .values((
            users::email.eq(&body.email),
            users::username.eq(body.username),
            users::password_hash.eq(password_hash),
        ))
        .execute(&mut connection)
        .await;

    match result {
        Ok(_) => {
            let (access_token, refresh_token) = match state.create_token_pair(&body.email).await {
                Ok(pair) => pair,
                Err(err) => {
                    return (
                        jar,
                        JsonResponse::InternalServerError().error(err.to_string()),
                    );
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

            return (jar, JsonResponse::Created());
        }
        Err(e) => (jar, JsonResponse::BadRequest().error(e.to_string())),
    }
}
