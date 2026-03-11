//! Repository modules — SQL query layer.
//!
//! Each module encapsulates all SQL operations for a domain entity.
//! Functions take `&PgPool` and return typed rows.

/// Account repository — SQL operations on the `accounts` table.
pub mod account;
/// Audit-log repository — insert and query audit entries.
pub mod audit;
/// Bank repository — SQL operations on the `banks` table.
pub mod bank;
/// Category repository — SQL operations on the `categories` table.
pub mod category;
/// Session repository — refresh-token session persistence.
pub mod session;
/// Tag repository — SQL operations on the `tags` table.
pub mod tag;
/// User repository — SQL operations on the `users` table.
pub mod user;
