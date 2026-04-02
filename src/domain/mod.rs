use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Generic API Response Wrapper ────────────────────────────────────────────

/// Standard successful response envelope.
///
/// ```json
/// { "data": { ... } }
/// ```
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

impl<T: Serialize> ApiResponse<T> {
    pub const fn new(data: T) -> Self {
        Self { data }
    }
}

// ── Pagination ──────────────────────────────────────────────────────────────

/// Query parameters for paginated endpoints.
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

impl PaginationParams {
    /// Calculate the SQL `OFFSET` value.
    #[must_use]
    pub const fn offset(&self) -> u32 {
        (self.page.saturating_sub(1)) * self.per_page
    }
}

const fn default_page() -> u32 {
    1
}

const fn default_per_page() -> u32 {
    20
}

/// Paginated response envelope.
///
/// ```json
/// {
///   "data": [ ... ],
///   "meta": { "page": 1, "per_page": 20, "total": 100 }
/// }
/// ```
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
}

// ── Repository Pattern (Example) ───────────────────────────────────────────
//
// Define repository traits in this module as your domain grows.
// Implement them in a separate `repositories/` module using SQLx.
//
// ```rust
// use uuid::Uuid;
// use crate::error::Result;
//
// pub trait UserRepository: Send + Sync {
//     async fn find_by_id(&self, id: Uuid) -> Result<Option<User>>;
//     async fn find_all(&self, pagination: &PaginationParams) -> Result<Vec<User>>;
//     async fn create(&self, user: &CreateUser) -> Result<User>;
//     async fn update(&self, id: Uuid, user: &UpdateUser) -> Result<User>;
//     async fn delete(&self, id: Uuid) -> Result<()>;
// }
// ```

/// Example entity ID type alias for clarity.
pub type EntityId = Uuid;
