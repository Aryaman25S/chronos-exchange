# Deploying Chronos Exchange

The trading path is **gateway + engine** (WAL under `DATA_DIR`); there is **no database** required. For a **public URL** that serves both the **built React UI** and the **API/WebSocket** on one origin, use **`deploy/Dockerfile.web`** (nginx on **8080** proxies `/v1` and `/ws` to the gateway on localhost **8081**). The UI uses relative `/v1` and same-origin `/ws`, so no `VITE_WS_URL` is needed when everything is behind one host.

The `postgres` service in `docker-compose.yml` is optional and **not used** by the gateway.

### Fly.io: `Cargo.toml does not contain a valid package section`

The repo root **`Cargo.toml` is a Rust workspace** (only `[workspace]`, no `[package]`). **`fly launch`** scans the directory first and tries to treat it like a single crate, which fails.

**Fix (pick one):**

1. **Point Fly at our Dockerfile** (from the repo root):

   ```bash
   fly launch --dockerfile deploy/Dockerfile.web
   ```

2. **Skip `fly launch` and use config + deploy** (recommended; no scanner):

   ```bash
   cp deploy/fly.toml.example fly.toml
   # Edit fly.toml: set app = "your-unique-name"
   fly apps create your-unique-name    # omit if the app already exists
   fly deploy
   ```

---

## Recommended: single image (nginx + UI + gateway)

### Build locally

From the **repository root**:

```bash
docker build -f deploy/Dockerfile.web -t chronos-web .
docker run --rm -p 8080:8080 -e ADMIN_TOKEN=dev chronos-web
```

Open **http://localhost:8080** — static UI, API at `/v1`, WebSocket at `/ws`. `/metrics` is **not** exposed at the edge (returns 404 via nginx); the gateway still serves metrics on localhost inside the container.

### Fly.io (step by step)

1. Install the [Fly CLI](https://fly.io/docs/hands-on/install-flyctl/). Sign in when prompted (`fly launch` / `fly deploy` will ask if needed).

2. From the **chronos-exchange repo root**, copy the example config and set a **unique** app name:

   ```bash
   cp deploy/fly.toml.example fly.toml
   # Edit fly.toml: set app = "your-unique-name"
   ```

3. Create the app (first time only) and deploy:

   ```bash
   fly apps create your-unique-name
   fly deploy
   ```

   **Alternative:** `fly launch --dockerfile deploy/Dockerfile.web` (see the note above if bare `fly launch` fails on `Cargo.toml`).

4. **Secrets (optional but recommended for admin settle):**

   ```bash
   fly secrets set ADMIN_TOKEN="$(openssl rand -hex 16)"
   ```

5. **Optional volume** so engine data survives restarts (same region as the app):

   ```bash
   fly volumes create chronos_data --region iad --size 1
   ```

   Then uncomment the `[[mounts]]` block in `fly.toml` (and match `region` + volume name).

6. Share **`https://<your-app>.fly.dev`** — one HTTPS origin; WebSockets use `wss://` automatically.

**Env defaults in the image:** `DATA_DIR=/data`, `MARKETS_SEED=/etc/chronos/markets_seed.json`, `CHRONOS_SEED_DUMMY_TRADES=0` (set to `1` if you want first-run synthetic liquidity on a fresh volume).

---

## Local development: Docker Compose

From `deploy/`:

```bash
export ADMIN_TOKEN=dev
docker compose up --build
```

- UI (Vite dev): **5173**
- Gateway: **8081**

---

## Other hosts (Railway, Render, etc.)

- **Dockerfile:** `deploy/Dockerfile.web` (build context = repo root).
- **Public port:** container **8080** (nginx).
- **Health check:** `GET /` (serves `index.html`).
- **Disk:** mount persistent storage on **`/data`** if you want WAL to survive restarts.

If you split UI and API onto different domains, set **`VITE_WS_URL`** at UI build time to your full `wss://.../ws` URL; same-origin deployment does not need it.
