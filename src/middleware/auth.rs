use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use tower::{Layer, Service};

use crate::config::AuthConfig;

/// JWT claims payload. Extend this struct with your own fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (typically user ID)
    pub sub: String,
    /// Expiration time (UTC timestamp)
    pub exp: usize,
    /// Issued at (UTC timestamp)
    pub iat: usize,
}

/// Tower layer that validates JWT Bearer tokens.
///
/// Extracts the `Authorization: Bearer <token>` header, decodes with HS256,
/// and inserts `Claims` into request extensions. Requests to paths in
/// `skip_paths` bypass authentication.
///
/// # Usage
///
/// ```rust,ignore
/// use app::middleware::auth::{JwtAuthLayer, Claims};
///
/// let router = Router::new()
///     .route("/protected", get(handler))
///     .layer(JwtAuthLayer::new(&auth_config, vec!["/health".into()]));
/// ```
#[derive(Clone)]
pub struct JwtAuthLayer {
    decoding_key: Arc<DecodingKey>,
    validation: Arc<Validation>,
    skip_paths: Arc<Vec<String>>,
}

impl JwtAuthLayer {
    #[must_use]
    pub fn new(config: &AuthConfig, skip_paths: Vec<String>) -> Self {
        let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_bytes());
        let mut validation = Validation::default();
        validation.set_required_spec_claims(&["sub", "exp", "iat"]);

        Self {
            decoding_key: Arc::new(decoding_key),
            validation: Arc::new(validation),
            skip_paths: Arc::new(skip_paths),
        }
    }
}

impl<S> Layer<S> for JwtAuthLayer {
    type Service = JwtAuthService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        JwtAuthService {
            inner,
            decoding_key: Arc::clone(&self.decoding_key),
            validation: Arc::clone(&self.validation),
            skip_paths: Arc::clone(&self.skip_paths),
        }
    }
}

#[derive(Clone)]
pub struct JwtAuthService<S> {
    inner: S,
    decoding_key: Arc<DecodingKey>,
    validation: Arc<Validation>,
    skip_paths: Arc<Vec<String>>,
}

impl<S> Service<Request<Body>> for JwtAuthService<S>
where
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        // Skip authentication for configured paths
        let path = req.uri().path().to_owned();
        if self.skip_paths.iter().any(|p| path.starts_with(p)) {
            let future = self.inner.call(req);
            return Box::pin(future);
        }

        // Extract Bearer token
        let auth_header = req
            .headers()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(str::to_owned);

        let Some(token) = auth_header else {
            return Box::pin(async { Ok(unauthorized("Missing authorization token")) });
        };

        // Decode and validate JWT
        match decode::<Claims>(&token, &self.decoding_key, &self.validation) {
            Ok(token_data) => {
                req.extensions_mut().insert(token_data.claims);
                let future = self.inner.call(req);
                Box::pin(future)
            }
            Err(err) => {
                tracing::warn!(error = %err, "JWT validation failed");
                Box::pin(async { Ok(unauthorized("Invalid or expired token")) })
            }
        }
    }
}

fn unauthorized(message: &str) -> Response {
    let body = serde_json::json!({
        "error": {
            "code": "UNAUTHORIZED",
            "message": message,
        }
    });

    (StatusCode::UNAUTHORIZED, axum::Json(body)).into_response()
}
