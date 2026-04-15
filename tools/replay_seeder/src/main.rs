use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "http://localhost:8081/v1/orders")]
    url: String,
    #[arg(long, default_value = "data/nba_okc_sample.json")]
    data: PathBuf,
    /// Speed multiplier (2.0 = half delays)
    #[arg(long, default_value_t = 1.0)]
    speed: f64,
}

#[derive(Deserialize)]
struct Sample {
    market_id: String,
    #[allow(dead_code)]
    timeline: Vec<serde_json::Value>,
    orders: Vec<OrderSpec>,
}

#[derive(Deserialize)]
struct OrderSpec {
    delay_ms: u64,
    side: String,
    price: u32,
    qty: u32,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let raw = std::fs::read_to_string(&args.data)?;
    let sample: Sample = serde_json::from_str(&raw)?;
    let client = reqwest::Client::new();
    let user = Uuid::new_v4();

    println!(
        "Seeding {} orders to {} as user {}",
        sample.orders.len(),
        args.url,
        user
    );

    for (i, o) in sample.orders.iter().enumerate() {
        let wait = Duration::from_millis((o.delay_ms as f64 / args.speed.max(0.01)) as u64);
        tokio::time::sleep(wait).await;

        let body = serde_json::json!({
            "market_id": sample.market_id,
            "side": o.side,
            "price": o.price,
            "qty": o.qty,
            "tif": "GTC",
            "idempotency": Uuid::new_v4().to_string(),
        });

        let resp = client
            .post(&args.url)
            .header("X-User-Id", user.to_string())
            .json(&body)
            .send()
            .await?;
        println!(
            "[{}] {} {} @ {} -> {}",
            i,
            o.side,
            o.qty,
            o.price,
            resp.status()
        );
    }

    println!("done");
    Ok(())
}
