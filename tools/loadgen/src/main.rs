
use clap::Parser;
use rand::Rng;
use serde_json::json;
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value_t = 1000)] qps: u32,
    #[arg(long, default_value_t = 10)] seconds: u64,
    #[arg(long, default_value = "http://localhost:8081/v1/orders")] url: String,
    #[arg(long, default_value_t = 4)] concurrency: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let client = reqwest::Client::new();
    let start = Instant::now();
    let total = args.qps as usize * args.seconds as usize;
    let per_task = (total + args.concurrency - 1) / args.concurrency;
    let mut tasks = vec![];
    for _ in 0..args.concurrency {
        let c = client.clone();
        let url = args.url.clone();
        tasks.push(tokio::spawn(async move {
            let mut rng = rand::thread_rng();
            let mut ok = 0usize;
            for _ in 0..per_task {
                let price = 40 + rng.gen_range(0..20);
                let side = if rng.gen_bool(0.5) { "buy" } else { "sell" };
                let body = json!({ "market_id":"OKC_WIN_YESNO", "side": side, "price": price, "qty": 1 + rng.gen_range(0..5), "tif":"GTC", "idempotency": format!("{}", Uuid::new_v4()) });
                if let Ok(resp) = c.post(&url).json(&body).send().await { if resp.status().is_success() { ok += 1; } }
                tokio::time::sleep(Duration::from_micros((1_000_000u64 / (args.qps as u64 * args.concurrency as u64)) as u64)).await;
            }
            ok
        }));
    }
    let mut ok = 0usize; for t in tasks { ok += t.await??; }
    let dur = start.elapsed();
    println!("sent={} ok={} duration_secs={:.2} approx_qps={:.1}", total, ok, dur.as_secs_f64(), ok as f64 / dur.as_secs_f64());
    Ok(())
}
