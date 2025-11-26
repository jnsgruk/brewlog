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

On first start, you must set an admin username and password via the `BREWLOG_ADMIN_USERNAME` and `BREWLOG_ADMIN_PASSWORD` environment variables:

```bash
BREWLOG_ADMIN_USERNAME="admin" BREWLOG_ADMIN_PASSWORD="your-secure-password" brewlog serve
```

This creates the admin user in the database. On subsequent starts, the environment variables are not required.

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
brewlog create-token --name "my-cli-token"
# You will be prompted for username and password.
# Alternatively, you can provide them via flags:
# brewlog create-token --name "my-cli-token" --username admin --password secret

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
brewlog add-roaster \
  --name "Radical Roasters" \
  --country "UK" \
  --city "Bristol" \
  --homepage "https://radicalroasters.co.uk"

brewlog add-roast \
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
brewlog list-tokens

# Revoke a token
brewlog revoke-token --id abc123
```

#### API Usage

For direct API access, include your token as a Bearer token:

```bash
curl http://localhost:3000/api/v1/roasters \
  -H "Authorization: Bearer dEadB3efDeadb33fdeadb33F..." \
  --json '{"name":"Radical Roasters","country":"UK"}'
```

**Note**: All read operations (GET requests) are public and don't require authentication. Only write operations (POST/PUT/DELETE) require authentication.

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

The project includes a number of unit and integration tests, all of which can be executed with `cargo`:

```bash
cargo test
```
