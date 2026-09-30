use std::sync::Arc;

use crate::schema::users::{self};
use argon2::{Algorithm, Argon2, Params, PasswordHasher, Version};
use axum::{Json, extract::State};
use axum_responses::JsonResponse;
use diesel::{ExpressionMethods, query_dsl::methods::FilterDsl};
use diesel_async::RunQueryDsl;
use serde::Deserialize;

use crate::state::AppState;

#[derive(Deserialize)]
pub(super) struct CreateUserDTO {
    email: String,
    username: String,
    password: String,
}

pub(super) async fn create_user(
    State(state): State<AppState>,
    Json(body): Json<CreateUserDTO>,
) -> JsonResponse {
    let mut connection = match state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => return JsonResponse::InternalServerError().error(e.to_string()),
    };

    let email_exists: bool = match diesel::select(diesel::dsl::exists(
        users::table.filter(users::email.eq(&body.email)),
    ))
    .get_result(&mut connection)
    .await
    {
        Ok(exists) => exists,
        Err(e) => return JsonResponse::InternalServerError().error(e.to_string()),
    };

    if email_exists {
        return JsonResponse::Conflict().message("Email already registered");
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
        Err(_) => return JsonResponse::InternalServerError(),
    };

    let password_hash = match password_hash_result {
        Ok(hash) => hash,
        Err(_) => return JsonResponse::InternalServerError(),
    };

    let result = diesel::insert_into(users::table)
        .values((
            users::email.eq(body.email),
            users::username.eq(body.username),
            users::password_hash.eq(password_hash),
        ))
        .execute(&mut connection)
        .await;

    match result {
        Ok(_) => JsonResponse::Created(),
        Err(e) => JsonResponse::BadRequest().error(e.to_string()),
    }
}
