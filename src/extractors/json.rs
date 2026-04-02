use axum::extract::rejection::JsonRejection;
use axum::extract::FromRequest;
use axum::http::Request;
use axum::response::{IntoResponse, Response};
use serde::de::DeserializeOwned;

use crate::error::AppError;

/// JSON extractor with improved error messages.
///
/// Wraps Axum's `Json` extractor but converts deserialization failures
/// into `AppError::Validation` with a descriptive message, instead of
/// returning a generic 400 response.
///
/// # Example
///
/// ```rust,ignore
/// use app::extractors::json::AppJson;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct CreateUser {
///     email: String,
///     name: String,
/// }
///
/// async fn create_user(AppJson(payload): AppJson<CreateUser>) -> impl IntoResponse {
///     // payload is validated and deserialized
/// }
/// ```
pub struct AppJson<T>(pub T);

impl<S, T> FromRequest<S> for AppJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(
        req: Request<axum::body::Body>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req, state).await {
            Ok(axum::Json(value)) => Ok(Self(value)),
            Err(rejection) => {
                let message = match &rejection {
                    JsonRejection::JsonDataError(e) => format!("Invalid JSON data: {e}"),
                    JsonRejection::JsonSyntaxError(e) => format!("Invalid JSON syntax: {e}"),
                    JsonRejection::MissingJsonContentType(e) => {
                        format!("Missing content type: {e}")
                    }
                    JsonRejection::BytesRejection(e) => format!("Failed to read body: {e}"),
                    _ => "Invalid request body".to_owned(),
                };

                Err(AppError::Validation(message).into_response())
            }
        }
    }
}
