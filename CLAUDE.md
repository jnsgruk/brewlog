# Claude Code Guidelines for Brewlog

## Project Overview

Brewlog is a self-hosted coffee logging platform built in Rust. It provides:
- HTTP server with web UI (Axum + Askama templates + Datastar)
- REST API for programmatic access
- CLI client for command-line operations
- SQLite/PostgreSQL database support (feature-flagged)

## Build & Test Commands

```bash
cargo build                    # Build the project
cargo test                     # Run all tests
cargo clippy --allow-dirty --fix  # Lint and auto-fix
cargo fmt                      # Format code
```

**Before committing**: Always run `cargo clippy --allow-dirty --fix && cargo fmt` and fix any issues.

## Architecture

The codebase follows **Clean Architecture / Domain-Driven Design** with four layers:

```
src/
├── domain/           # Pure business logic, no external dependencies
│   ├── errors.rs     # RepositoryError enum
│   ├── ids.rs        # Typed ID wrappers (RoasterId, RoastId, BagId)
│   ├── repositories.rs  # Repository traits
│   └── {entity}.rs   # Entity definitions (roasters, roasts, bags, etc.)
│
├── infrastructure/   # External integrations (database, HTTP client)
│   ├── repositories/ # SQL implementations of repository traits
│   ├── client/       # HTTP client for CLI
│   └── database.rs   # Database pool abstraction
│
├── application/      # HTTP server, routes, middleware
│   ├── routes/       # Axum route handlers
│   └── errors.rs     # HTTP error mapping
│
└── presentation/     # User interfaces
    ├── cli/          # CLI commands and argument parsing
    └── web/          # View models for templates
```

**Dependency flow**: `presentation → application → domain ← infrastructure`

## Code Patterns

### Repository Pattern

All data access goes through trait-based repositories defined in `domain/repositories.rs`:

```rust
#[async_trait]
pub trait RoasterRepository {
    async fn insert(&self, roaster: NewRoaster) -> Result<Roaster, RepositoryError>;
    async fn get(&self, id: RoasterId) -> Result<Roaster, RepositoryError>;
    // ...
}
```

SQL implementations live in `infrastructure/repositories/`.

### Typed IDs

Use the typed ID wrappers from `domain/ids.rs` to prevent mixing up IDs:

```rust
// Good
fn get_roast(&self, id: RoastId) -> Result<Roast, RepositoryError>

// Bad - raw i64 could be any ID type
fn get_roast(&self, id: i64) -> Result<Roast, RepositoryError>
```

### SQL Query Construction

Use `QueryBuilder` for dynamic queries. For UPDATE queries, use the `push_update_field!` macro:

```rust
use super::macros::push_update_field;

let mut builder = QueryBuilder::new("UPDATE roasters SET ");
let mut sep = false;

push_update_field!(builder, sep, "name", changes.name);
push_update_field!(builder, sep, "country", changes.country);
// ... more fields

if !sep {
    return Err(RepositoryError::unexpected("No fields provided for update"));
}

builder.push(" WHERE id = ");
builder.push_bind(i64::from(id));
```

### Sorting/Ordering

Each repository has an `order_clause()` method for consistent sort query generation:

```rust
fn order_clause(request: &ListRequest<RoasterSortKey>) -> String {
    let dir_sql = match request.sort_direction() {
        SortDirection::Asc => "ASC",
        SortDirection::Desc => "DESC",
    };
    match request.sort_key() {
        RoasterSortKey::Name => format!("LOWER(name) {dir_sql}, created_at DESC"),
        // ...
    }
}
```

### CLI Commands

For simple get/delete commands, use the macros in `presentation/cli/macros.rs`:

```rust
use super::macros::{define_get_command, define_delete_command};

define_get_command!(GetRoasterCommand, get_roaster, RoasterId, roasters);
define_delete_command!(DeleteRoasterCommand, delete_roaster, RoasterId, roasters, "roaster");
```

### Error Handling

- Domain errors: `RepositoryError` in `domain/errors.rs`
- HTTP errors: `AppError` in `application/errors.rs` with proper status code mapping
- CLI errors: Use `anyhow::Result` for simplicity

### Domain Conversion

Records from the database should have an `into_domain()` method or equivalent:

```rust
impl BagRecord {
    fn into_domain(self) -> Bag { ... }
}
```

## Conventions

1. **Method naming**: Use `order_clause()` for sort query builders (not `sort_clause`)
2. **Imports**: Group by `super::`, then `crate::`, with macros imported explicitly
3. **SQL strings**: Use raw strings `r#"..."#` for multi-line queries
4. **Tests**: Integration tests in `tests/cli/` and `tests/server/`
5. **Commits**: Use Conventional Commit format (`feat:`, `fix:`, `refactor:`, etc.)

## Communication Style

- Be direct and factual
- Analyse root causes before proposing solutions
- Prefer simple solutions over complex ones
- When proposing changes, explain the trade-offs
