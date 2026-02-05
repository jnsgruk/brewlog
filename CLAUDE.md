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

### When to Use Datastar vs JavaScript

**Use Datastar for:**
- Visibility toggling (`data-show` + signals) — replaces `classList.add/remove("hidden")`
- List CRUD — delete with `confirm() && @delete()`, create with `@post()` + fragment re-render
- Debounced search — `data-on:input__debounce.300ms` + `@get()` with `responseOverrides`
- AI extraction signal patching — server returns `application/json` signal patches via `render_signals_json()`
- Multi-step wizards — step signals (`$_step`) with `data-show="$_step === N"`
- Searchable selection lists — use `<searchable-select>` component with `data-on:change`

**Use JavaScript for:**
- Browser APIs: WebAuthn (`navigator.credentials`), clipboard (`navigator.clipboard`), geolocation (`navigator.geolocation`), FileReader
- Infinite scroll (`IntersectionObserver` in `base.html`) — no native Datastar equivalent
- Theme toggle — must run in `<head>` before DOM renders, manipulates `<html>` data-theme attribute + `localStorage`
- Any flow that requires `window.location.reload()` after completion (delete passkey, revoke token)

**Signal naming conventions for in-progress states:**
- `_extracting` — AI extraction in progress (consistent across home, add, check-in pages)
- `_submitting` — form save/create in progress
- `_extract-error` / `_error` — error message signals
- `_show-{thing}` — boolean visibility toggles (e.g. `_show-passkey-form`)

### Static Assets

Static files live in `static/` and are compiled into the binary via `include_str!()`/`include_bytes!()`. Each file needs an explicit route in `application/routes/mod.rs`:

```rust
.route("/styles.css", get(styles))
.route("/favicon.ico", get(favicon))

async fn styles() -> impl IntoResponse {
    (
        [("content-type", "text/css; charset=utf-8")],
        include_str!("../../../static/css/styles.css"),
    )
}
```

There is no `tower-http` static file serving — all assets are embedded at compile time. Web component JS files live in `static/js/components/` and are served via explicit routes (e.g., `/components/photo-capture.js`, `/components/searchable-select.js`). Most interactivity is handled via Datastar attributes and minimal inline JS.

### CSS Architecture

The UI uses **Tailwind CSS v4** built via the standalone CLI (no Node.js required). The Nix flake provides `tailwindcss_4` in both the devShell and the package `preBuild`.

**Source file**: `static/css/input.css` — the single source of truth for all styles.
**Generated file**: `static/css/styles.css` — gitignored, built automatically by `build.rs`.

#### Build integration

`build.rs` runs `tailwindcss` automatically during `cargo build`. It watches all files under `templates/` and `static/` and re-runs when any change. Release builds pass `--minify`. If `tailwindcss` is not found on `PATH`, the build prints a warning but continues (useful for CI without the CLI installed).

There is **no need to run `tailwindcss` manually** — `cargo build` / `cargo run` handles it. You can still run it directly for validation or to check output without a full Rust compile:

```bash
tailwindcss -i static/css/input.css -o static/css/styles.css
```

For continuous CSS-only iteration, the `--watch` flag is useful alongside `cargo watch`:

```bash
tailwindcss -i static/css/input.css -o static/css/styles.css --watch  # terminal 1
cargo watch -x run                                                      # terminal 2
```

#### Design Tokens

Colors are defined as raw CSS custom properties in `:root` (light) and `[data-theme="dark"]` (dark), then mapped to Tailwind utilities via `@theme`:

| Token | Light | Dark | Tailwind class |
|-------|-------|------|---------------|
| `--page` | `#fafaf9` (stone-50) | `#1c1917` | `bg-page` |
| `--surface` | `#ffffff` | `#292524` | `bg-surface` |
| `--surface-alt` | `#f5f5f4` (stone-100) | `#44403c` | `bg-surface-alt` |
| `--border` | `#e7e5e4` (stone-200) | `#57534e` | default `border` |
| `--accent` | `#c2410c` (orange-700) | `#ea580c` | `bg-accent`, `text-accent` |
| `--accent-hover` | `#ea580c` | `#f97316` | `bg-accent-hover`, `hover:bg-accent-hover` |
| `--accent-subtle` | `#fff7ed` (orange-50) | `rgba(234,88,12,0.1)` | `bg-accent-subtle` |
| `--accent-text` | `#ffffff` | `#ffffff` | `text-accent-text` |
| `--text` | `#1c1917` (stone-900) | `#e7e5e4` (stone-200) | `text-text` |
| `--text-secondary` | `#57534e` (stone-600) | `#d6d3d1` (stone-300) | `text-text-secondary` |
| `--text-muted` | `#78716c` (stone-500) | `#a8a29e` (stone-400) | `text-text-muted` |

A base layer rule sets the default border color so bare `border` / `divide-y` classes use the theme:

```css
@layer base {
  *, ::after, ::before {
    border-color: var(--border);
  }
}
```

Neutral text colors use the tokenised utilities (`text-text`, `text-text-secondary`, `text-text-muted`) which adapt automatically between light and dark themes. Do not use hardcoded `text-stone-*` classes for neutral text — always use the token-based classes.

#### Dark Mode

- `[data-theme="dark"]` attribute on `<html>` — set via a `<script>` in `<head>` that reads `localStorage` / `prefers-color-scheme` before body render (prevents flash)
- Sun/moon toggle in nav saves preference to `localStorage`
- Tailwind `@custom-variant dark (&:where([data-theme="dark"] *))` enables the `dark:` prefix as an escape hatch

#### Card Styling

Cards use `rounded-lg border bg-surface p-4` — no shadows. The flat design relies on borders and surface colour differentiation rather than elevation.

#### Pills

Three pill variants are defined in `input.css` as CSS component classes. Use these instead of ad-hoc `rounded-full` + colour utility combinations:

| Class | Border | Text | Background | Use for |
|-------|--------|------|------------|---------|
| `.pill.pill-accent` | accent (40% opacity) | accent | accent-subtle | Tasting notes, highlighted tags |
| `.pill.pill-muted` | default border | text-secondary | surface-alt | Categories, kind labels, neutral status |
| `.pill.pill-success` | emerald | emerald | emerald bg | Positive status ("Open") |

Always include the `.pill` base class alongside the variant:

```html
<span class="pill pill-muted">Grinder</span>
<span class="pill pill-accent">Chocolate</span>
<span class="pill pill-success">Open</span>
```

Extra utilities (e.g., `w-28 justify-center whitespace-nowrap`) can be added alongside the pill classes when layout requires it. Do not override the pill's colour/border/padding with utility classes.

### UI Style Guide

#### Typography Hierarchy

| Level | Classes | Use |
|-------|---------|-----|
| Page title | `text-3xl font-semibold` | Top-level `<h1>` on each page |
| Section title | `text-lg font-semibold text-text` | `<h2>` / `<h3>` for form card titles |
| Subsection title | `text-base font-semibold text-text` | `<h3>` within cards (e.g. "Confirm cafe details") |
| Form section label | `text-xs font-semibold text-text-muted uppercase tracking-wide` | `<h4>` grouping related fields (e.g. "Coffee", "Grinder", "Water") |
| Body / description | `text-sm text-text-secondary` | Paragraph text below headings |
| Muted secondary | `text-xs text-text-muted` | Subtext in option lists, metadata |
| Accent title | `text-2xl font-semibold text-accent` | Login / register page headers |

Page headers follow a consistent structure — title + constrained description:

```html
<header class="flex flex-col gap-2">
  <h1 class="text-3xl font-semibold">Page Title</h1>
  <p class="max-w-2xl text-sm text-text-secondary">Short description of what the page does.</p>
</header>
```

#### Buttons

**Primary** — main actions (Save, Submit, Check In):
```html
<button class="rounded-md bg-accent px-4 py-2 text-sm font-semibold text-accent-text transition hover:bg-accent-hover">
```
Use `w-full` for full-width CTAs. Use `py-3` for larger touch targets on final submission buttons. Add `disabled:opacity-50` with `data-attr:disabled` for loading states.

**Secondary** — cancel, back, alternative actions:
```html
<button class="rounded-md border px-4 py-2 text-sm font-semibold text-text transition hover:border-accent hover:text-text">
```

**Text-only** — minimal actions in summary bars (Change, Back):
```html
<button class="text-xs text-text-muted hover:text-text">
```

**Icon-only** — delete, revoke actions in rows/cards:
```html
<button class="inline-flex h-8 w-8 items-center justify-center rounded-md p-1 text-text-muted transition hover:text-red-600">
```

**Link-style** — inline actions with icon (Brew Again, View all):
```html
<a class="inline-flex items-center gap-1 text-sm font-medium text-accent hover:text-accent-hover">
```

**Adjustment (+/-)** — numeric stepper buttons flanking an input:
```html
<div class="flex items-center gap-2">
  <button type="button" class="btn-adjust" data-on:click="$_value = Math.max(0, $_value - 0.5)">-</button>
  <input type="number" class="input-field flex-1 text-center" />
  <button type="button" class="btn-adjust" data-on:click="$_value = $_value + 0.5">+</button>
</div>
```

Submit buttons are always right-aligned: `<div class="flex items-center justify-end gap-2">`. When paired with a secondary button (Cancel/Back), the secondary comes first.

#### Forms

**Label + input pattern** — every form field wraps label text and input in a flex column:

```html
<label class="flex flex-col gap-1 text-sm">
  <span class="text-text">Field Name *</span>
  <input type="text" name="field" required class="input-field" placeholder="Example" />
</label>
```

Mark required fields with `*` in the label text. The `input-field` class is defined in `input.css` and handles border, padding, rounded corners, and focus ring.

**Multi-column grids**: `<div class="grid gap-4 sm:grid-cols-2">` — stacks on mobile, 2 columns on `sm:`. For fields that should span both columns: `class="sm:col-span-2"`.

**Form card wrapper**: `<div class="rounded-lg border bg-surface p-5">` with internal `flex flex-col gap-4` (or `gap-6` for forms with section headers).

**Form section headers**: use `<h4>` with the form section label style to visually group related fields within a single form (e.g. "Coffee", "Grinder", "Water" sections in the brew form).

**Checkbox pattern**:
```html
<label class="inline-flex items-center gap-2 text-sm cursor-pointer">
  <input type="checkbox" name="flag" value="true" class="accent-orange-700" />
  <span class="font-semibold text-text">Label text</span>
</label>
```

#### Feedback States

**Error text** — inline beneath form or section:
```html
<p data-show="$_error" data-text="$_error" style="display:none" class="text-sm text-red-600"></p>
```

**Alert boxes** — for persistent status messages:

| Variant | Classes |
|---------|---------|
| Error | `rounded-md bg-red-100 border border-red-300 p-3 text-sm text-red-800` |
| Success | `rounded-md bg-green-100 border border-green-300 p-4 text-sm text-green-800` |
| Warning | `rounded-md bg-yellow-100 border border-yellow-300 p-3 text-sm text-yellow-800` |
| Info (scan) | `rounded-md bg-green-50 border border-green-200 px-3 py-2 text-sm text-green-800` |

**Loading spinner** — shown while an async action is in progress:
```html
<div data-show="$_submitting" style="display:none" class="flex items-center gap-3 text-sm text-accent">
  {% call icons::spinner("h-5 w-5") %}
  Saving&hellip;
</div>
```

The spinner icon includes `animate-spin text-accent` internally. Always pair with `data-show` bound to the in-progress signal, and include a descriptive `&hellip;`-suffixed label.

#### Empty / Prerequisite States

When a form requires a parent entity that doesn't exist yet (e.g. "roasts need a roaster"), show a static message instead of the form:

```html
{% if roaster_options.is_empty() %}
<div class="text-sm text-text-secondary">
  <h3 class="text-lg font-semibold text-text">Add a roaster first</h3>
  <p class="mt-2">Roasts need a roaster. Add a roaster above to enable this form.</p>
</div>
{% else %}
<!-- actual form -->
{% endif %}
```

#### Selected Item Indicator

When a user selects an item (cafe, roast) in a multi-step flow, show a summary bar with an option to change:

```html
<div class="rounded-lg border bg-surface px-4 py-3 flex items-center justify-between"
  data-show="$_step > 1 && $_cafeName" style="display: none">
  <div class="flex items-center gap-2">
    {% call icons::location("h-4 w-4 text-accent shrink-0") %}
    <span class="font-medium text-text" data-text="$_cafeName"></span>
    <span class="text-xs text-text-muted" data-show="$_cafeCity" data-text="$_cafeCity" style="display:none"></span>
  </div>
  <button type="button" class="text-xs text-text-muted hover:text-text"
    data-on:click="$_cafeName = ''; $_step = 1">Change</button>
</div>
```

Icon + name + muted secondary text on the left, text-only "Change"/"Back" button on the right.

#### Navigation Active State

Each page template passes `nav_active` to the layout. Nav links use conditional classes:

```html
{% if nav_active == "data" %}
  class="text-accent font-medium"
{% else %}
  class="text-text-muted hover:text-text transition"
{% endif %}
```

Desktop nav: `hidden md:flex items-center gap-6`. Mobile nav: toggled via `data-show` bound to a signal.

#### Responsive Patterns

- **Desktop-only**: `hidden md:flex` or `hidden md:block`
- **Mobile-only**: `md:hidden`
- Pagination controls use `hidden md:flex`; mobile uses infinite scroll
- Desktop table combines related fields into one cell with `hidden md:block` subtext; mobile uses separate `md:hidden` cells with `data-label`

#### Spacing Conventions

| Context | Gap | Example |
|---------|-----|---------|
| Major form sections | `gap-6` | Between "Coffee", "Grinder", "Water" in brew form |
| Related field groups | `gap-4` | Between inputs in a `grid sm:grid-cols-2` |
| Label to input | `gap-1` | `flex flex-col gap-1` wrapping label + input |
| Icon + text pairs | `gap-2` | Summary bars, button content |
| Navigation links | `gap-6` | Desktop nav items |
| Button groups | `gap-2` | Cancel + Save buttons |
| Page sections | `gap-8` | `<main>` flex column (set in `base.html`) |

### `<brew-photo-capture>` Web Component

The `<brew-photo-capture>` custom element (defined in `static/js/components/photo-capture.js`, served at `/components/photo-capture.js`) encapsulates the FileReader + hidden file input pattern used for photo-based AI extraction. It replaces 5 identical inline `onchange` handlers.

**Attributes**:
- `target-input` — ID of the hidden `<input>` that receives the data URL
- `target-form` — ID of the `<form>` to submit after reading the photo

**Usage**: Wrap the trigger button content in the element. Clicking anywhere inside opens the camera/file picker:

```html
<brew-photo-capture target-input="scan-image" target-form="scan-extract-form"
  class="inline-flex items-center gap-2 rounded-md border px-3 py-2 text-sm font-medium cursor-pointer">
  {% call icons::camera("h-4 w-4") %}
  Take Photo
</brew-photo-capture>
```

The component creates a hidden file input internally, reads the selected file as a data URL, sets the target input's value, and submits the target form — all without any inline JS in the template.

### `<searchable-select>` Web Component

The `<searchable-select>` custom element (defined in `static/js/components/searchable-select.js`, served at `/components/searchable-select.js`) encapsulates the search-filter-select-clear pattern used for picking from a list of options. It replaces the repeated pattern of search input + button list + hidden input + selected display + clear button.

**Attributes**:
- `name` — Name for the hidden `<input>` included in form submission
- `placeholder` — Placeholder text for the search input (default: "Type to search…")

**Events**:
- `change` — Fired when an option is selected. `detail: { value, display, data }` where `data` is a spread of the button's `dataset`
- `clear` — Fired when the selection is cleared

**Usage**: Place `<button>` elements as children with `value` and `data-display` attributes:

```html
<searchable-select name="roaster_id" placeholder="Type to search roasters…">
  {% for roaster in roaster_options %}
  <button type="button" value="{{ roaster.id }}" data-display="{{ roaster.name }}"
    class="w-full px-3 py-2 text-left text-sm hover:bg-surface-alt transition">
    <span class="font-medium text-text">{{ roaster.name }}</span>
  </button>
  {% endfor %}
</searchable-select>
```

The component creates the search input, hidden form input, selected-value display with clear button, and filter logic internally. On selection, the hidden input's value is set to the button's `value` and the display shows `data-display`. For plain HTML form POST (like the add page), no Datastar signals are needed — the hidden input submits the value directly.

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
  <form id="{id}-extract-form"
    data-on:submit="$_extracting = true; $_extractError = ''; @post('{endpoint}', {contentType: 'form'})"
    data-on:datastar-fetch="if (!$_extracting) return;
      if (evt.detail.type === 'finished') { $_extracting = false }
      else if (evt.detail.type === 'error') { $_extracting = false; $_extractError = 'Extraction failed.' }"
  >
    <input type="hidden" name="image" id="{id}-image" />
    <div data-show="!$_extracting">
      <brew-photo-capture target-input="{id}-image" target-form="{id}-extract-form"
        class="inline-flex items-center gap-2 rounded-md border px-3 py-2 cursor-pointer">
        Take Photo
      </brew-photo-capture>
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

Photo capture uses the `<brew-photo-capture>` web component (see above). Everything else is pure Datastar — no inline JS needed in templates.

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

### Logging & Tracing

The server uses `tracing` + `tracing-subscriber` for structured logging, with `tower-http`'s `TraceLayer` for automatic HTTP request/response logging.

**Configuration**:
- `RUST_LOG` — filter level (default `info`)
- `RUST_LOG_FORMAT=json` — opt-in structured JSON output (default is compact human-readable)

**Initialisation** is in `src/main.rs` via `init_tracing()`. The `TraceLayer` is added to the Axum middleware stack in `application/routes/mod.rs`.

#### Error logging rules

Never silently discard errors. Every error path must be logged before being mapped or discarded:

1. **`map_err(|_| StatusCode::*)` patterns** — always log the original error before mapping:
   ```rust
   .map_err(|err| {
       error!(error = %err, "failed to list passkeys");
       StatusCode::INTERNAL_SERVER_ERROR
   })?;
   ```
   Use `warn!` for client-caused failures (auth, validation), `error!` for server-side failures.

2. **`.ok()` / `.ok()?` patterns** — replace with explicit match that logs:
   ```rust
   let session = match state.session_repo.get_by_token_hash(&hash).await {
       Ok(s) => s,
       Err(err) => {
           warn!(error = %err, "session lookup failed");
           return None;
       }
   };
   ```

3. **Fire-and-forget (`let _ = ...`)** — always log failures:
   ```rust
   if let Err(err) = state.timeline_repo.insert(event).await {
       warn!(error = %err, entity_type = "bag", "failed to record timeline event");
   }
   ```

4. **Background tasks (`tokio::spawn`)** — log inside the spawned future:
   ```rust
   tokio::spawn(async move {
       if let Err(err) = token_repo.update_last_used(token_id).await {
           warn!(error = %err, %token_id, "failed to update token last_used");
       }
   });
   ```

#### CRUD operation logging

Every successful create, update, and delete operation logs at `info!` level with the entity ID and key fields:

```rust
info!(roaster_id = %roaster.id, name = %roaster.name, "roaster created");
info!(%id, "roaster updated");
info!(%id, "entity deleted");  // in define_delete_handler! macro
```

#### Security event logging

Authentication and credential lifecycle events are logged at `info!` level:

- Token create/revoke: `info!(token_id, token_name, user_id, "API token created/revoked")`
- Passkey delete: `info!(passkey_id, user_id, "passkey deleted")`
- Login/logout: `info!("user logged out")`, `info!(user_id, "user authenticated via passkey")`
- CLI token creation: `info!(user_id, "CLI token created via passkey auth")`

## Table & List Patterns

### Page Template Structure

Each list page template (`templates/pages/{entity}.html`) places the form and list as **separate siblings** so they become independent flex children of `<main>` (which uses `flex flex-col gap-6`):

```html
<section data-signals:_show-form="false">
  <header>...</header>
  {% if is_authenticated %}
  <div data-show="$_showForm" style="display: none">
    <!-- Form -->
  </div>
  {% endif %}
</section>

{% include "partials/lists/{entity}_list.html" %} {% endblock %}
```

The list include must be **outside** `</section>`, never inside it. Placing it inside removes the flex gap between the form section and the list, causing the table to sit higher on the page.

### List Partial Structure

List partials live in `templates/partials/lists/` (e.g., `roaster_list.html`, `brew_list.html`). Each follows the same structure:

```html
{% import "partials/lists/table.html" as table %}

<div id="{entity}-list" class="mt-6" data-star-scope="{entity}">
  {% if items.is_empty() && !navigator.has_search() %}
  <div
    class="rounded-lg border border-dashed px-4 py-6 text-sm text-text-secondary"
  >
    <p class="text-center">
      No {entities} recorded yet. Use the form above to add your first {entity}.
    </p>
  </div>
  {% else %}
  <section class="rounded-lg border bg-surface"
    {% if items.has_next() %}data-infinite-scroll data-next-url="..." data-target="#{entity}-list"{% endif %}
  >
    {% call table::search_header(navigator, "#{entity}-list") %}
    <table class="responsive-table ...">
      <thead>...</thead>
      <tbody>...</tbody>
    </table>
    {% if items.is_empty() %}
    <div class="p-8 text-center text-text-muted">No {entities} match your search.</div>
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
| Empty state | Conditional on `items.is_empty() && !navigator.has_search()`, dashed border |
| Table wrapper | `<section>` (not `<div>`) |
| Search header | `{% call table::search_header(...) %}` |
| No-results msg | Inside the `<section>`, shown when search is active but returns nothing |
| Pagination | `{% call table::pagination_header(...) %}` |
| Scroll sentinel | `class="infinite-scroll-sentinel h-4 md:hidden"` |

**Exception**: the bags partial uses a dual-section layout (open-bag cards + history table) and does not follow this pattern exactly.

### Shared Table Macros (`templates/partials/lists/table.html`)

Three macros are available:

- **`search_header(navigator, target_selector)`** — search input with debounced Datastar `@get`, pushes URL state via `history.pushState`
- **`pagination_header(items, navigator, target_selector)`** — prev/next buttons, page count, rows-per-page selector; hidden on mobile via `pagination-controls hidden md:flex`
- **`sortable_header(label, key, navigator, target_selector)`** — clickable column header with sort direction arrows

### "Added" Column

Every table has a sortable "Added" column as its **first column**, sorted by `created-at`. It uses a distinct smaller style to visually separate it from content columns:

```html
<td data-label="Added" class="whitespace-nowrap px-4 py-3 text-xs font-medium text-text-secondary">
  {{ item.created_at }}
</td>
```

The `text-xs font-medium text-text-secondary` classes give it a muted, compact appearance compared to the default `text-sm` body text.

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
  <div class="hidden md:block text-xs text-text-muted">{{ brew.roaster_name }}</div>
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

## Keeping Code Flat and Simple

These rules prevent complexity from creeping in through closures, iterator chains, and duplicated blocks.

### Extract large closures into named functions

If a closure passed to `.filter_map()`, `.map()`, or similar exceeds roughly 10 lines, extract it into a private named function. The iterator call should read as a single scannable line:

```rust
// Good — intent is obvious at the call site
let cafes = results.into_iter().filter_map(|p| parse_cafe(p, location)).collect();

// Bad — 30-line closure buries logic inside an iterator chain
let cafes = results.into_iter().filter_map(|place| {
    // ... 30 lines of validation, fallbacks, struct construction ...
}).collect();
```

### DRY repeated blocks that differ only by a parameter

When three or more code blocks follow the same structure and differ only by a value (an enum variant, a column name, a type), extract a helper function parameterised on that value:

```rust
// Good
let grinder_options = load_gear_options(state, GearCategory::Grinder, &request).await?;
let brewer_options  = load_gear_options(state, GearCategory::Brewer, &request).await?;

// Bad — same 10-line block copy-pasted three times
let grinders = state.gear_repo.list(GearFilter::for_category(GearCategory::Grinder), &request, None)
    .await.map_err(AppError::from)?;
let grinder_options: Vec<GearOptionView> = grinders.items.into_iter().map(GearOptionView::from).collect();
// ... repeat for brewers, filter papers ...
```

### Prefer `match` over chained `if/else-if` when branching on a value

When the condition is testing the same variable against different values, use `match` instead of `if/else-if` chains. This makes all branches visible at the same indentation level:

```rust
// Good — flat, each case at the same level
match label_lower.as_str() {
    "homepage" | "website" => { /* extract link */ }
    "position" => { /* build map link */ }
    _ => { /* default detail */ }
}

// Bad — nested if/else-if obscures the branching structure
if label.eq_ignore_ascii_case("homepage") || label.eq_ignore_ascii_case("website") {
    // ...
} else if label.eq_ignore_ascii_case("position") {
    // ...
} else {
    // ...
}
```

### Extract shared predicates into named helpers

When the same boolean condition appears in multiple functions, extract it:

```rust
fn is_blank(value: &str) -> bool {
    value.is_empty() || value == "\u{2014}"
}

// Then use it everywhere instead of repeating the condition inline
.filter(|v| !is_blank(v))
```

### Use generic helpers for repeated structural patterns

When the same match/decode/encode pattern appears three or more times with different types, write a small generic helper:

```rust
fn decode_json_vec<T: DeserializeOwned>(raw: Option<String>, label: &str) -> anyhow::Result<Vec<T>> {
    match raw {
        Some(s) if !s.is_empty() => from_str(&s).with_context(|| format!("failed to decode {label}: {s}")),
        _ => Ok(Vec::new()),
    }
}

// Replaces three identical match blocks differing only in type and error message
let details = decode_json_vec(self.details_json, "timeline event details")?;
let tasting_notes = decode_json_vec(self.tasting_notes_json, "timeline tasting notes")?;
```

## Conventions

1. **Method naming**: Use `order_clause()` for sort query builders (not `sort_clause`)
2. **Imports**: Group by `super::`, then `crate::`, with macros imported explicitly
3. **SQL strings**: Use raw strings `r#"..."#` for multi-line queries
4. **Tests**: Integration tests in `tests/cli/` and `tests/server/`. External API calls are mocked with `wiremock` (see `spawn_app_with_foursquare_mock()` for the pattern)
5. **Commits**: Use Conventional Commit format (`feat:`, `fix:`, `refactor:`, etc.)
6. **Commit authorship**: Never add "Co-Authored-By" trailers to commit messages
7. **Commit signing**: Never use `--no-gpg-sign` when committing — always allow the default GPG signing
8. **Committing**: Never try to commit unless explicitly prompted — provide a draft commit message instead
9. **JavaScript style**:
   - **Never use `var`** — always `const` (default) or `let` (only when reassignment is needed)
   - **Never use `function` declarations** — always arrow functions assigned to `const`:
     ```js
     // Good
     const showForm = () => { ... };
     const handleClick = (evt) => { ... };

     // Bad
     function showForm() { ... }
     ```
   - **Always use template literals** for string interpolation — never `+` concatenation:
     ```js
     // Good
     `Failed (HTTP ${response.status}).`

     // Bad
     'Failed (HTTP ' + response.status + ').'
     ```
   - Prefer `if`/`else` and `switch` over ternary operators; use ternaries only sparingly for simple, single-level expressions — never nest them
   - **Inline `onclick` handlers + global arrow functions** — pages with imperative JS (e.g. account, checkin) define `const` arrow functions at `<script>` top level and call them from `onclick="fnName()"` in the HTML. Do not use `DOMContentLoaded` + `addEventListener` for these pages

## Communication Style

- Be direct and factual
- Analyse root causes before proposing solutions
- Prefer simple solutions over complex ones
- When proposing changes, explain the trade-offs
