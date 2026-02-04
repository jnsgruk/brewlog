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
4. Update `README.md` if the change adds/removes/renames CLI commands, environment variables, or user-facing features
5. Update `scripts/bootstrap-db.sh` if the change adds/removes/renames CLI commands, flags, or entity fields used by the bootstrap script
6. Provide a **draft commit message** using Conventional Commits format

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
├── infrastructure/   # External integrations (database, HTTP clients, third-party APIs)
│   ├── repositories/ # SQL implementations of repository traits
│   ├── client/       # HTTP client for CLI
│   ├── ai/           # OpenRouter LLM integration for AI extraction
│   ├── foursquare.rs # Foursquare Places API for nearby cafe search
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
<section data-signals:_show-form="false" data-signals:_submitting="false">

  <!-- Visibility binding -->
  <div data-show="$_showForm" style="display: none">
    <!-- Two-way binding: signal ↔ input value -->
    <input name="name" data-bind:_roaster-name class="input-field" />
  </div>

  <!-- Event handlers with HTTP actions -->
  <form
    data-on:submit="$_submitting = true; @post('/api/v1/roasters', {
      contentType: 'form',
      responseOverrides: {selector: '#roaster-list', mode: 'replace'}
    })"
    data-ref="_form"
    data-on:datastar-fetch="if (!$_submitting) return;
      if (evt.detail.type === 'finished') { $_submitting = false; $_showForm = false; $_form && $_form.reset() }
      else if (evt.detail.type === 'error') { $_submitting = false }"
  >
    <!-- Form fields -->
  </form>
</section>
```

Key attributes:

- `data-signals:_name="value"` — Local signals (underscore prefix excludes from backend requests)
- `data-show="$_signal"` — Conditional visibility
- `data-bind:_signal-name` — Two-way binding between signal and input value
- `data-on:event="expression"` — Event handlers
- `data-ref="_name"` — DOM element references (underscore prefix for local refs)
- `data-text="$_signal"` — Set element text content from signal
- `data-attr:value="$_signal"` — Set element attribute from signal (used for hidden inputs)
- `@get/@post/@put/@delete(url, options)` — HTTP actions with automatic Datastar headers

#### Signal Naming

Signal names use kebab-case in HTML attributes and auto-convert to camelCase in JS expressions:

- HTML attribute: `data-signals:_roaster-name="''"` or `data-bind:_roaster-name`
- JS expression: `$_roasterName`
- JSON response key: `_roasterName` (camelCase)

#### Two-Way Binding

Use `data-bind:_signal-name` for two-way binding between signals and form inputs. **Do not use `data-model`** — it does not exist in Datastar v1 and is silently ignored.

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

### Static Assets

Static files live in `templates/` and are compiled into the binary via `include_str!()`/`include_bytes!()`. Each file needs an explicit route in `application/routes/mod.rs`:

```rust
.route("/styles.css", get(styles))
.route("/favicon.ico", get(favicon))

async fn styles() -> impl IntoResponse {
    (
        [("content-type", "text/css; charset=utf-8")],
        include_str!("../../../templates/styles.css"),
    )
}
```

There is no `tower-http` static file serving — all assets are embedded at compile time. There are no separate JavaScript files; all interactivity is handled via Datastar attributes and minimal inline JS.

### AI Extraction (Datastar-native)

Pages with AI-powered form filling (roasters, roasts, home page scan) use a fully Datastar-native pattern with **no external JavaScript files**. Extraction endpoints return JSON signal patches that Datastar merges into the signal store, and `data-bind` pushes updated values into form fields automatically.

#### Server Response Format

Extraction endpoints detect Datastar requests and return `application/json` signal patches using `render_signals_json()`:

```rust
// application/routes/support.rs
pub fn render_signals_json(
    signals: &[(&str, serde_json::Value)],
) -> Result<Response, AppError> { ... }

// Usage in a handler:
let signals = vec![
    ("_roaster-name", Value::String(result.name)),
    ("_roaster-country", Value::String(result.country)),
];
render_signals_json(&signals)
```

Signal keys use kebab-case (e.g., `_roaster-name`); the function converts them to camelCase (`_roasterName`) for the JSON response. Datastar processes `application/json` responses as signal patches automatically.

**Important**: Do not return `text/html` fragments with `data-signals` attributes for signal patching — Datastar only processes signal updates from `application/json` responses, not from DOM-patched HTML elements.

#### `datastar-fetch` Event Bubbling

The `datastar-fetch` custom event **bubbles through the DOM**. When a page has multiple forms with `data-on:datastar-fetch` handlers (e.g., an extraction form and a create/save form), each handler will fire for events from *any* `@post`/`@get` in the same DOM tree.

**Every `data-on:datastar-fetch` handler must guard with its own "in progress" signal** to ignore events from other forms:

```html
<!-- Extraction form -->
<form
  data-on:submit="$_extracting = true; @post('/api/v1/extract-roaster', {contentType: 'form'})"
  data-on:datastar-fetch="if (!$_extracting) return;
    if (evt.detail.type === 'finished') { $_extracting = false }
    else if (evt.detail.type === 'error') { $_extracting = false; $_extractError = 'Extraction failed.' }"
>

<!-- Create form (same parent section) -->
<form
  data-on:submit="$_submitting = true; @post('/api/v1/roasters', {contentType: 'form', ...})"
  data-on:datastar-fetch="if (!$_submitting) return;
    if (evt.detail.type === 'finished') { $_submitting = false; $_showForm = false; $_form && $_form.reset() }
    else if (evt.detail.type === 'error') { $_submitting = false }"
>
```

Without these guards, the create form's handler will react to the extraction form's `finished` event and vice versa. Note that `evt.detail.type` fires for `started`, `finished`, and `error` — only reset state on `finished` or `error`, never unconditionally.

#### Extraction Template Pattern

Each extraction-enabled page uses this structure:

```html
<section data-signals:_extracting="false" data-signals:_extract-error="''" data-signals:_submitting="false">
  <!-- Hidden file input — only JS needed (FileReader API) -->
  <input type="file" id="{id}-photo" accept="image/*" capture="environment" class="hidden"
    onchange="if(this.files[0]){const r=new FileReader();r.onload=()=>{
      document.getElementById('{id}-image').value=r.result;
      document.getElementById('{id}-extract-form').requestSubmit()};
      r.readAsDataURL(this.files[0]);this.value=''}" />

  <form id="{id}-extract-form"
    data-on:submit="$_extracting = true; $_extractError = ''; @post('{endpoint}', {contentType: 'form'})"
    data-on:datastar-fetch="if (!$_extracting) return;
      if (evt.detail.type === 'finished') { $_extracting = false }
      else if (evt.detail.type === 'error') { $_extracting = false; $_extractError = 'Extraction failed.' }"
  >
    <input type="hidden" name="image" id="{id}-image" />
    <div data-show="!$_extracting">
      <button type="button" onclick="document.getElementById('{id}-photo').click()">Take Photo</button>
      <input name="prompt" type="text" placeholder="Or describe..." />
      <button type="submit">Go</button>
    </div>
    <div data-show="$_extracting" style="display:none"><!-- spinner --></div>
    <p data-show="$_extractError" data-text="$_extractError" style="display:none"></p>
  </form>

  <!-- Main form with data-bind fields -->
  <form data-on:submit="$_submitting = true; @post(...)">
    <input name="name" data-bind:_roaster-name class="input-field" />
    <!-- ... more fields ... -->
  </form>
</section>
```

The only inline JS is the `onchange` handler for FileReader (reading photos as data URLs) and `onclick` to trigger the hidden file input. Everything else is pure Datastar.

### Foursquare Integration

The cafes page uses the [Foursquare Places API](https://docs.foursquare.com/developer/reference/place-search) to search for nearby cafes. The integration lives in `infrastructure/foursquare.rs`.

**Configuration**: Set `BREWLOG_FOURSQUARE_API_KEY` (a Foursquare service API key).

**Search modes** via the `SearchLocation` enum:

```rust
pub enum SearchLocation {
    Coordinates { lat: f64, lng: f64 },  // GPS coords → sends ll + radius params
    Near(String),                         // City name → sends near param
}
```

The route handler in `application/routes/cafes.rs` accepts either `lat`/`lng` query params or a `near` param, builds the appropriate `SearchLocation`, and delegates to `foursquare::search_nearby()`.

**API details**:
- Endpoint: `https://places-api.foursquare.com/places/search`
- Auth: `Authorization: Bearer <key>` header
- Version: `X-Places-Api-Version: 2025-06-17`
- Country codes from the API (ISO 3166-1 alpha-2) are converted to full names via the `isocountry` crate, with short-form overrides for verbose names (e.g. `GB` → "United Kingdom")

**Testing**: Integration tests in `tests/server/nearby_api.rs` use `wiremock::MockServer` to mock the Foursquare API. The `spawn_app_with_foursquare_mock()` helper in `tests/server/helpers.rs` wires up the mock server URL and a test API key.

### Error Handling

- Domain errors: `RepositoryError` in `domain/errors.rs`
- HTTP errors: `AppError` in `application/errors.rs` with proper status code mapping
- CLI errors: Use `anyhow::Result` for simplicity

## Table & List Patterns

### Page Template Structure

Each list page template (`templates/{entity}.html`) places the form and list as **separate siblings** so they become independent flex children of `<main>` (which uses `flex flex-col gap-6`):

```html
<section data-signals:_show-form="false">
  <header>...</header>
  {% if is_authenticated %}
  <div data-show="$_showForm" style="display: none">
    <!-- Form -->
  </div>
  {% endif %}
</section>

{% include "partials/{entity}_list.html" %} {% endblock %}
```

The list include must be **outside** `</section>`, never inside it. Placing it inside removes the flex gap between the form section and the list, causing the table to sit higher on the page.

### List Partial Structure

List partials live in `templates/partials/` (e.g., `roaster_list.html`, `brew_list.html`). Each follows the same structure:

```html
{% import "partials/table.html" as table %}

<div id="{entity}-list" class="mt-6" data-star-scope="{entity}">
  {% if items.is_empty() && !navigator.has_search() %}
  <div
    class="rounded-lg border border-dashed border-amber-300 bg-amber-100/40 px-4 py-6 text-sm text-stone-600"
  >
    <p class="text-center">
      No {entities} recorded yet. Use the form above to add your first {entity}.
    </p>
  </div>
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
    <div class="p-8 text-center text-stone-500">No {entities} match your search.</div>
    {% endif %}
    {% call table::pagination_header(items, navigator, "#{entity}-list") %}
    {% if items.has_next() %}
    <div class="infinite-scroll-sentinel h-4 md:hidden" aria-hidden="true"></div>
    {% endif %}
  </section>
  {% endif %}
</div>
```

Required attributes and elements on every list partial:

| Element | Requirement |
|---------|-------------|
| Outer `<div>` | `id="{entity}-list"`, `class="mt-6"`, `data-star-scope="{entity}"` |
| Empty state | Conditional on `items.is_empty() && !navigator.has_search()`, dashed amber border |
| Table wrapper | `<section>` (not `<div>`) |
| Search header | `{% call table::search_header(...) %}` |
| No-results msg | Inside the `<section>`, shown when search is active but returns nothing |
| Pagination | `{% call table::pagination_header(...) %}` |
| Scroll sentinel | `class="infinite-scroll-sentinel h-4 md:hidden"` |

**Exception**: the bags partial uses a dual-section layout (open-bag cards + history table) and does not follow this pattern exactly.

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
4. **Tests**: Integration tests in `tests/cli/` and `tests/server/`. External API calls are mocked with `wiremock` (see `spawn_app_with_foursquare_mock()` for the pattern)
5. **Commits**: Use Conventional Commit format (`feat:`, `fix:`, `refactor:`, etc.)
6. **Commit authorship**: Never add "Co-Authored-By" trailers to commit messages
7. **Commit signing**: Never use `--no-gpg-sign` when committing — always allow the default GPG signing
8. **Committing**: Do not commit unless explicitly asked to — provide a draft commit message instead
9. **JavaScript style**: Use ES6+ syntax — `const`/`let`, arrow functions, template literals. Prefer `if`/`else` and `switch` over ternary operators; use ternaries only sparingly for simple, single-level expressions — never nest them

## Communication Style

- Be direct and factual
- Analyse root causes before proposing solutions
- Prefer simple solutions over complex ones
- When proposing changes, explain the trade-offs
