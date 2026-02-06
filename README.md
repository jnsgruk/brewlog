# B{rew}log

**B{rew}log** is a self-hosted coffee logging platform for tracking roasters, roasts, brews,
cafes and brewing gear. It ships as a single Rust binary that serves a web UI, a REST API, and a
CLI client.

- **Web UI** with reactive updates via [Datastar](https://data-star.dev/) and [Tailwind CSS v4](https://tailwindcss.com/)
- **REST API** for programmatic access (reads are public, writes require auth)
- **CLI client** for terminal-based workflows
- **AI extraction** — scan a coffee bag label or type a description to auto-fill forms (via [OpenRouter](https://openrouter.ai/))
- **Nearby cafe search** powered by [Foursquare Places](https://docs.foursquare.com/developer/reference/place-search)
- **SQLite** database
- **Passkey authentication** — no passwords, WebAuthn only

## Quick Start

### Install

```bash
cargo install --git https://github.com/jnsgruk/brewlog.git

```

### Configure

Create a `.env` file (or export the variables). Four values are required:

```bash
BREWLOG_RP_ID="localhost"
BREWLOG_RP_ORIGIN="http://localhost:3000"
BREWLOG_OPENROUTER_API_KEY="sk-or-..."
BREWLOG_FOURSQUARE_API_KEY="fsq3..."
```

### Run

```bash
brewlog serve
```

On first start with an empty database the server prints a one-time registration URL:

```
No users found. Register the first user at:
  http://localhost:3000/register/abc123...
This link expires in 1 hour.
```

Open that URL, choose a display name, and register a passkey. This creates your account and
signs you in.

### CLI Authentication

To use the CLI or API for write operations, create a token via browser handoff:

```bash
brewlog token create --name "my-cli-token"
# Browser opens → authenticate with your passkey → token printed once

export BREWLOG_URL="http://localhost:3000"
export BREWLOG_TOKEN="<token from above>"

# Now you can create data from the CLI
brewlog roaster add --name "Radical Roasters" --country "United Kingdom"
```

Run `brewlog --help` for the full command reference.

## Configuration

All settings are read from environment variables or CLI flags. A `.env` file in the working
directory is loaded automatically via [dotenvy](https://crates.io/crates/dotenvy).

### Server (`brewlog serve`)

| Variable                 | Purpose                                                | Default               |
| ------------------------ | ------------------------------------------------------ | --------------------- |
| `BREWLOG_RP_ID`          | WebAuthn Relying Party ID (your domain)                | **required**          |
| `BREWLOG_RP_ORIGIN`      | WebAuthn Relying Party origin (full URL)               | **required**          |
| `BREWLOG_DATABASE_URL`   | Database connection string                             | `sqlite://brewlog.db` |
| `BREWLOG_BIND_ADDRESS`   | Server bind address                                    | `127.0.0.1:3000`      |
| `BREWLOG_INSECURE_COOKIES` | Disable the `Secure` cookie flag (set `true` for local dev over HTTP) | `false` |
| `RUST_LOG`               | Log level filter                                       | `info`                |
| `RUST_LOG_FORMAT`        | Set to `json` for structured log output                | —                     |

### CLI Client

| Variable        | Purpose                               | Default                 |
| --------------- | ------------------------------------- | ----------------------- |
| `BREWLOG_URL`   | Server URL                            | `http://localhost:3000` |
| `BREWLOG_TOKEN` | API bearer token for write operations | —                       |

### Integrations

| Variable                     | Purpose                                                                     | Default           |
| ---------------------------- | --------------------------------------------------------------------------- | ----------------- |
| `BREWLOG_OPENROUTER_API_KEY` | [OpenRouter](https://openrouter.ai/) API key for AI extraction              | **required**      |
| `BREWLOG_OPENROUTER_MODEL`   | LLM model for AI extraction                                                 | `openrouter/free` |
| `BREWLOG_FOURSQUARE_API_KEY` | [Foursquare](https://foursquare.com/) Places API key for nearby cafe search | **required**      |

### Database

SQLite is used for storage. Migrations run automatically on server startup.

## Running with Docker

The Nix flake includes a Docker image built with `dockerTools`:

```bash
nix build .#brewlog-container
docker load < result
```

Create a `docker.env` file:

```env
BREWLOG_OPENROUTER_API_KEY=sk-or-...
BREWLOG_OPENROUTER_MODEL=google/gemini-3-flash-preview
BREWLOG_FOURSQUARE_API_KEY=fsq3...
BREWLOG_RP_ID=localhost
BREWLOG_BIND_ADDRESS=0.0.0.0:3000
BREWLOG_RP_ORIGIN=http://localhost:4000
BREWLOG_INSECURE_COOKIES=true
```

```bash
mkdir data
docker run --rm -p 4000:3000 --env-file docker.env -v $PWD/data:/data brewlog:0.1.0
```

The database is stored at `/data/brewlog.db` inside the container — mount a host directory to `/data` to persist it.

## Contributing

```bash
cargo build                           # Build
cargo clippy --allow-dirty --fix      # Lint
cargo fmt                             # Format
cargo test                            # Test
```

See [CLAUDE.md](CLAUDE.md) for architecture, code patterns, and development conventions.

## License

[Apache License 2.0](LICENSE)
