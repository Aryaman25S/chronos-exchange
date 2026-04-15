# Deploying Chronos Exchange

The stack is a **single-node** gateway (Rust) plus optional **Vite UI**. There is **no database requirement** for the trading path: engine state lives under `DATA_DIR` (WAL + snapshots). The `postgres` service in `docker-compose.yml` is optional and is **not used by the gateway** today; you can remove it locally if you prefer a smaller compose file.

## One-command options (pick one)

### A. Docker Compose (local or small VM)

From the repo root:

```bash
cd deploy
export ADMIN_TOKEN=dev
docker compose up --build
```

- UI: port **5173** (Vite dev server in the default compose file)
- Gateway: **8081**

Set `CHRONOS_SEED_DUMMY_TRADES=0` on the gateway service if you do not want synthetic book seeding on first startup.

### B. Fly.io (example)

Prerequisites: [Fly CLI](https://fly.io/docs/hands-on/install-flyctl/), logged in.

1. Create an app and attach a volume for `DATA_DIR` (persistent WAL/snapshots), e.g. `/data`.
2. Build from `gateway/Dockerfile` (context: repository root, as in compose).
3. Set secrets/env: `ADMIN_TOKEN`, `DATA_DIR=/data`, `MARKETS_SEED=/etc/chronos/markets_seed.json` (image already copies `markets_seed.json` to that path).
4. Expose **8081** (HTTP + WebSocket on same port).

Example `fly.toml` fragment (adjust app name and region):

```toml
app = "chronos-exchange"
primary_region = "iad"

[build]
  dockerfile = "gateway/Dockerfile"
  context = "."

[http_service]
  internal_port = 8081
  force_https = true
  auto_stop_machines = true
  auto_start_machines = true

[[mounts]]
  source = "chronos_data"
  destination = "/data"
```

Run `fly volumes create chronos_data --size 1` (or similar), then `fly deploy` from the repository root with `Dockerfile` path overridden to `gateway/Dockerfile` if needed.

### C. Railway / Render (example)

- **Build**: Docker image using `gateway/Dockerfile` with context **parent directory** (repo root), same as Fly.
- **Start command**: `gateway` (default `CMD` in the image).
- **Port**: bind public HTTP to **8081**.
- **Volume**: mount a persistent disk on `DATA_DIR` (e.g. `/data`) so WAL survives restarts.
- **Env**: `DATA_DIR`, `ADMIN_TOKEN`, `MARKETS_SEED` as above.

### UI in production

The repo’s default compose runs **Vite dev** for convenience. For a public site, build static assets (`ui`: `npm run build`) and serve `dist/` with nginx, Caddy, or S3+CloudFront, proxying `/v1` and `/ws` to the gateway origin. Configure `VITE_WS_URL` if the UI and API are on different hosts.
