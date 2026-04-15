# Chronos Exchange (Simulation Only)

Play-money event trading simulator: Rust matching engine (price–time, STP, IOC/FOK/GTC), WAL + zstd snapshots, Axum gateway with REST/WebSocket, Prometheus metrics, React depth ladder + tape.

## Quickstart

```bash
cargo build --workspace
cd deploy
export ADMIN_TOKEN=dev   # required for POST /v1/admin/.../settle
docker compose up --build
```

- **UI**: http://localhost:5173 — Vite proxies `/v1` and `/ws` to the gateway. **React Router**: `/` is the markets browser (search, horizontally scrollable topic filters, cards with volume and implied odds), `/m/:marketId` is the trading view. **TanStack Query** for REST; trading screen has a **depth ladder**, **order ticket**, **tape**, and **positions/activity** panels.
- **Gateway**: http://localhost:8081
- **Metrics**: http://localhost:8081/metrics
- **Prometheus**: http://localhost:9090
- **Grafana**: http://localhost:3000

## ~90 second demo

1. Open the UI at `/` — browse or search, then open a market (e.g. `/m/OKC_WIN_YESNO`). Your user id is stored in `localStorage` and sent as `X-User-Id`.
2. Use **Buy YES** / **Sell YES** at different prices; watch **mid/spread**, **Tape**, and **Activity**. On mobile, open **Trade** for the bottom sheet ticket.
3. After a fill, **Positions** and **Activity** refresh automatically; self-trade blocks show inline hints.
4. Settle the market (from host):

   ```bash
   curl -s -X POST "http://localhost:8081/v1/admin/markets/OKC_WIN_YESNO/settle" \
     -H "Content-Type: application/json" -H "X-Admin-Token: dev" \
     -d '{"resolve_yes":true}'
   ```

5. **Replay seeder** (with gateway running):

   ```bash
   cd /path/to/chronos-exchange
   cargo run -p replay_seeder -- --data data/nba_okc_sample.json
   ```

6. In Grafana, open the Chronos dashboard (if provisioned) for order rate and match latency histogram.

See [docs/Architecture.md](docs/Architecture.md), [docs/APIs.md](docs/APIs.md), [docs/OrderFlow.md](docs/OrderFlow.md), [docs/Benchmarks.md](docs/Benchmarks.md). Deployment options: [deploy/README.md](deploy/README.md).

### Market catalog & env

- **`DATA_DIR`**: persisted engine state (WAL, snapshots). Default `./data_runtime`; Docker sets `/data` via the gateway image.
- **`MARKETS_SEED`**: path to `markets_seed.json` (see [docs/APIs.md](docs/APIs.md)). Default filename is resolved from the process working directory; Docker image sets `/etc/chronos/markets_seed.json`.
- **Dummy book & volume (sim)**: On the **first** startup for a given `DATA_DIR`, the gateway places synthetic GTC ladders and IOC crosses per seeded market so list cards show plausible spread, implied %, and session volume. Skip with `CHRONOS_SEED_DUMMY_TRADES=0`. To re-seed later, remove `DATA_DIR/.dummy_trades_seeded` (and usually reset or remove the WAL under that directory so you do not stack duplicate history).
- **UI**: optional `VITE_WS_URL` in `ui/.env.example` — otherwise the app uses same-origin `/ws` (Vite proxies to the gateway in dev).

### Future (not on the core roadmap)

Server-side trade candles / charting and a separate SEO marketing site (e.g. Next.js) are optional follow-ups if you need them.

## Safety

Simulation only — no real money or prizes.
