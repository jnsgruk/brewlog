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

Start the server:

```bash
brewlog serve
```

Interact with a running instance via the CLI:

```bash
# Point the CLI at your server (defaults to http://127.0.0.1:3000)
export BREWLOG_URL=http://localhost:3000

# Add a roaster
brewlog add-roaster \
  --name "Radical Roasters" \
  --country "UK" \
  --city "Bristol" \
  --homepage "https://radicalroasters.co.uk"

# Add a roast metadata and tasting notes
brewlog add-roast \
  --roaster-id "deadbeef" \
  --name "Chelbesa Lot 2" \
  --origin "Ethiopia" \
  --region "Gedeo" \
  --producer "Chelbesa Cooperative" \
  --process "Washed" \
  --tasting-notes "Blueberry, Jasmine"
```

Every CLI command maps to an HTTP endpoint. You can perform the same operations with `curl`,
Postman, or any HTTP client:

```bash
curl http://localhost:3000/api/v1/roasters \
    --json '{"name":"Radical Roasters","country":"UK","city":"Bristol","homepage":"https://radicalroasters.co.uk"}'
```

Once the server is running, visit `http://localhost:3000` to access the user interface.

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
