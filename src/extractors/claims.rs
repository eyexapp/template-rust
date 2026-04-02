use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::error::AppError;
use crate::middleware::auth::Claims;

/// Axum extractor that retrieves JWT `Claims` from request extensions.
///
/// The `JwtAuthLayer` middleware must insert `Claims` into extensions
/// before this extractor is used. If claims are missing, returns 401.
///
/// # Example
///
/// ```rust,ignore
/// async fn protected_handler(claims: AuthClaims) -> impl IntoResponse {
///     format!("Hello, user {}", claims.0.sub)
/// }
/// ```
pub struct AuthClaims(pub Claims);

impl<S: Send + Sync> FromRequestParts<S> for AuthClaims {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Claims>()
            .cloned()
            .map(AuthClaims)
            .ok_or_else(|| AppError::Unauthorized("Missing authentication".into()))
    }
}
