# Invariants

- **No crossed book** after resting operations (see `engine/tests/props.rs`).
- **FIFO at price** via `VecDeque` per price level.
- **FOK**: either full size executes against the book (no self-trade blocking completion) or the order is rejected with **no** mutation.
- **Deterministic WAL replay**: order `ts` is derived from `order_id`; replaying the WAL into a fresh engine yields the same `state_hash` as a continuous run without snapshot (`engine/tests/integration.rs`).
- **STP**: incoming order is zeroed when the best resting counterparty is the same user (no trade print, nothing rests). The API exposes `self_trade_prevented` on `POST /v1/orders` when this path fires.
