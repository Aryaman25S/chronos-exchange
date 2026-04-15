# Order placement flow (HTTP to book)

This diagram shows how a single `POST /v1/orders` request moves through the gateway into the engine and out to market-data subscribers.

```mermaid
sequenceDiagram
  participant Client as Client_UI_or_loadgen
  participant GW as Gateway_Axum
  participant Risk as Risk_limits
  participant Led as Ledger_sim
  participant Eng as Engine_OrderBook
  participant WAL as WAL_on_disk
  participant MD as md_Broadcaster

  Client->>GW: POST /v1/orders JSON
  GW->>GW: Parse X-User-Id header
  GW->>Risk: idempotency + rate limit
  Risk-->>GW: ok or reject
  GW->>Led: check_intent position caps
  Led-->>GW: ok or reject
  GW->>Eng: place_order NewOrder
  Eng->>WAL: append WalRecord Place
  Eng->>Eng: OrderBook match STP fills
  Eng-->>GW: PlaceResult fills rested
  GW->>Led: apply_fills
  GW->>MD: publish_trade + emit_book_update delta
  GW-->>Client: JSON fills status
```

**Notes**

- The engine persists **before** applying the match (`WAL` append then `place`) so recovery can replay the same sequence.
- WebSocket clients receive **delta** and **trade** frames from `md`, not the HTTP response body.
- Settlement and cancel/replace follow the same pattern: WAL record first, then book mutation, then MD update.
