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

On first start, you **must** set an admin password via the `BREWLOG_ADMIN_PASSWORD` environment variable:

```bash
BREWLOG_ADMIN_PASSWORD="your-secure-password" brewlog serve
```

This creates the admin user in the database. On subsequent starts, the password is not required.

### Authentication

Brewlog supports two authentication methods:

1. **Web Frontend**: Session-based authentication via login page
2. **CLI/API**: Token-based authentication via Bearer tokens

#### Web Authentication

1. Start the server and visit `http://localhost:3000`
2. Click "Login" in the navigation bar
3. Sign in with username `admin` and your password
4. You're now authenticated and can create/update/delete records

#### CLI/API Authentication

First, create an API token:

```bash
brewlog create-token --name "my-cli-token"
# Enter username: admin
# Enter password: ****
# 
# Token created successfully!
# Token ID: abc123
# Token: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

Export the token and use it for all CLI commands:

```bash
export BREWLOG_TOKEN="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
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
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
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

## Environment Variables

### Server Configuration

- **`BREWLOG_ADMIN_PASSWORD`** *(required on first start)*: Sets the admin user password. Must be provided when starting the server for the first time.
- **`BREWLOG_SECURE_COOKIES`**: Set to `"true"` to enable the `Secure` flag on session cookies (HTTPS-only transmission). Recommended for production deployments.
- **`DATABASE_URL`**: SQLite database file path (default: `brewlog.db`)
- **`BIND_ADDRESS`**: Server bind address (default: `0.0.0.0:3000`)

### CLI Configuration

- **`BREWLOG_URL`**: API server URL (default: `http://127.0.0.1:3000`)
- **`BREWLOG_TOKEN`**: API authentication token for write operations

## Security Considerations

### Password Security

- Passwords are hashed using **Argon2id** with industry-standard secure defaults
- Password hashing uses constant-time comparison to prevent timing attacks
- Never store passwords in plain text - always use the password prompt

### Token Security

- API tokens are cryptographically secure 32-byte random values
- Tokens are stored as **SHA-256 hashes** in the database
- Token values are only displayed **once** at creation time
- Revoked tokens cannot be reused

### Session Security

- Session tokens are 256-bit cryptographically secure random values
- Sessions are stored in the database with 30-day expiration
- Session tokens are hashed with **SHA-256** before storage
- Cookies are **HttpOnly** and **SameSite=Strict** for CSRF protection
- Sessions are properly invalidated on logout

### Production Deployment

When deploying to production:

1. **Enable HTTPS**: Set `BREWLOG_SECURE_COOKIES=true` to ensure cookies are only transmitted over HTTPS
2. **Use Strong Passwords**: Choose a strong admin password (12+ characters, mixed case, numbers, symbols)
3. **Restrict Access**: Use firewall rules to limit who can access the server
4. **Regular Updates**: Keep your Brewlog installation up to date
5. **Backup Database**: Regularly backup your `brewlog.db` file
6. **Revoke Unused Tokens**: Periodically review and revoke API tokens you no longer need

### Authentication Model

Brewlog uses a **single-user** authentication model:

- Only one user account exists (`admin`)
- No sign-up or password recovery flows
- No multi-user support or permissions
- All authenticated users have full access to all operations

This design assumes Brewlog is deployed for **personal use** or in a **trusted environment**.
