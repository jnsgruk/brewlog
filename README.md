# B{rew}log

**B{rew}log** is a self-hosted coffee logging platform for tracking your roasters, roasts, brews,
cafes and brewing gear.

The application is distributed as a single Rust binary that powers both an HTTP server and a
command-line client for the API. There is a web frontend built with [Tailwind CSS](https://tailwindcss.com/)
that enables client-side reactivity with [Datastar](https://data-star.dev/).

<!-- prettier-ignore-start -->
> [!NOTE]
> This project was built with significant assistance from Github Copilot. I used it as a test-bed
> for trying out newer agentic coding workflows, and to get some basic experience with Datastar,
> which had attracted my attention.
<!-- prettier-ignore-end -->

## Basic usage

B{rew}log ships as one executable. You decide whether it acts as a server or a client.

### First-time setup

The server requires `BREWLOG_OPENROUTER_API_KEY` and `BREWLOG_FOURSQUARE_API_KEY` to be set. On first start, you must also set an admin username and password:

```bash
BREWLOG_ADMIN_USERNAME="admin" \
BREWLOG_ADMIN_PASSWORD="your-secure-password" \
BREWLOG_OPENROUTER_API_KEY="sk-or-..." \
BREWLOG_FOURSQUARE_API_KEY="fsq3..." \
brewlog serve
```

This creates the admin user in the database. On subsequent starts, the admin environment variables are not required.

### Authentication

Brewlog supports two authentication methods:

1. **Web Frontend**: Session-based authentication via login page
2. **CLI/API**: Token-based authentication via Bearer tokens

#### Web Authentication

1. Start the server and browse to the frontend
2. Click "Login" in the navigation bar
3. Sign in with username `admin` and your password
4. You're now authenticated and can create/update/delete records

#### CLI/API Authentication

First, create an API token:

```bash
brewlog token create --name "my-cli-token"
# You will be prompted for username and password.
# Alternatively, you can provide them via flags:
# brewlog token create --name "my-cli-token" --username admin --password secret

# Username: admin
# Password: ********
#
# Token created successfully!
# Token ID: nye9BDqnLL
# Token Name: my-cli-token
#
# ⚠  Save this token securely - it will not be shown again:
#
# dEadB3efDeadb33fdeadb33F...
#
# Export it in your environment:
#   export BREWLOG_TOKEN=dEadB3efDeadb33fdeadb33F...
```

Export the token and use it for all CLI commands:

```bash
export BREWLOG_TOKEN="dEadB3efDeadb33fdeadb33F..."
export BREWLOG_URL=http://localhost:3000

# Now all write operations work
brewlog roaster add \
  --name "Radical Roasters" \
  --country "United Kingdom" \
  --city "Bristol" \
  --homepage "https://radicalroasters.co.uk"

brewlog roast add \
  --roaster-id "deadbeef" \
  --name "Chelbesa Lot 2" \
  --origin "Ethiopia" \
  --region "Gedeo" \
  --producer "Chelbesa Cooperative" \
  --process "Washed" \
  --tasting-notes "Blueberry, Jasmine"
```

#### Token Management

```bash
# List your active tokens
brewlog token list

# Revoke a token
brewlog token revoke --id abc123
```

#### API Usage

For direct API access, include your token as a Bearer token:

```bash
curl http://localhost:3000/api/v1/roasters \
  -H "Authorization: Bearer dEadB3efDeadb33fdeadb33F..." \
  --json '{"name":"Radical Roasters","country":"United Kingdom"}'
```

**Note**: All read operations (GET requests) are public and don't require authentication. Only write operations (POST/PUT/DELETE) require authentication.

## CLI Commands

The CLI uses a subcommand structure. Each entity command supports `add`, `list`, `get`, `update`, and `delete` subcommands (except where noted):

```
brewlog serve              Run the HTTP server
brewlog roaster <cmd>      Manage roasters
brewlog roast <cmd>        Manage roasts
brewlog bag <cmd>          Manage bags of coffee
brewlog gear <cmd>         Manage brewing gear (grinders, brewers, filter papers)
brewlog brew <cmd>         Manage brews (add, list, get, delete — no update)
brewlog cafe <cmd>         Manage cafes
brewlog cup <cmd>          Manage cups (cafe visits with ratings)
brewlog token <cmd>        Manage API tokens (create, list, revoke)
brewlog backup             Export all data to JSON on stdout
brewlog restore --file F   Restore data from a JSON backup into an empty database
```

Use `brewlog <command> --help` for detailed options on any command.

## Environment Variables

All configuration is via environment variables or CLI flags. A `.env` file in the working directory is loaded automatically at startup (via [dotenvy](https://crates.io/crates/dotenvy)).

### Server (`brewlog serve`)

| Variable | Purpose | Default |
|----------|---------|---------|
| `BREWLOG_DATABASE_URL` | Database connection string | `sqlite://brewlog.db` |
| `BREWLOG_BIND_ADDRESS` | Server bind address | `127.0.0.1:3000` |
| `BREWLOG_ADMIN_USERNAME` | Initial admin username | — (required on first run) |
| `BREWLOG_ADMIN_PASSWORD` | Initial admin password | — (required on first run) |
| `BREWLOG_SECURE_COOKIES` | Set to `true` to enable the Secure cookie flag (for HTTPS) | `false` |
| `RUST_LOG` | Log level filter | `info` |

### CLI Client

| Variable | Purpose | Default |
|----------|---------|---------|
| `BREWLOG_URL` | Server URL for CLI commands | `http://127.0.0.1:3000` |
| `BREWLOG_TOKEN` | API token for authenticated CLI operations | — |

### Integrations

| Variable | Purpose | Default |
|----------|---------|---------|
| `BREWLOG_OPENROUTER_API_KEY` | [OpenRouter](https://openrouter.ai/) API key for AI extraction | — (required) |
| `BREWLOG_OPENROUTER_MODEL` | LLM model for AI extraction | `openrouter/free` |
| `BREWLOG_FOURSQUARE_API_KEY` | [Foursquare](https://foursquare.com/) Places API key for nearby cafe search | — (required) |

## Integrations

### AI Extraction

The web UI uses an LLM via [OpenRouter](https://openrouter.ai/) to extract roaster and roast details from photos or text descriptions. `BREWLOG_OPENROUTER_API_KEY` is required. It powers:

- Photo extraction buttons on the roaster and roast forms
- Text-based extraction from typed descriptions
- The **Scan Bag** feature on the home page, which extracts both roaster and roast data from a single coffee bag label photo
- The **Scan Bag** feature on the check-in page, which identifies a roast from a bag photo

### Nearby Cafe Search

The check-in and cafes pages search for nearby coffee shops via the [Foursquare Places API](https://docs.foursquare.com/developer/reference/place-search). `BREWLOG_FOURSQUARE_API_KEY` is required. Searches can be made by GPS coordinates or city name.

## Database

SQLite is the default database. PostgreSQL is supported via a compile-time feature flag:

```bash
# SQLite (default)
cargo build --release

# PostgreSQL
cargo build --release --features postgres --no-default-features
```

Migrations run automatically on server startup.

### Backup & Restore

```bash
# Export all data to JSON
brewlog backup > backup.json

# Restore into an empty database
brewlog restore --file backup.json
```

Both commands accept `--database-url` (or `BREWLOG_DATABASE_URL`) to target a specific database.

## Installation

At present, the only way to use `brewlog` is to build it from source:

```bash
git clone https://github.com/jnsgruk/brewlog.git
cd brewlog
cargo build --release
```

The resulting binary lives at `target/release/brewlog`.

During development you can run directly:

```bash
cargo run -- serve
```

## Testing

The project includes unit and integration tests:

```bash
cargo test
```
