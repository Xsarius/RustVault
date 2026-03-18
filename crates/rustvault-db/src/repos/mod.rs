//! Repository modules — SQL query layer.
//!
//! Each module encapsulates all SQL operations for a domain entity.
//! Functions take `&PgPool` and return typed rows.

/// Account repository — SQL operations on the `accounts` table.
pub mod account;
/// Audit-log repository — insert and query audit entries.
pub mod audit;
/// Auto-rule repository — SQL operations on the `auto_rules` table.
pub mod auto_rule;
/// Bank repository — SQL operations on the `banks` table.
pub mod bank;
/// Budget repository — SQL operations on `budgets` and `budget_lines`.
pub mod budget;
/// Category repository — SQL operations on the `categories` table.
pub mod category;
/// Exchange rate repository — SQL operations on the `exchange_rates` table.
pub mod exchange_rate;
/// Import repository — SQL operations on the `imports` table.
pub mod import;
/// Report repository — aggregation queries for visualisation and analysis.
pub mod report;
/// Session repository — refresh-token session persistence.
pub mod session;
/// Tag repository — SQL operations on the `tags` table.
pub mod tag;
/// Transaction repository — SQL operations on the `transactions` table.
pub mod transaction;
/// Transfer repository — SQL operations on the `transfers` table.
pub mod transfer;
/// User repository — SQL operations on the `users` table.
pub mod user;
