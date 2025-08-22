
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let e = engine::Engine::new("./data_runtime".into())?; e.restore_from_latest()?;
    let h = e.state_hash()?; println!("state_hash={}", hex::encode(h)); Ok(())
}
