//! Domain models.
//!
//! These are the core domain structs used throughout the application.
//! They are distinct from database row types and are mapped in the repository layer.

/// Account model — checking, savings, credit card, etc.
pub mod account;
/// Audit log entry model.
pub mod audit;
/// Bank / financial institution model.
pub mod bank;
/// Hierarchical category model (income / expense).
pub mod category;
/// Pagination helpers for list endpoints.
pub mod pagination;
/// Refresh-token session model.
pub mod session;
/// User preferences / settings model.
pub mod settings;
/// Tag model for transaction labelling.
pub mod tag;
/// User identity model.
pub mod user;
