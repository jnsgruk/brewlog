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

### Database Migrations

Create new migrations using sqlx:

```bash
sqlx migrate add <migration_name>  # Creates migrations/NNNN_<migration_name>.sql
```

Migration files are plain SQL in the `migrations/` directory, numbered sequentially (e.g., `0008_remove_gear_notes.sql`).

## Workflow Requirements

**Before finishing any task**, always:

1. Run `cargo clippy --allow-dirty --fix && cargo fmt` to lint and format
2. Run `cargo build` to verify compilation
3. Run `cargo test` if changes affect testable code
4. Provide a **draft commit message** using Conventional Commits format

Example commit message:

```
feat(gear): add category filtering to gear list

- Add GearFilter with optional category field
- Update repository to apply filter in SQL WHERE clause
- Add --category flag to CLI list-gear command

```

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

### Datastar Integration

The web UI uses [Datastar](https://data-star.dev/) for reactive updates without full page reloads. This provides HTMX-style interactions with a declarative API.

#### Request Detection

Datastar requests are identified by the `datastar-request: true` header:

```rust
// application/routes/support.rs
pub fn is_datastar_request(headers: &HeaderMap) -> bool {
    headers
        .get("datastar-request")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}
```

#### Fragment Rendering

When a Datastar request is detected, return a fragment instead of a full page:

```rust
pub(crate) async fn roasters_page(...) -> Result<Response, StatusCode> {
    let request = query.into_request::<RoasterSortKey>();

    if is_datastar_request(&headers) {
        // Datastar request → return fragment only
        return render_roaster_list_fragment(state, request, is_authenticated).await;
    }

    // Traditional request → return full page with layout
    let template = RoastersTemplate { ... };
    render_html(template).map(IntoResponse::into_response)
}
```

Fragments are rendered with special headers that tell Datastar where to patch the DOM:

```rust
// application/routes/support.rs
pub fn render_fragment<T: Template>(template: T, selector: &'static str) -> Result<Response, AppError> {
    let html = render_template(template)?;
    let mut response = Html(html).into_response();
    response.headers_mut().insert("datastar-selector", HeaderValue::from_static(selector));
    response.headers_mut().insert("datastar-mode", HeaderValue::from_static("replace"));
    Ok(response)
}
```

#### Frontend Attributes

Templates use Datastar attributes for interactivity:

```html
<!-- Reactive state -->
<section data-signals:show-form="false" data-signals:is-submitting="false">

  <!-- Visibility binding -->
  <div data-show="$showForm" style="display: none">
    <!-- Form content -->
  </div>

  <!-- Event handlers with HTTP actions -->
  <form data-on:submit="@post('/api/v1/roasters', {
    contentType: 'form',
    responseOverrides: {selector: '#roaster-list', mode: 'replace'}
  })">
    <!-- Form fields -->
  </form>

  <!-- Reset form on completion -->
  <form data-ref="form"
        data-on:datastar-fetch="evt.detail.type === 'finished' && ($showForm = false, $form.reset())">
</section>
```

Key attributes:

- `data-signals:name="value"` - Reactive state signals
- `data-show="$signal"` - Conditional visibility
- `data-on:event="expression"` - Event handlers
- `data-ref="name"` - DOM element references
- `@get/@post/@put/@delete(url, options)` - HTTP actions with automatic Datastar headers

#### URL Generation

`ListNavigator` generates URLs for pagination and sorting:

```rust
// presentation/web/views.rs
navigator.page_href(2)          // "/roasters?page=2&..." (full page)
navigator.fragment_page_href(2) // "/roasters?page=2&...#roaster-list" (fragment)
navigator.sort_href(key)        // "/roasters?sort=name&dir=..."
```

#### Flexible Payload Handling

Handlers accept both JSON and form data via `FlexiblePayload<T>`:

```rust
pub(crate) async fn create_roaster(
    payload: FlexiblePayload<NewRoaster>,
) -> Result<Response, ApiError> {
    let (new_roaster, source) = payload.into_parts();

    if is_datastar_request(&headers) {
        render_fragment(state, request, true).await  // Return updated fragment
    } else if matches!(source, PayloadSource::Form) {
        Ok(Redirect::to(&target).into_response())    // Traditional form redirect
    } else {
        Ok((StatusCode::CREATED, Json(roaster)).into_response())  // JSON API
    }
}
```

### Route Handler Macros

For simple get/delete API handlers, use the macros in `application/routes/macros.rs`:

```rust
use super::macros::{define_get_handler, define_delete_handler};

// GET /api/v1/roasters/:id → returns JSON
define_get_handler!(get_roaster, RoasterId, Roaster, roaster_repo);

// DELETE /api/v1/roasters/:id → returns fragment for Datastar or 204 for API
define_delete_handler!(
    delete_roaster,
    RoasterId,
    RoasterSortKey,
    roaster_repo,
    render_roaster_list_fragment
);
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
6. **Commit authorship**: Never add "Co-Authored-By" trailers to commit messages
7. **Commit signing**: Never use `--no-gpg-sign` when committing — always allow the default GPG signing
8. **Committing**: Do not commit unless explicitly asked to — provide a draft commit message instead

## Communication Style

- Be direct and factual
- Analyse root causes before proposing solutions
- Prefer simple solutions over complex ones
- When proposing changes, explain the trade-offs
