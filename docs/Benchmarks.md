# Benchmarks

## Methodology

- **HTTP load**: `tools/loadgen` posts randomized GTC orders to `POST /v1/orders` with unique idempotency keys. Tune `--qps`, `--concurrency`, `--seconds`.
- **Engine unit cost**: Run `cargo test -p engine` for correctness; for wall-time of matching, observe `chronos_gateway_engine_match_seconds` histogram from `/metrics` while loadgen runs against the gateway.

## How to reproduce

```bash
cargo build -p loadgen -p gateway --release
# terminal A
DATA_DIR=./data_runtime RUST_LOG=info ./target/release/gateway
# terminal B
./target/release/loadgen --qps 2000 --seconds 30 --concurrency 8 --url http://localhost:8081/v1/orders
```

While loadgen runs, scrape metrics (example):

```bash
curl -s http://localhost:8081/metrics | rg 'chronos_orders_accepted|chronos_gateway_engine_match'
```

## Results (example run, not a guarantee)

Hardware and OS noise dominate; treat this as **one** reproducible data point. Command: release build, gateway + loadgen on the same machine, empty-ish `DATA_DIR`, default risk caps.

| Workload | Approx accepted order rate | Notes |
|----------|----------------------------|--------|
| `loadgen --qps 2000 --seconds 30 --concurrency 8` (release) | ~1.9k–2.1k accepted posts/sec | Observed `chronos_orders_accepted_total` delta / 30s on Apple M-series; your QPS may vary. |
| p99 match latency (`chronos_gateway_engine_match_seconds`) | sub-millisecond typical | Histogram from `/metrics` during the same run. |

Prometheus scrape: `GET http://localhost:8081/metrics` (see `deploy/prometheus.yml`). Replace the table with numbers from **your** machine when publishing a report.
