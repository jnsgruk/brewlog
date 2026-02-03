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
│   ├── ids.rs        # Typed ID wrappers (RoasterId, RoastId, BagId, BrewId, GearId, etc.)
│   ├── listing.rs    # Pagination & sorting (SortKey, ListRequest, Page, PageSize)
│   ├── repositories.rs  # Repository traits
│   └── {entity}.rs   # Entity definitions (roasters, roasts, bags, brews, gear, etc.)
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

SQL implementations live in `infrastructure/repositories/`. Each uses a private `Record` struct (e.g., `BagRecord`) with a `to_domain()` method to convert from database row to domain entity:

```rust
impl BagRecord {
    fn to_domain(self) -> Bag { ... }
}
```

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
    let (request, search) = query.into_request_and_search::<RoasterSortKey>();

    if is_datastar_request(&headers) {
        // Datastar request → return fragment only
        return render_roaster_list_fragment(state, request, search, is_authenticated).await;
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
<!-- Local signals (underscore prefix = not sent to server) -->
<section data-signals:_show-form="false" data-signals:_is-submitting="false">

  <!-- Visibility binding -->
  <div data-show="$_showForm" style="display: none">
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
  <form data-ref="_form"
        data-on:datastar-fetch="evt.detail.type === 'finished' && ($_showForm = false, $_form.reset())">
</section>
```

Key attributes:

- `data-signals:_name="value"` - Local signals (underscore prefix excludes from backend requests)
- `data-show="$_signal"` - Conditional visibility
- `data-on:event="expression"` - Event handlers
- `data-ref="_name"` - DOM element references (underscore prefix for local refs)
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

Handlers accept both JSON and form data via `FlexiblePayload<T>`. When form fields don't map directly to the domain `New*` struct (e.g., the form sends a roaster name that needs to be resolved to an ID), use a `*Submission` newtype that handles the conversion:

```rust
pub(crate) async fn create_roaster(
    payload: FlexiblePayload<NewRoaster>,  // simple — form maps 1:1 to domain
    // or: FlexiblePayload<NewBrewSubmission>  // submission type — needs conversion
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
use super::macros::{define_get_handler, define_enriched_get_handler, define_delete_handler};

// GET /api/v1/roasters/:id → returns JSON
define_get_handler!(get_roaster, RoasterId, Roaster, roaster_repo);

// GET with enriched data (joins related entities) → returns JSON
define_enriched_get_handler!(get_roast, RoastId, RoastWithRoaster, roast_repo, get_with_roaster);

// DELETE /api/v1/roasters/:id → returns fragment for Datastar or 204 for API
define_delete_handler!(
    delete_roaster,
    RoasterId,
    RoasterSortKey,
    roaster_repo,
    render_roaster_list_fragment
);
```

### Route Module Structure

Each list-bearing route module (roasters, roasts, bags, gear, brews) follows the same internal structure:

```rust
// Path constants for full-page and fragment URLs
const ROASTER_PAGE_PATH: &str = "/roasters";
const ROASTER_FRAGMENT_PATH: &str = "/roasters#roaster-list";

// Data loader — calls repo.list() and builds view models via build_page_view()
async fn load_roaster_page(state, request, search)
    -> Result<(Paginated<RoasterView>, ListNavigator<RoasterSortKey>), AppError>

// Page handler — checks is_datastar_request(), returns full page or fragment
pub(crate) async fn roasters_page(...) -> Result<Response, StatusCode>

// Fragment renderer — returns just the list partial for Datastar replacement
async fn render_roaster_list_fragment(state, request, search, is_authenticated)
    -> Result<Response, AppError>
```

The `build_page_view()` helper in `application/routes/support.rs` standardises the conversion from a repo `Page<T>` to `(Paginated<V>, ListNavigator<K>)`:

```rust
let (items, navigator) = build_page_view(page, request, RoasterView::from,
    ROASTER_PAGE_PATH, ROASTER_FRAGMENT_PATH, search);
```

### Error Handling

- Domain errors: `RepositoryError` in `domain/errors.rs`
- HTTP errors: `AppError` in `application/errors.rs` with proper status code mapping
- CLI errors: Use `anyhow::Result` for simplicity

## Table & List Patterns

### Template Structure

List partials live in `templates/partials/` (e.g., `roaster_list.html`, `brew_list.html`). Each follows the same structure:

```
{% import "partials/table.html" as table %}

<div id="{entity}-list">
  {% if items.is_empty() && !navigator.has_search() %}
    <!-- Empty state (no data, no search active) -->
  {% else %}
  <section class="rounded-lg border border-amber-300 bg-amber-100/80 shadow-sm"
    {% if items.has_next() %}data-infinite-scroll data-next-url="..." data-target="#{entity}-list"{% endif %}
  >
    {% call table::search_header(navigator, "#{entity}-list") %}
    <table class="responsive-table ...">
      <thead>...</thead>
      <tbody>...</tbody>
    </table>
    {% if items.is_empty() %}
      <!-- "No results match your search" message -->
    {% endif %}
    {% call table::pagination_header(items, navigator, "#{entity}-list") %}
    {% if items.has_next() %}
    <div class="infinite-scroll-sentinel h-4 md:hidden" aria-hidden="true"></div>
    {% endif %}
  </section>
  {% endif %}
</div>
```

Key points:
- The outer `<div>` with `id="{entity}-list"` is the Datastar fragment target for replacements
- Empty state only shows when there are no items **and** no active search query
- When a search is active but returns no results, the table section renders with the search bar and a "no matches" message

### Shared Table Macros (`templates/partials/table.html`)

Three macros are available:

- **`search_header(navigator, target_selector)`** — search input with debounced Datastar `@get`, pushes URL state via `history.pushState`
- **`pagination_header(items, navigator, target_selector)`** — prev/next buttons, page count, rows-per-page selector; hidden on mobile via `pagination-controls hidden md:flex`
- **`sortable_header(label, key, navigator, target_selector)`** — clickable column header with sort direction arrows

### "Added" Column

Every table has a sortable "Added" column as its **first column**, sorted by `created-at`. It uses a distinct smaller style to visually separate it from content columns:

```html
<td data-label="Added" class="whitespace-nowrap px-4 py-3 text-xs font-medium text-stone-600">
  {{ item.created_at }}
</td>
```

The `text-xs font-medium text-stone-600` classes give it a muted, compact appearance compared to the default `text-sm` body text.

### Actions Column

When a row has multiple action buttons/icons, wrap them in `<div class="inline-flex items-center gap-1">` inside the `<td>` to keep them horizontal. Without this wrapper, block-level elements like `<form>` will stack vertically.

```html
<td data-label="" class="px-4 py-3 text-right">
  <div class="inline-flex items-center gap-1">
    <form class="inline" ...>
      <button type="submit" class="inline-flex h-8 w-8 ...">...</button>
    </form>
    <button type="button" class="inline-flex h-8 w-8 ...">...</button>
  </div>
</td>
```

### Responsive Table Pattern

Tables use the `responsive-table` CSS class which converts rows to card-style layout on mobile (`max-width: 767px`). The CSS in `styles.css` hides `<thead>` and uses `data-label` attributes on `<td>` elements to show field labels.

**Desktop**: combined columns with subtext via `hidden md:block`:

```html
<td data-label="Coffee" class="px-4 py-3 whitespace-nowrap">
  <div class="font-medium">{{ brew.roast_name }}</div>
  <div class="hidden md:block text-xs text-stone-500">{{ brew.roaster_name }}</div>
</td>
```

**Mobile**: separate `<td>` elements with `md:hidden` for each sub-field:

```html
<td data-label="Roaster" class="px-4 py-3 whitespace-nowrap md:hidden">
  {{ brew.roaster_name }}
</td>
```

This keeps the desktop table compact while giving each value its own labelled row in the mobile card view. Conditional sub-fields (e.g., filter paper, city) use `{% if %}` guards around both the desktop subtext and the mobile-only `<td>`.

### Search

Server-side search uses a `q` query parameter. The `ListQuery` struct extracts it and passes it to repository `list()` methods via `SearchFilter`. Repositories apply `LIKE` filtering across entity-specific columns (e.g., name, country, origin).

`ListNavigator` preserves the search term across pagination and sort URL generation via `search_query_base()`.

### Pagination vs Infinite Scroll

- **Desktop** (`md:` breakpoint and above): traditional pagination controls (prev/next, page size selector, result count) via the `pagination_header` macro
- **Mobile** (below `md:`): pagination controls are hidden (`hidden md:flex`); infinite scroll loads the next page automatically

The infinite scroll sentinel (`<div class="infinite-scroll-sentinel h-4 md:hidden">`) must always include `md:hidden` to avoid adding unwanted height to the desktop layout. The JavaScript in `base.html` uses `IntersectionObserver` and only activates on mobile via `matchMedia("(max-width: 767px)")`.

When creating sentinels dynamically in JS, use:
```js
newSentinel.className = "infinite-scroll-sentinel h-4 md:hidden";
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
