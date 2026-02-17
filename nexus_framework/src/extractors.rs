//! # Custom Extractors
//!
//! Provides enhanced Axum extractors that integrate with the framework's error handling.

use axum::body::Body;
use axum::extract::rejection::JsonRejection;
use axum::extract::FromRequest;
use axum::http::Request;
use serde::de::DeserializeOwned;

use crate::error::AppError;

/// A JSON extractor that returns `AppError::unprocessable` on deserialization failure.
///
/// Unlike Axum's built-in `Json<T>`, this extractor provides user-friendly error messages
/// when the request body cannot be parsed as the expected type.
///
/// # Example
///
/// ```rust
/// use nexus_framework::prelude::*;
/// use nexus_framework::extractors::ValidatedJson;
///
/// #[model]
/// pub struct CreateUser {
///     pub username: String,
///     pub email: String,
/// }
///
/// #[route(POST, "/")]
/// async fn create_user(
///     ValidatedJson(payload): ValidatedJson<CreateUser>,
/// ) -> Result<Json<CreateUser>, AppError> {
///     Ok(Json(payload))
/// }
/// ```
pub struct ValidatedJson<T>(pub T);

#[axum::async_trait]
impl<T> FromRequest<()> for ValidatedJson<T>
where
    T: DeserializeOwned + 'static,
{
    type Rejection = AppError;

    async fn from_request(req: Request<Body>, state: &()) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req, state).await {
            Ok(axum::Json(value)) => Ok(ValidatedJson(value)),
            Err(rejection) => {
                let message = match rejection {
                    JsonRejection::JsonDataError(ref e) => {
                        format!("Invalid JSON data: {}", e)
                    }
                    JsonRejection::JsonSyntaxError(ref e) => {
                        format!("Malformed JSON: {}", e)
                    }
                    JsonRejection::MissingJsonContentType(ref _e) => {
                        "Missing Content-Type: application/json header".to_string()
                    }
                    JsonRejection::BytesRejection(ref e) => {
                        format!("Failed to read request body: {}", e)
                    }
                    _ => "Invalid request body".to_string(),
                };
                Err(AppError::unprocessable(message))
            }
        }
    }
}
