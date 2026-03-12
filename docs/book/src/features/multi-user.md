# Multi-User

> **Status:** Planned for Phase 6. This feature is not yet implemented.

RustVault will support multiple users with role-based access for shared household finances.

## Planned Features

- **User roles** — owner, admin, member, viewer with different permission levels
- **Shared accounts** — grant access to specific banks and accounts per user
- **Personal accounts** — keep some accounts private to a single user
- **Audit trail** — all changes are attributed to the user who made them
- **Invitation system** — invite household members by email

## Planned Roles

| Role | Permissions |
|------|-------------|
| **Owner** | Full access, manage users, delete instance |
| **Admin** | Manage accounts, categories, rules; view all data |
| **Member** | Add/edit own transactions, view shared accounts |
| **Viewer** | Read-only access to shared accounts |
