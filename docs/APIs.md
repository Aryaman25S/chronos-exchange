# HTTP and WebSocket APIs

Base URL: `http://localhost:8081` (or gateway service in Docker).

## Headers

- **`X-User-Id`**: UUID. Stable identity for rate limits, positions, and fills. If omitted, the gateway generates a random UUID per request (not recommended).
- **`X-Admin-Token`**: Required for admin routes; must match env `ADMIN_TOKEN`.

## REST

| Method | Path | Body / query | Response |
|--------|------|----------------|----------|
| `GET` | `/v1/orders` | Query `market_id?` — if set, only that market; else all markets | JSON array of open (resting) orders for the caller (`X-User-Id`): `{ market_id, order_id, side, price, qty }` |
| `POST` | `/v1/orders` | `{ market_id, side, price, qty, tif, idempotency? }` | `{ fills, status, self_trade_prevented, rested }` — see below |
| `DELETE` | `/v1/orders/{id}` | Query `market_id?` (default `OKC_WIN_YESNO`) | `ok` text |
| `PUT` | `/v1/orders/{id}` | `{ market_id, new_price?, new_qty? }` | `ok` text |
| `GET` | `/v1/markets` | — | JSON array per market: fields above plus **`settled`**, and **`stats`**: `{ volume_usd, fill_count, best_bid_cents, best_ask_cents, mid_cents, yes_implied_pct, last_trade_cents }` — book snapshot (live) plus **session** sim volume/fill count from the in-memory ledger (resets on gateway restart; engine/WAL recovery does not rebuild historical fill stats). |
| `GET` | `/v1/positions` | — | Map `market_id -> { qty_yes, avg_price_cents }` |
| `GET` | `/v1/fills?limit=50` | — | JSON array of recent fills |
| `POST` | `/v1/admin/markets/{market_id}/settle` | `{ resolve_yes: bool }` | `ok` text |
| `GET` | `/metrics` | — | Prometheus text |

### `POST /v1/orders` response

- **`fills`**: Immediate executions (may be empty).
- **`self_trade_prevented`**: `true` if the matcher canceled your incoming order because it would only trade against **your own** resting size at the top of book (nothing rests).
- **`rested`**: `true` if remaining quantity was posted as GTC (only when not fully filled and TIF is GTC).

### Time in force

- `GTC`: rest unfilled size on book.
- `IOC`: fill what crosses, cancel remainder.
- `FOK`: fill **entire** size at crossing prices or **reject with no book change** (no partial).

## WebSocket `/ws`

Connect with optional query **`?market_id=YOUR_MARKET_ID`**. The initial snapshot and stream are tagged for that book; the ring may include other markets’ updates (clients should filter on `market_id` in each message).

1. Optional first client message (within ~250ms): `{"type":"resync","from_seq":N}`.
2. Server sends a **snapshot**: `{ type, market_id, seq, bids: [[price,qty],...], asks, last_trade }`.
3. If `resync` was sent, server sends buffered ring messages with `seq > N` (mixed snapshots/deltas/trades).
4. Then live stream: `snapshot` (20ms per market), `delta` (on book changes), `trade` (each fill). Each message includes **`market_id`**.

Price is in **cents** (1 = $0.01). UI divides by 100 for display dollars.

## Market catalog (`MARKETS_SEED`)

At startup the gateway loads **`MARKETS_SEED`** (path to a JSON file; default `./markets_seed.json` if present). Format:

```json
{ "markets": [ { "id": "...", "name": "...", "tick_size": 1, "description": "...", "tags": ["a","b"] } ] }
```

If the file is missing or invalid, a built-in default market is used.
