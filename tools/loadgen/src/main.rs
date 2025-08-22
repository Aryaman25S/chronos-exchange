use clap::Parser;
use rand::Rng;
use serde_json::json;
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Parser, Debug, Clone)]
struct Args {
    /// Target total QPS across all workers
    #[arg(long, default_value_t = 1000)]
    qps: u32,
    /// Test duration in seconds
    #[arg(long, default_value_t = 10)]
    seconds: u64,
    /// Gateway orders endpoint
    #[arg(long, default_value = "http://localhost:8081/v1/orders")]
    url: String,
    /// Concurrent workers (Tokio tasks)
    #[arg(long, default_value_t = 4)]
    concurrency: usize,
    /// Market ID to target
    #[arg(long, default_value = "OKC_WIN_YESNO")]
    market_id: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let client = reqwest::Client::new();

    let total_target = args.qps as usize * args.seconds as usize;
    let per_task = (total_target + args.concurrency - 1) / args.concurrency;

    // Pace each worker at (qps / concurrency)
    let per_task_qps = (args.qps as f64) / (args.concurrency as f64);
    let sleep_us = if per_task_qps > 0.0 {
        (1_000_000.0 / per_task_qps).round() as u64
    } else {
        0
    };

    let start = Instant::now();
    let mut tasks = Vec::with_capacity(args.concurrency);
    for _ in 0..args.concurrency {
        let c = client.clone();
        let url = args.url.clone();
        let market_id = args.market_id.clone();

        tasks.push(tokio::spawn(async move {
            let mut ok = 0usize;
            for _ in 0..per_task {
                // Generate randomness in a scope that ends before `.await`
                let (price, side, qty) = {
                    let mut r = rand::thread_rng();
                    let price = 40 + r.gen_range(0..20);          // 40..59 (cents)
                    let side: &str = if r.gen_bool(0.5) { "buy" } else { "sell" };
                    let qty = 1 + r.gen_range(0..5);               // 1..5
                    (price, side, qty)
                };

                let body = json!({
                    "market_id": market_id,
                    "side": side,
                    "price": price,
                    "qty": qty,
                    "tif": "GTC",
                    "idempotency": Uuid::new_v4().to_string(),
                });

                if let Ok(resp) = c.post(&url).json(&body).send().await {
                    if resp.status().is_success() {
                        ok += 1;
                    }
                }

                if sleep_us > 0 {
                    tokio::time::sleep(Duration::from_micros(sleep_us)).await;
                }
            }
            ok
        }));
    }

    // Gather results
    let mut ok = 0usize;
    for t in tasks {
        ok += t.await?; // JoinHandle<usize> -> usize
    }
    let dur = start.elapsed();
    let sent = per_task * args.concurrency;
    let approx_qps = (ok as f64) / dur.as_secs_f64();

    println!(
        "target_total={} sent={} ok={} duration_secs={:.2} approx_qps={:.1}",
        total_target, sent, ok, dur.as_secs_f64(), approx_qps
    );

    Ok(())
}