# Claude Code Guidelines for Brewlog

## Project Overview

Brewlog is a self-hosted coffee logging platform built in Rust: HTTP server (Axum + Askama + Datastar), REST API, CLI client, SQLite database.

## Build & Test Commands

```bash
prek run -av                      # All lints, tests, formatters
cargo build                       # Build
cargo test                        # Tests
cargo clippy --allow-dirty --fix  # Lint + auto-fix
nix fmt                           # Format
sqlx migrate add <name>           # New migration → migrations/NNNN_<name>.sql
```

## Workflow Requirements

**Before finishing any task**, always:

1. Run `prek run -av`
1. Consider if the test coverage needs updating
1. Update `README.md` if the change adds/removes/renames CLI commands, env vars, or user-facing features
1. Update `scripts/bootstrap-db.sh` if the change affects CLI commands, flags, or entity fields used by it
1. Provide a **draft commit message** using Conventional Commits format

## Architecture

Clean Architecture / DDD with four layers:

```
src/
├── domain/              # Pure business logic, no external deps
│   ├── errors.rs        # RepositoryError enum
│   ├── ids.rs           # Typed ID wrappers (RoasterId, BagId, BrewId, etc.)
│   ├── listing.rs       # Pagination & sorting (SortKey, ListRequest, Page, PageSize)
│   ├── repositories.rs  # Repository traits
│   ├── countries.rs     # Country name → ISO code, flag emoji
│   ├── formatting.rs    # format_relative_time(), format_weight()
│   ├── coffee/          # roasters, roasts, bags, brews, cups, gear, cafes
│   ├── auth/            # users, sessions, tokens, passkeys, registration_tokens
│   └── analytics/       # timeline, stats, country_stats, ai_usage
├── infrastructure/      # DB, HTTP clients, third-party APIs
│   ├── repositories/    # SQL impls of repository traits (coffee/, auth/, analytics/)
│   ├── client/          # HTTP client for CLI
│   ├── ai.rs            # OpenRouter LLM integration
│   ├── foursquare.rs    # Foursquare Places API
│   ├── backup.rs        # Database backup/restore
│   └── database.rs      # Database pool + SQLite pragmas
├── application/         # HTTP server, routes, middleware, services
│   ├── routes/          # Axum handlers (api/ for REST, app/ for web UI)
│   ├── services/        # Entity services (create + timeline event)
│   └── errors.rs        # HTTP error mapping
└── presentation/        # User interfaces
    ├── cli/             # CLI commands
    └── web/             # View models for templates
```

**Dependency flow**: `presentation → application → domain ← infrastructure`

## Gotchas

These are non-obvious footguns that will cause bugs if missed.

**1. `datastar-fetch` event bubbles through the DOM.** Every `data-on:datastar-fetch` handler must guard with its own in-progress signal, or it fires for events from _any_ `@post`/`@get` in the DOM tree:

```html
<form
  data-on:submit="$_extracting = true; @post(...)"
  data-on:datastar-fetch="if (!$_extracting) return;
    if (evt.detail.type === 'finished') { $_extracting = false }
    else if (evt.detail.type === 'error') { $_extracting = false; $_extractError = 'Failed.' }"
></form>
```

**2. No `data-model` in Datastar v1** — silently ignored. Use `data-bind:_signal-name`.

**3. Signal patching requires JSON, not HTML.** Use `render_signals_json()`, not `data-signals` in DOM fragments.

**4. List partial must be OUTSIDE the form section** — sibling of the form `<section>`, not nested inside it.

**5. Table wrapper must be `<section>`, not `<div>`** — `<section class="rounded-lg border bg-surface">`.

**6. Infinite scroll sentinel needs `md:hidden`** — `<div class="infinite-scroll-sentinel h-4 md:hidden">`.

**7. Use token-based text classes, never `text-stone-*`.** Use `text-text`, `text-text-secondary`, `text-text-muted`.

**8. Static assets need explicit routes and cache headers.** Embedded via `include_str!()`/`include_bytes!()` with explicit routes in `application/routes/app/mod.rs`. All under `/static/`. Every handler must return `cache-control: public, max-age=604800`.

**9. CSP must be updated when adding external resources.** Set in `application/routes/mod.rs`. Datastar requires `'unsafe-inline'` and `'unsafe-eval'` in `script-src`.

**10. Cookie `Secure` flag is on by default.** Set `BREWLOG_INSECURE_COOKIES=true` for local HTTP dev.

**11. URL fields must validate scheme server-side.** Reject non-`http(s)` schemes to prevent XSS. See `is_valid_url_scheme()` in `domain/coffee/roasters.rs`.

**12. Datastar create handlers must check referer for fragment targets.** If a `@post` can fire from pages lacking the target element, check `Referer` and return a reload-script. See `create_brew` in `application/routes/api/coffee/brews.rs`.

## Backend Patterns

### Repository Pattern

Repositories defined as traits in `domain/repositories.rs`, SQL impls in `infrastructure/repositories/`. Each uses a private `Record` struct with `to_domain()`. Use typed ID wrappers from `domain/ids.rs` — never raw `i64`.

### Service Layer

Services (`application/services/`) encapsulate "create + timeline event". Use **services** for `create()` (and `finish()` for bags), **repos** for `get()`/`list()`/`update()`/`delete()`.

`define_simple_service!` macro generates services for `RoasterService`, `CafeService`, `GearService`. Others (`RoastService`, `BagService`, `BrewService`, `CupService`) are hand-written because they need enrichment from related repos.

Timeline events use fire-and-forget: `if let Err(err) = ... { warn!(...) }`.

### Route Module Structure

Each list-bearing route follows: path constants → `load_entity_page()` → `entity_page()` (fragment vs full page via `is_datastar_request()`) → `render_entity_list_fragment()`.

Create handlers use a three-way response pattern:

```rust
if is_datastar_request(&headers) {
    render_fragment(...)   // Datastar → updated list fragment
} else if matches!(source, PayloadSource::Form) {
    Redirect::to(...)      // Browser form → redirect
} else {
    Json(entity)           // API → JSON
}
```

### Detail Pages

Seven detail pages (bag, brew, cafe, cup, gear, roast, roaster) share macros from `templates/partials/detail_cards.html` and helpers from `presentation/web/views/mod.rs` (`build_coffee_info`, `build_roaster_info`, `build_map_data`).

### Macros

All macros have doc comments. Key ones: `define_simple_service!`, `define_get_handler!`, `define_enriched_get_handler!`, `define_delete_handler!`, `define_list_fragment_renderer!`, `define_get_command!`, `define_delete_command!`, `push_update_field!`. Check source files for usage.

### SQL & Queries

Use `QueryBuilder` for dynamic queries, `push_update_field!` for UPDATEs. Sort method is `order_clause()` (not `sort_clause`).

### Stats Cache

Stats are pre-computed in `stats_cache` via a background task with 2-second debouncing (`application/services/stats.rs`). **Every entity create/update/delete handler must call `state.stats_invalidator.invalidate()`** — the `define_delete_handler!` macro does this automatically.

**Adding new stats:**

1. Add the field to the relevant domain struct (`RoastSummaryStats`, `ConsumptionStats`, `BrewingSummaryStats`, or `GeoStats`)
2. Add the query in `SqlStatsRepository`
3. `CachedStats` inherits the change via serde
4. The stats page template can reference the new field immediately

### Error Handling

Error types: `RepositoryError` (domain), `AppError` (HTTP), `anyhow::Result` (CLI). Never silently discard errors — log before `map_err`, avoid bare `.ok()`, use `if let Err` instead of `let _ =`. Every create/update/delete logs at `info!` with entity ID.

### Open Graph

Base URL from `BREWLOG_RP_ORIGIN` via `crate::base_url()`. To add OG tags: add `pub base_url: &'static str` to template struct, override `{% block og_title %}`, `{% block og_description %}`, add og:image in `{% block head %}`.

## Datastar & Frontend

### Core Concepts

Key Datastar attributes:

| Attribute                    | Purpose                                  |
| ---------------------------- | ---------------------------------------- |
| `data-signals:_name="value"` | Declare local signal (underscore prefix) |
| `data-show="$_signal"`       | Conditional visibility                   |
| `data-bind:_signal-name`     | Two-way binding to input                 |
| `data-on:event="expr"`       | Event handler                            |
| `data-text="$_signal"`       | Set text content from signal             |
| `data-attr:attr="$_signal"`  | Set attribute from signal                |
| `@get/@post/@put/@delete`    | HTTP actions with Datastar headers       |

Signal names: **kebab-case** in HTML (`data-signals:_roaster-name`), **camelCase** in JS (`$_roasterName`) and JSON (`_roasterName`).

Two response formats: **HTML fragments** via `render_fragment(template, selector)`, **JSON signal patches** via `render_signals_json(&[("_signal-name", value)])` (pass kebab-case, auto-converts to camelCase).

### Datastar vs JavaScript

**Datastar**: visibility toggling, list CRUD, debounced search, AI extraction, multi-step wizards, searchable selects.
**JavaScript**: browser APIs (WebAuthn, clipboard, geolocation), infinite scroll, theme toggle, flows needing `window.location.reload()`.

### AI Extraction Pattern

Extraction forms use `@post` with `data-on:datastar-fetch` guarded by `_extracting` signal. Server returns `render_signals_json()` which Datastar merges into form fields via `data-bind`. See existing extraction forms for the template pattern.

### Web Components

- **`<brew-photo-capture>`** — camera/file picker, sets data URL on `target-input`, submits `target-form`
- **`<searchable-select>`** — filterable dropdown with `name`, `placeholder`, `change`/`clear` events
- **`<chip-scroll>`** — horizontal scroll with chevron buttons, needs `[data-chip-scroll]`, `[data-scroll-left]`, `[data-scroll-right]`
- **`<world-map>`** — SVG choropleth via `data-countries` (ISO:count pairs), `data-max`, optional `data-selected`
- **`<donut-chart>`** — SVG donut via `data-items` (pipe-separated label:count), `data-icon` ("beaker"/"grinder")

### FlexiblePayload

Handlers accept JSON and form data via `FlexiblePayload<T>`. Use `*Submission` newtypes when form fields don't map 1:1 to domain structs.

## Design System

### CSS Build

Tailwind CSS v4 via standalone CLI. `build.rs` runs it during `cargo build`. Source: `static/css/input.css`. Output: `static/css/styles.css` (gitignored).

### Design Tokens

Defined in `input.css` (`:root` light, `[data-theme="dark"]` dark). Key tokens: `bg-page`, `bg-surface`, `bg-surface-alt`, `bg-accent`, `text-text`, `text-text-secondary`, `text-text-muted`, `text-accent`, `text-accent-text`. Dark mode via `[data-theme="dark"]` on `<html>`.

### Component Classes

Defined in `input.css`: `.input-field`, `.btn-adjust`, `.sticky-submit`, `.pill` + variants (`.pill-muted`, `.pill-success`, `.pill-warning`, `.pill-floral` through `.pill-vegetal`), `.tab`/`.tab-active`, `.tab-mobile`/`.tab-mobile-active`, `.responsive-table`, `.scrollbar-hide`, `.timeline-*`/`.tl-card`, `.text-2xs`, `.small-caps`.

### Key UI Rules

- Cards: `rounded-lg border bg-surface` — no shadows. Clickable cards use `hover:border-accent/40`.
- Typography: page title `text-3xl font-semibold`, section `text-lg font-semibold`, body `text-sm text-text-secondary`, muted `text-xs text-text-muted`. Font weights: `bold` for stats only, `semibold` for headings/primary buttons, `medium` for secondary actions.
- Icons: entity mapping — brew=beaker, roast=coffee_bean, roaster=fire, bag=bag, cup=cup, cafe=location, gear=grinder. Sizes: `h-3 w-3` (inline labels), `h-4 w-4` (buttons), `h-5 w-5` (nav/spinners), `h-6 w-6` (stat cards). Always `shrink-0` in flex.
- Buttons: primary `bg-accent text-accent-text`, outlined `border text-sm font-medium`, card action `h-8 border px-2 text-accent`, link `text-sm font-medium text-accent`. Submit buttons right-aligned.
- Forms: `<label class="flex flex-col gap-1 text-sm">` with `.input-field`. Required fields marked `*`. Multi-column: `grid gap-4 sm:grid-cols-2`.
- Spacing: page sections `gap-8`, form sections `gap-6`, field groups `gap-4`, button groups `gap-2`/`gap-3`, label-to-input `gap-1`.

### Tables & Lists

List partials in `templates/partials/lists/`. Table macros in `table.html`: `search_header()`, `pagination_header()`, `sortable_header()`. Every table has "Added" as first sortable column. Tables use `.responsive-table` (cards on mobile, table on desktop). Desktop: pagination controls. Mobile: infinite scroll via `IntersectionObserver`.

## Code Style

### Rust

- Extract closures >10 lines into named functions
- DRY 3+ similar blocks into helpers
- Prefer `match` over `if/else-if` on same variable
- Extract shared predicates and generic helpers for repeated patterns

### JavaScript

- `const`/`let` only, never `var`
- Arrow functions only, never `function` declarations
- Template literals for interpolation, never `+`
- Inline `onclick` + global arrow functions, not `DOMContentLoaded` + `addEventListener`

### Copy & UI Text

**No second-person pronouns** — never "you"/"your" in user-facing strings. Use imperative or impersonal phrasing.

### Naming & Conventions

- Sort method: `order_clause()` not `sort_clause()`
- SQL: raw strings `r#"..."#` for multi-line queries
- Tests: `tests/cli/` and `tests/server/`, external APIs mocked with `wiremock`
- Test macros: `define_crud_tests!`, `define_datastar_entity_tests!`, `define_cli_auth_test!`, `define_cli_list_test!` — see source files for usage
- Commits: Conventional Commits, never add "Co-Authored-By" trailers, never use `--no-gpg-sign`, never commit unless explicitly prompted
