# Claude Code Guidelines for Brewlog

## Project Overview

Brewlog is a self-hosted coffee logging platform built in Rust. It provides:

- HTTP server with web UI (Axum + Askama templates + Datastar)
- REST API for programmatic access
- CLI client for command-line operations
- SQLite database

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
├── application/      # HTTP server, routes, middleware, services
│   ├── routes/       # Axum route handlers
│   ├── services/     # Entity services (create + timeline orchestration)
│   └── errors.rs     # HTTP error mapping
│
└── presentation/     # User interfaces
    ├── cli/          # CLI commands and argument parsing
    └── web/          # View models for templates
```

**Dependency flow**: `presentation → application → domain ← infrastructure`

## Gotchas

These are non-obvious footguns that will cause bugs if missed.

**1. `datastar-fetch` event bubbles through the DOM.** When a page has multiple forms with `data-on:datastar-fetch` handlers, each handler fires for events from *any* `@post`/`@get` in the same DOM tree. **Every handler must guard with its own in-progress signal**:

```html
<form data-on:submit="$_extracting = true; @post(...)"
  data-on:datastar-fetch="if (!$_extracting) return;
    if (evt.detail.type === 'finished') { $_extracting = false }
    else if (evt.detail.type === 'error') { $_extracting = false; $_extractError = 'Failed.' }">
```

Only reset state on `finished` or `error`, never unconditionally.

**2. No `data-model` in Datastar v1** — it is silently ignored. Use `data-bind:_signal-name` for two-way binding.

**3. Signal patching requires JSON, not HTML.** Datastar only processes signal updates from `application/json` responses (via `render_signals_json()`), not from `data-signals` attributes in DOM-patched HTML fragments.

**4. List partial must be OUTSIDE the form section.** In page templates, the `{% include %}` for the list partial must be a **sibling** of the form `<section>`, not nested inside it. Placing it inside removes the flex gap between form and list.

**5. Table wrapper must be `<section>`, not `<div>`.** List partials wrap the table in `<section class="rounded-lg border bg-surface">`.

**6. Infinite scroll sentinel needs `md:hidden`.** The sentinel `<div class="infinite-scroll-sentinel h-4 md:hidden">` must include `md:hidden` to avoid unwanted height on desktop. Same applies when creating sentinels dynamically in JS.

**7. Use token-based text classes, never hardcoded `text-stone-*`.** Always use `text-text`, `text-text-secondary`, `text-text-muted` which adapt between light and dark themes.

**8. Static assets need explicit routes.** All assets are embedded at compile time via `include_str!()`/`include_bytes!()` with explicit routes in `application/routes/mod.rs`. There is no `tower-http` static file serving.

**9. CSP must be updated when adding external resources.** The `Content-Security-Policy` header is set in `application/routes/mod.rs`. If you add a new external script, stylesheet, font, or image source, update the corresponding CSP directive (`script-src`, `style-src`, `font-src`, `img-src`) or the browser will block it silently. Datastar requires `'unsafe-inline'` and `'unsafe-eval'` in `script-src`.

**10. Cookie `Secure` flag is on by default.** Session cookies are marked `Secure` unless `BREWLOG_INSECURE_COOKIES=true` is set. Local HTTP development needs this env var in `.env`. Do not use the old `BREWLOG_SECURE_COOKIES` variable — it no longer exists.

**11. URL fields must validate scheme server-side.** Any user-supplied URL field (e.g., roaster `homepage`) must reject non-`http(s)` schemes to prevent `javascript:` or `data:` XSS. Use the `is_valid_url_scheme()` helper in `domain/roasters.rs` as a reference pattern.

**12. Datastar create handlers must check referer for fragment targets.** When a `@post` creates an entity and returns a list fragment (e.g., `#brew-list`), that fragment only exists on the entity's data page. If the same `@post` can fire from other pages (homepage, timeline), check the `Referer` header and return a reload-script response for pages that lack the target element. See `create_brew` in `application/routes/api/brews.rs`.

## Backend Patterns

### Repository Pattern

All data access goes through trait-based repositories defined in `domain/repositories.rs`. SQL implementations live in `infrastructure/repositories/`, each using a private `Record` struct with a `to_domain()` method to convert database rows to domain entities. Use typed ID wrappers from `domain/ids.rs` (e.g., `RoastId`, `BagId`) — never raw `i64`.

### Service Layer

Services in `application/services/` encapsulate "create entity + record timeline event" as a single operation.

**When to use services vs repos:**
- **Services** — for `create()` (and `finish()` for bags). These record a timeline event after the insert.
- **Repos** — for `get()`, `list()`, `update()`, `delete()`. No side effects needed.

`AppState` holds both repos and services. Route handlers call `state.xxx_service.create()` for creation and `state.xxx_repo.get()` / `.list()` / etc. for reads and updates.

The `define_simple_service!` macro in `services/mod.rs` generates services for entities whose `to_timeline_event()` needs only `&self`. This covers `RoasterService`, `CafeService`, `GearService`.

Entities needing enrichment are hand-written:

| Service | Extra repos | Why |
|---------|-------------|-----|
| `RoastService` | `roaster_repo` | Needs roaster name/slug for timeline |
| `BagService` | `roast_repo`, `roaster_repo` | `create()` + `finish()`, needs roast+roaster for timeline |
| `BrewService` | — | `create()` enriches via `get_with_details()` for timeline + response |
| `CupService` | — | `create()` enriches via `get_with_details()` for timeline |

Timeline events are display-only (not data integrity), so they use fire-and-forget error handling:

```rust
if let Err(err) = self.timeline_repo.insert(entity.to_timeline_event()).await {
    warn!(error = %err, id = %entity.id, "failed to record timeline event");
}
```

### Route Module Structure

Each list-bearing route module (roasters, roasts, bags, gear, brews) follows the same structure:

1. **Path constants** — `ENTITY_PAGE_PATH` (full page URL) and `ENTITY_FRAGMENT_PATH` (with `#entity-list` anchor)
2. **`load_entity_page()`** — calls `repo.list()` and builds view models via `build_page_view()` from `support.rs`
3. **`entity_page()`** — checks `is_datastar_request()`: returns fragment for Datastar, full page otherwise
4. **`render_entity_list_fragment()`** — returns just the list partial for Datastar replacement

Create handlers follow a three-way response pattern:

```rust
if is_datastar_request(&headers) {
    render_fragment(...)   // Datastar → updated list fragment
} else if matches!(source, PayloadSource::Form) {
    Redirect::to(...)      // Browser form → redirect
} else {
    Json(entity)           // API → JSON
}
```

### Macros Reference

All macros have doc comments with usage examples. Check the source files for full documentation.

| Macro | Location | Purpose |
|-------|----------|---------|
| `define_simple_service!` | `application/services/mod.rs` | Generate service struct with `create()` + timeline |
| `define_get_handler!` | `application/routes/api/macros.rs` | GET `/api/v1/:entity/:id` → JSON |
| `define_enriched_get_handler!` | `application/routes/api/macros.rs` | GET with joined related entities → JSON |
| `define_delete_handler!` | `application/routes/api/macros.rs` | DELETE → fragment for Datastar or 204 for API |
| `define_list_fragment_renderer!` | `application/routes/api/macros.rs` | Generate fragment renderer for a list page |
| `define_get_command!` | `presentation/cli/macros.rs` | CLI get-entity command |
| `define_delete_command!` | `presentation/cli/macros.rs` | CLI delete-entity command |
| `push_update_field!` | `infrastructure/repositories/macros.rs` | Build dynamic UPDATE queries with `QueryBuilder` |

### SQL & Queries

Use `QueryBuilder` for dynamic queries. For UPDATE, use `push_update_field!` (see macro docs). Each repository has an `order_clause()` method for sort query generation — use `order_clause` as the method name, not `sort_clause`.

### Error Handling & Logging

**Error types**: `RepositoryError` (domain), `AppError` (HTTP with status code mapping), `anyhow::Result` (CLI).

**Logging**: `tracing` + `tracing-subscriber` with `tower-http` `TraceLayer`. Configure via `RUST_LOG` (default `info`) and `RUST_LOG_FORMAT=json` for structured output.

**Error logging rules** — never silently discard errors:

1. **`map_err(|_| StatusCode::*)` patterns** — log the original error before mapping. Use `warn!` for client-caused failures, `error!` for server-side.
2. **`.ok()` / `.ok()?` patterns** — replace with explicit match that logs before returning `None`.
3. **Fire-and-forget (`let _ = ...`)** — use `if let Err(err) = ...` and log.
4. **Background tasks (`tokio::spawn`)** — log inside the spawned future.

**CRUD logging**: Every successful create/update/delete logs at `info!` with entity ID and key fields.

**Security logging**: Auth events (login, logout, token create/revoke, passkey delete) log at `info!` with user ID.

### Foursquare Integration

Nearby cafe search uses the Foursquare Places API. Set `BREWLOG_FOURSQUARE_API_KEY`. See `infrastructure/foursquare.rs` for the implementation and `tests/server/nearby_api.rs` for the `wiremock`-based test pattern.

## Datastar & Frontend

### Core Concepts

The web UI uses [Datastar](https://data-star.dev/) for reactive updates without full page reloads.

Key attributes:

| Attribute | Purpose | Example |
|-----------|---------|---------|
| `data-signals:_name="value"` | Declare local signal (underscore = not sent to server) | `data-signals:_show-form="false"` |
| `data-show="$_signal"` | Conditional visibility | `data-show="$_showForm"` |
| `data-bind:_signal-name` | Two-way binding to input value | `data-bind:_roaster-name` |
| `data-on:event="expr"` | Event handler | `data-on:submit="$_submitting = true; @post(...)"` |
| `data-ref="_name"` | DOM element reference | `data-ref="_form"` |
| `data-text="$_signal"` | Set text content from signal | `data-text="$_cafeName"` |
| `data-attr:attr="$_signal"` | Set attribute from signal | `data-attr:value="$_roastId"` |
| `@get/@post/@put/@delete` | HTTP actions with Datastar headers | `@post('/api/v1/roasters', {contentType: 'form'})` |

### Signal Naming

Signal names use **kebab-case** in HTML attributes and auto-convert to **camelCase** in JS expressions and JSON:

- HTML: `data-signals:_roaster-name="''"` or `data-bind:_roaster-name`
- JS expression: `$_roasterName`
- JSON key: `_roasterName`

**Naming conventions for common signals:**

| Signal | Purpose |
|--------|---------|
| `_extracting` | AI extraction in progress |
| `_submitting` | Form save/create in progress |
| `_extract-error` / `_error` | Error message |
| `_show-{thing}` | Boolean visibility toggle |

### Response Types

Two response formats exist for Datastar:

**HTML fragments** — for replacing DOM sections. Use `render_fragment(template, selector)` from `support.rs`, which sets `datastar-selector` and `datastar-mode: replace` headers.

**JSON signal patches** — for updating signal values (e.g., AI extraction filling form fields). Use `render_signals_json(&[("_signal-name", value)])` from `support.rs`. Signal keys are passed in kebab-case; the function converts to camelCase for the JSON response.

### Datastar vs JavaScript

**Use Datastar for:**
- Visibility toggling (`data-show` + signals)
- List CRUD — delete with `confirm() && @delete()`, create with `@post()` + fragment re-render
- Debounced search — `data-on:input__debounce.300ms` + `@get()` with `responseOverrides`
- AI extraction signal patching
- Multi-step wizards — step signals (`$_step`) with `data-show="$_step === N"`
- Searchable selection lists — `<searchable-select>` with `data-on:change`

**Use JavaScript for:**
- Browser APIs: WebAuthn, clipboard, geolocation, FileReader
- Infinite scroll (`IntersectionObserver` in `base.html`)
- Theme toggle — must run in `<head>` before DOM renders
- Any flow requiring `window.location.reload()` after completion

### AI Extraction Pattern

Pages with AI-powered form filling use a Datastar-native pattern. Extraction endpoints return JSON signal patches that Datastar merges into the signal store, and `data-bind` pushes values into form fields automatically.

Template structure:

```html
<section data-signals:_extracting="false" data-signals:_extract-error="''" data-signals:_submitting="false">
  <!-- Extraction form -->
  <form id="{id}-extract-form"
    data-on:submit="$_extracting = true; $_extractError = ''; @post('{endpoint}', {contentType: 'form'})"
    data-on:datastar-fetch="if (!$_extracting) return;
      if (evt.detail.type === 'finished') { $_extracting = false }
      else if (evt.detail.type === 'error') { $_extracting = false; $_extractError = 'Extraction failed.' }">
    <input type="hidden" name="image" id="{id}-image" />
    <div data-show="!$_extracting">
      <brew-photo-capture target-input="{id}-image" target-form="{id}-extract-form" class="...">
        Take Photo
      </brew-photo-capture>
      <input name="prompt" type="text" placeholder="Or describe..." />
      <button type="submit">Go</button>
    </div>
    <div data-show="$_extracting" style="display:none"><!-- spinner --></div>
    <p data-show="$_extractError" data-text="$_extractError" style="display:none" class="text-sm text-red-600"></p>
  </form>

  <!-- Main form with data-bind fields populated by extraction -->
  <form data-on:submit="$_submitting = true; @post(...)">
    <input name="name" data-bind:_roaster-name class="input-field" />
  </form>
</section>
```

Server-side handler:

```rust
let signals = vec![
    ("_roaster-name", Value::String(result.name)),
    ("_roaster-country", Value::String(result.country)),
];
render_signals_json(&signals)
```

### Web Components

**`<brew-photo-capture>`** (`static/js/components/photo-capture.js`):

| Attribute | Purpose |
|-----------|---------|
| `target-input` | ID of hidden `<input>` that receives the data URL |
| `target-form` | ID of `<form>` to submit after reading the photo |

Clicking the element opens the camera/file picker, reads the file as a data URL, sets the target input, and submits the form.

**`<searchable-select>`** (`static/js/components/searchable-select.js`):

| Attribute | Purpose |
|-----------|---------|
| `name` | Name for hidden `<input>` in form submission |
| `placeholder` | Search input placeholder (default: "Type to search...") |

| Event | Detail |
|-------|--------|
| `change` | `{ value, display, data }` — fires on selection |
| `clear` | Fires when selection is cleared |

Place `<button>` children with `value` and `data-display` attributes. The component handles search filtering, hidden input, and selected-value display internally.

### FlexiblePayload

Handlers accept both JSON and form data via `FlexiblePayload<T>`. When form fields don't map 1:1 to the domain `New*` struct (e.g., a roaster name needing resolution to an ID), use a `*Submission` newtype that handles the conversion.

## Design System

### CSS Build

**Tailwind CSS v4** built via standalone CLI (no Node.js). `build.rs` runs `tailwindcss` automatically during `cargo build` — no manual step needed. Source: `static/css/input.css`. Output: `static/css/styles.css` (gitignored).

### Design Tokens

Colors are defined as CSS custom properties in `static/css/input.css` (`:root` for light, `[data-theme="dark"]` for dark) and mapped to Tailwind utilities via `@theme`. See `input.css` for exact colour values.

| Token | Tailwind classes |
|-------|-----------------|
| `--page` | `bg-page` |
| `--surface` | `bg-surface` |
| `--surface-alt` | `bg-surface-alt` |
| `--border` | default `border` / `divide-y` |
| `--accent` | `bg-accent`, `text-accent` |
| `--accent-hover` | `bg-accent-hover`, `hover:bg-accent-hover` |
| `--accent-subtle` | `bg-accent-subtle` |
| `--accent-text` | `text-accent-text` |
| `--text` | `text-text` |
| `--text-secondary` | `text-text-secondary` |
| `--text-muted` | `text-text-muted` |

Dark mode uses `[data-theme="dark"]` on `<html>`, set via a `<script>` in `<head>` that reads `localStorage` / `prefers-color-scheme` before body render. The `dark:` Tailwind prefix is available as an escape hatch.

### Component Classes

Defined in `input.css` — use these instead of ad-hoc utility combinations:

| Class | Use for |
|-------|---------|
| `.input-field` | All text/number/select inputs (border, padding, rounded, focus ring) |
| `.btn-adjust` | +/- stepper buttons flanking a numeric input |
| `.pill.pill-accent` | Tasting notes, highlighted tags |
| `.pill.pill-muted` | Categories, kind labels, neutral status |
| `.pill.pill-success` | Positive status ("Open") |

Always include `.pill` base class with the variant. Do not override pill colour/border/padding with utilities.

Cards use `rounded-lg border bg-surface p-4` — no shadows.

### UI Patterns

#### Typography

| Level | Classes | Use |
|-------|---------|-----|
| Page title | `text-3xl font-semibold` | `<h1>` on each page |
| Section title | `text-lg font-semibold text-text` | `<h2>` / `<h3>` for cards |
| Subsection title | `text-base font-semibold text-text` | `<h3>` within cards |
| Form section label | `text-xs font-semibold text-text-muted uppercase tracking-wide` | `<h4>` grouping fields |
| Body | `text-sm text-text-secondary` | Description text |
| Muted | `text-xs text-text-muted` | Metadata, subtext |
| Accent title | `text-2xl font-semibold text-accent` | Login/register headers |

Page headers: `<header class="flex flex-col gap-2">` with `<h1>` + `<p class="max-w-2xl text-sm text-text-secondary">`.

#### Buttons

| Variant | Classes | Use |
|---------|---------|-----|
| Primary | `rounded-md bg-accent px-4 py-2 text-sm font-semibold text-accent-text transition hover:bg-accent-hover` | Save, Submit, Check In |
| Secondary | `rounded-md border px-4 py-2 text-sm font-semibold text-text transition hover:border-accent hover:text-text` | Cancel, Back |
| Text-only | `text-xs text-text-muted hover:text-text` | Change, Back in summary bars |
| Icon-only | `inline-flex h-8 w-8 items-center justify-center rounded-md p-1 text-text-muted transition hover:text-red-600` | Delete, revoke |
| Link-style | `inline-flex items-center gap-1 text-sm font-medium text-accent hover:text-accent-hover` | Brew Again, View all |
| Adjustment | `.btn-adjust` CSS class | +/- steppers |

Submit buttons are right-aligned: `<div class="flex items-center justify-end gap-2">`. Secondary comes first when paired.

Use `w-full` for full-width CTAs, `py-3` for larger touch targets, `disabled:opacity-50` with `data-attr:disabled` for loading states.

#### Forms

Label + input pattern:

```html
<label class="flex flex-col gap-1 text-sm">
  <span class="text-text">Field Name *</span>
  <input type="text" name="field" required class="input-field" placeholder="Example" />
</label>
```

Mark required fields with `*`. Multi-column: `<div class="grid gap-4 sm:grid-cols-2">`. Form card wrapper: `<div class="rounded-lg border bg-surface p-5">` with `flex flex-col gap-4` (or `gap-6` with section headers).

Checkbox: `<label class="inline-flex items-center gap-2 text-sm cursor-pointer">` with `<input class="accent-orange-700" />`.

#### Feedback States

| Variant | Classes |
|---------|---------|
| Error text | `text-sm text-red-600` with `data-show`/`data-text` bound to error signal |
| Error alert | `rounded-md bg-red-100 border border-red-300 p-3 text-sm text-red-800` |
| Success alert | `rounded-md bg-green-100 border border-green-300 p-4 text-sm text-green-800` |
| Warning alert | `rounded-md bg-yellow-100 border border-yellow-300 p-3 text-sm text-yellow-800` |
| Info alert | `rounded-md bg-green-50 border border-green-200 px-3 py-2 text-sm text-green-800` |
| Loading spinner | `flex items-center gap-3 text-sm text-accent` with `{% call icons::spinner("h-5 w-5") %}` + `Saving&hellip;` |

Always pair loading spinners with `data-show` bound to an in-progress signal.

#### Empty & Prerequisite States

When a form requires a parent entity that doesn't exist yet, show a static message with `text-lg font-semibold text-text` heading and `text-sm text-text-secondary` body.

#### Selected Item Indicator

In multi-step flows, show: `rounded-lg border bg-surface px-4 py-3 flex items-center justify-between` — icon + name + muted subtext on left, text-only "Change" button on right.

#### Navigation

Active state: `text-accent font-medium`. Inactive: `text-text-muted hover:text-text transition`. Desktop nav: `hidden md:flex items-center gap-6`. Mobile nav: toggled via `data-show`.

#### Responsive Patterns

- Desktop-only: `hidden md:flex` or `hidden md:block`
- Mobile-only: `md:hidden`
- Pagination: `hidden md:flex` (desktop); infinite scroll (mobile)

#### Spacing

| Context | Gap |
|---------|-----|
| Major form sections | `gap-6` |
| Related field groups | `gap-4` |
| Label to input | `gap-1` |
| Icon + text pairs | `gap-2` |
| Navigation links | `gap-6` |
| Button groups | `gap-2` |
| Page sections | `gap-8` |

## Tables & Lists

### Page Template Structure

Each list page has form and list as **separate siblings** under `<main>` (which uses `flex flex-col gap-6`):

```html
<section data-signals:_show-form="false">
  <header>...</header>
  {% if is_authenticated %}
  <div data-show="$_showForm" style="display: none"><!-- Form --></div>
  {% endif %}
</section>

{% include "partials/lists/{entity}_list.html" %}
{% endblock %}
```

### List Partial Structure

List partials live in `templates/partials/lists/`. Each follows:

```html
{% import "partials/lists/table.html" as table %}

<div id="{entity}-list" class="mt-6" data-star-scope="{entity}">
  {% if items.is_empty() && !navigator.has_search() %}
  <div class="rounded-lg border border-dashed px-4 py-6 text-sm text-text-secondary">
    <p class="text-center">No {entities} recorded yet.</p>
  </div>
  {% else %}
  <section class="rounded-lg border bg-surface" ...>
    {% call table::search_header(navigator, "#{entity}-list") %}
    <table class="responsive-table ...">...</table>
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

**Exception**: the bags partial uses a dual-section layout (open-bag cards + history table).

### Table Macros

Three macros in `templates/partials/lists/table.html`:

- **`search_header(navigator, target_selector)`** — search input with debounced `@get` + `history.pushState`
- **`pagination_header(items, navigator, target_selector)`** — prev/next, page count, rows-per-page; hidden on mobile via `pagination-controls hidden md:flex`
- **`sortable_header(label, key, navigator, target_selector)`** — clickable column header with sort arrows

### "Added" Column

Every table has a sortable "Added" column as its **first column** (sorted by `created-at`), styled with `text-xs font-medium text-text-secondary` to visually separate from content columns.

### Actions Column

When a row has multiple action buttons, wrap them in `<div class="inline-flex items-center gap-1">` inside the `<td>` to prevent vertical stacking.

### Responsive Table Pattern

Tables use the `responsive-table` CSS class (card layout on mobile, standard table on desktop).

**Desktop** — combine related fields with subtext:

```html
<td data-label="Coffee" class="px-4 py-3 whitespace-nowrap">
  <div class="font-medium">{{ brew.roast_name }}</div>
  <div class="hidden md:block text-xs text-text-muted">{{ brew.roaster_name }}</div>
</td>
```

**Mobile** — separate `<td>` for each sub-field:

```html
<td data-label="Roaster" class="px-4 py-3 whitespace-nowrap md:hidden">
  {{ brew.roaster_name }}
</td>
```

### Pagination & Infinite Scroll

- **Desktop** (`md:+`): pagination controls via `pagination_header` macro
- **Mobile** (below `md:`): infinite scroll via `IntersectionObserver` in `base.html`, activated only on mobile via `matchMedia("(max-width: 767px)")`

### Search

Server-side via `q` query parameter. `ListQuery` extracts it, repos apply `LIKE` filtering. `ListNavigator` preserves the search term across pagination and sort URLs.

`ListNavigator` URL helpers: `page_href()` (full page), `fragment_page_href()` (with anchor), `sort_href()`.

## Code Style

### Rust

**Extract large closures** — if a closure in `.filter_map()`, `.map()`, etc. exceeds ~10 lines, extract into a named function.

**DRY repeated blocks** — when 3+ blocks follow the same structure differing only by a parameter, extract a helper.

**Prefer `match` over `if/else-if`** when branching on the same variable.

**Extract shared predicates** — if the same boolean condition appears in multiple functions, make it a named helper.

**Use generic helpers** — when the same decode/encode/match pattern appears 3+ times with different types.

### JavaScript

- **Never use `var`** — always `const` (default) or `let` (when reassignment needed)
- **Never use `function` declarations** — always arrow functions: `const fn = () => { ... }`
- **Always use template literals** for interpolation — never `+` concatenation
- Prefer `if`/`else` and `switch` over ternaries; never nest ternaries
- **Inline `onclick` + global arrow functions** for pages with imperative JS — do not use `DOMContentLoaded` + `addEventListener`

### Naming & Conventions

- **Method naming**: `order_clause()` for sort query builders (not `sort_clause`)
- **Imports**: Group by `super::`, then `crate::`, macros imported explicitly
- **SQL strings**: Use raw strings `r#"..."#` for multi-line queries
- **Tests**: Integration tests in `tests/cli/` and `tests/server/`. External APIs mocked with `wiremock`. See [Test Macros](#test-macros) below
- **Commits**: Conventional Commit format (`feat:`, `fix:`, `refactor:`, etc.)
- **Commit authorship**: Never add "Co-Authored-By" trailers
- **Commit signing**: Never use `--no-gpg-sign` — always allow default GPG signing
- **Committing**: Never commit unless explicitly prompted — provide a draft commit message instead

### Test Macros

Repeated test patterns are generated via macros in `tests/server/test_macros.rs` and `tests/cli/test_macros.rs`. Use these instead of hand-writing boilerplate tests.

**Server API tests** (`tests/server/test_macros.rs`):

`define_crud_tests!` — generates nonexistent-GET-404, nonexistent-DELETE-404, empty-list-200, and optionally malformed-JSON-400 and missing-fields-400:

```rust
use crate::test_macros::define_crud_tests;
define_crud_tests!(
    entity: roaster, path: "/roasters", list_type: Roaster,
    malformed_json: r#"{"name": "Test", "country": }"#,
    missing_fields: r#"{"name": "Test Roasters"}"#
);
```

`define_datastar_entity_tests!` — generates list-with-fragment, list-without-full-page, and delete-with-fragment tests. Requires a setup function that creates an entity and returns its ID as `String`:

```rust
use crate::test_macros::define_datastar_entity_tests;
define_datastar_entity_tests!(
    entity: roasters, type_param: "roasters", api_path: "/roasters",
    list_element: r#"id="roaster-list""#, selector: "#roaster-list",
    setup: create_roaster_entity
);
```

**CLI tests** (`tests/cli/test_macros.rs`):

`define_cli_auth_test!` — asserts a command fails without a token:

```rust
define_cli_auth_test!(test_add_roaster_requires_authentication,
    &["roaster", "add", "--name", "Test", "--country", "UK"]);
```

`define_cli_list_test!` — asserts a list command succeeds without auth and returns a JSON array:

```rust
define_cli_list_test!(test_list_roasters_works_without_authentication,
    &["roaster", "list"]);
```

**Helper generics**: `create_entity<P, R>()` in `tests/server/helpers.rs` (POST + auth + deserialize) and `create_entity_cli()` in `tests/cli/helpers.rs` (run + assert + parse ID) eliminate duplication in entity creation helpers.

## Communication Style

- Be direct and factual
- Analyse root causes before proposing solutions
- Prefer simple solutions over complex ones
- When proposing changes, explain the trade-offs
