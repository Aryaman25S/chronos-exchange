# Benchmarks

## Methodology

- **HTTP load**: `tools/loadgen` posts randomized GTC orders to `POST /v1/orders` with unique idempotency keys. Tune `--qps`, `--concurrency`, `--seconds`.
- **Engine unit cost**: Run `cargo test -p engine` for correctness; for wall-time of matching, observe `chronos_gateway_engine_match_seconds` histogram from `/metrics` while loadgen runs against the gateway.

## How to reproduce

```bash
cargo build -p loadgen -p gateway
# terminal A
DATA_DIR=./data_runtime RUST_LOG=info ./target/debug/gateway
# terminal B
./target/debug/loadgen --qps 2000 --seconds 30 --concurrency 8 --url http://localhost:8081/v1/orders
```

## Results (fill in on your machine)

| Workload | Approx accepted QPS | p99 match latency (from Prometheus) |
|----------|---------------------|-------------------------------------|
| Laptop M-series, loadgen as above | _TBD_ | _TBD_ |

Prometheus scrape: `GET http://localhost:8081/metrics` (see `deploy/prometheus.yml`).
