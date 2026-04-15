# Deploying Chronos Exchange

The trading path is **gateway + engine** (WAL under `DATA_DIR`); there is **no database** required. For a **public URL** that serves both the **built React UI** and the **API/WebSocket** on one origin, use **`deploy/Dockerfile.web`** (nginx on **8080** proxies `/v1` and `/ws` to the gateway on localhost **8081**). The UI uses relative `/v1` and same-origin `/ws`, so no `VITE_WS_URL` is needed when everything is behind one host.

The `postgres` service in `docker-compose.yml` is optional and **not used** by the gateway.

---

## GitHub + Fly.io (push to deploy)

Use this when you want **Fly to build and deploy from your GitHub repo** on every push to `main` (workflow: [`.github/workflows/fly-deploy.yml`](../.github/workflows/fly-deploy.yml)).

### One-time setup

1. **Put the code on GitHub** (create a repo and push this project, or fork it).

2. **Install `flyctl`** and log in locally (once):

   ```bash
   fly auth login
   ```

3. **Pick a unique Fly app name** (e.g. `chronos-yourname`). Create the app on Fly:

   ```bash
   fly apps create chronos-yourname
   ```

4. **Add `fly.toml` at the repo root** (Fly reads this on deploy). Copy the example and set the same name:

   ```bash
   cp deploy/fly.toml.example fly.toml
   # Edit fly.toml: app = "chronos-yourname"
   ```

5. **Create a deploy token** and add it to GitHub:

   ```bash
   fly tokens create deploy
   ```

   In GitHub: **Repo → Settings → Secrets and variables → Actions → New repository secret**  
   Name: **`FLY_API_TOKEN`**  
   Value: the token from the command above.

6. **Commit and push** `fly.toml` and the workflow (already in this repo under `.github/workflows/`):

   ```bash
   git add fly.toml .github/workflows/fly-deploy.yml
   git commit -m "chore: Fly.io deploy config"
   git push origin main
   ```

   The **Deploy to Fly.io** workflow runs on each push to `main`. Check **Actions** for logs; when it succeeds, open **`https://chronos-yourname.fly.dev`** (use your real app name).

7. **Optional:** set admin token on the Fly app (for settlement API):

   ```bash
   fly secrets set ADMIN_TOKEN="$(openssl rand -hex 16)" -a chronos-yourname
   ```

8. **Optional volume** for persistent `/data`: see [Fly.io volumes](https://fly.io/docs/reference/volumes/) and uncomment `[[mounts]]` in `fly.toml` after `fly volumes create ...`.

### Fly dashboard “Launch from GitHub”

The onboarding screen that says **Sign in with GitHub** is an alternative way to **link your GitHub account to Fly**. You still need a **`fly.toml`** that points at **`deploy/Dockerfile.web`** and a way to deploy (this repo uses **GitHub Actions** + **`FLY_API_TOKEN`**). You can use the dashboard to create the app or manage secrets, but the **push-to-deploy** path above is the one this repository is set up for.

### Build details

- **`flyctl deploy --remote-only`** builds the image on **Fly’s builders** (no Docker required on GitHub’s runners for the Rust/UI compile).
- Image definition: **`deploy/Dockerfile.web`**, build context **repository root** (see `[build]` in `fly.toml`).

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

1. Install the [Fly CLI](https://fly.io/docs/hands-on/install-flyctl/) and run `fly auth login`.
2. From the repo root, copy the example config and choose a **unique** app name:

   ```bash
   cp deploy/fly.toml.example fly.toml
   # Edit fly.toml: set app = "your-unique-name"
   ```

3. Create the app (first time) and deploy:

   ```bash
   fly launch --no-deploy --copy-config   # if needed, or fly apps create your-unique-name
   fly deploy
   ```

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
