use axum::response::{IntoResponse, Response};
use axum_extra::extract::CookieJar;
use axum_responses::JsonResponse;

use crate::error::{AppError};

pub struct EndpointResponse(pub (Option<CookieJar>, JsonResponse));

impl IntoResponse for EndpointResponse {
    fn into_response(self) -> Response {
        let (jar_option, json) = self.0;

        match jar_option {
            Some(jar) => (jar, json).into_response(),
            None => json.into_response()
        }
    }
}

impl From<(CookieJar, JsonResponse)> for EndpointResponse {
    fn from((jar, json): (CookieJar, JsonResponse)) -> Self {
        Self((Some(jar), json))
    }
}

impl From<(Option<CookieJar>, JsonResponse)> for EndpointResponse {
    fn from((jar, json): (Option<CookieJar>, JsonResponse)) -> Self {
        Self((jar, json))
    }
}

pub type EndpointResult<T = EndpointResponse> = Result<T, AppError>;