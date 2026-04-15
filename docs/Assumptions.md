# Assumptions

- **Single-node demo**: one gateway process; Postgres in compose is optional for future durable API state.
- **Binary contract**: One outcome per market priced in cents (0–100). The UI labels **YES** as buy and **NO** as sell of the same instrument (complementary notionally as \(1 - p\)); there is not a separate NO contract in the matcher.
- **WAL fsync**: Each `WalRecord` append calls `flush()`; group commit is not implemented (see Architecture for snapshot/WAL interaction).
- **Order timestamp**: Resting orders use `ts` derived from `order_id` for deterministic replay hashing.
- **Settlement**: Admin-only; **simulated** P&L in the ledger (not legal/financial advice).
