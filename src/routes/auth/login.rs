use axum::{Json, extract::State};
use axum_extra::extract::CookieJar;
use axum_responses::JsonResponse;
use serde::Deserialize;

use crate::{routes::auth::register::CreateUserDTO, state::AppState};

#[derive(Deserialize)]
pub(super) struct LoginDTO {
    email: String,
    password: String,
}

impl From<CreateUserDTO> for LoginDTO {
    fn from(value: CreateUserDTO) -> Self {
        Self {
            email: value.email,
            password: value.password,
        }
    }
}

// pub(super) fn login(
//     State(state): State<AppState>,
//     Json(body): Json<LoginDTO>,
// ) -> (CookieJar, JsonResponse) {

// }
