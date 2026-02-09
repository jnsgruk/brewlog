![cover image](./static/og-image.png)

**B{rew}log** is a self-hosted specialty coffee logging platform optimised for filter brewing
enthusiasts. B{rew}log can be used for tracking roasters, roasts, brews, cafes and brewing gear.

B{rew}log features an LLM-powered "Bag Scanning" feature, which enables it to automatically fill
roaster and coffee information using a photo of a bag. It also supports "check-ins" to log coffee
enjoyed in a cafe.

B{rew}log ships as a single Rust binary that serves a web UI, a REST API, and a CLI client. The
application uses SQLite as a backend, and will automatically create and migrate the database on
start-up.

## Quick Start (Demo)

Before you start, you'll need to sign up for [Openrouter](https://openrouter.ai) and
[Foursquare Places](https://foursquare.com/products/places/) and get API keys for both.

Then create a `docker.env` file:

```env
# You'll need an API key from OpenRouter
BREWLOG_OPENROUTER_API_KEY=sk-or-...
# I've had good results with Gemini models, but you can try 'openrouter/free' to experiment
BREWLOG_OPENROUTER_MODEL=google/gemini-3-flash-preview
# FourSquare Places API key for location searching
BREWLOG_FOURSQUARE_API_KEY=fsq3...
```

You can see the full list of configuration options [below](#configuration). Once your `.env` file
is complete, start the container using the environment file

```bash
# Create a data directory to store the database
mkdir data
# Run the container
docker run \
  --rm \
  -p 3000 \
  --env-file docker.env \
  -v $PWD/data:/data \
  ghcr.io/jnsgruk/brewlog:latest
```

On first start with an empty database the server prints a one-time registration URL:

```
No users found. Register the first user at:
  http://localhost:3000/register/abc123...
This link expires in 1 hour.
```

Open that URL, choose a display name, and register a passkey. This creates an account and
signs in automatically.

### Install from Git

To build and install from source, you'll need a working Rust toolchain:

```bash
cargo install --locked --git https://github.com/jnsgruk/brewlog.git
```

Then create a `.env` file containing at least your OpenRouter and Foursquare API keys, and start
the server:

```bash
brewlog serve
```

### CLI Authentication

To use the CLI or API for write operations, create a token via browser hand-off:

```bash
brewlog token create --name "my-cli-token"
# Browser opens → authenticate with a passkey → token printed once

export BREWLOG_URL="http://localhost:3000"
export BREWLOG_TOKEN="<token from above>"

# Create data from the CLI
brewlog roaster add --name "Radical Roasters" --country "United Kingdom"
```

Run `brewlog --help` for the full command reference.

## Configuration

All settings are read from environment variables or CLI flags. A `.env` file in the working
directory is loaded automatically via [dotenvy](https://crates.io/crates/dotenvy).

### Server (`brewlog serve`)

| Variable                   | Purpose                                                                | Default                 |
| -------------------------- | ---------------------------------------------------------------------- | ----------------------- |
| `BREWLOG_RP_ID`            | WebAuthn Relying Party ID (server domain)                              | `localhost`             |
| `BREWLOG_RP_ORIGIN`        | WebAuthn Relying Party origin (full URL)                               | `http://localhost:3000` |
| `BREWLOG_DATABASE_URL`     | Database connection string                                             | `sqlite://brewlog.db`   |
| `BREWLOG_BIND_ADDRESS`     | Server bind address                                                    | `127.0.0.1:3000`        |
| `BREWLOG_INSECURE_COOKIES` | Disable the `Secure` cookie flag (auto-enabled for localhost defaults) | `false`                 |
| `RUST_LOG`                 | Log level filter                                                       | `info`                  |
| `RUST_LOG_FORMAT`          | Set to `json` for structured log output                                | —                       |

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
