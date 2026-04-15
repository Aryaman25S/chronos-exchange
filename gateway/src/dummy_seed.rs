//! One-time synthetic liquidity and trades so list/market metrics look populated.
//! Skipped if `data_dir/.dummy_trades_seeded` exists, or if `CHRONOS_SEED_DUMMY_TRADES=0`.

use std::path::Path;

use anyhow::Result;
use engine::types::{NewOrder, PlaceResult, Side, Tif};
use uuid::Uuid;

use crate::emit_book_update;
use crate::AppState;

pub fn maybe_seed_dummy_trades(app: &AppState, data_dir: &Path, market_ids: &[String]) -> Result<()> {
    let marker = data_dir.join(".dummy_trades_seeded");
    if marker.exists() {
        return Ok(());
    }
    if std::env::var("CHRONOS_SEED_DUMMY_TRADES")
        .map(|s| s == "0" || s.eq_ignore_ascii_case("false"))
        .unwrap_or(false)
    {
        return Ok(());
    }

    tracing::info!(
        "seeding dummy orders/trades for {} markets (delete {:?} to re-seed)",
        market_ids.len(),
        marker
    );

    for (mi, market_id) in market_ids.iter().enumerate() {
        seed_one_market(app, market_id, mi)?;
    }

    std::fs::write(&marker, b"ok")?;
    tracing::info!("dummy trade seed finished");
    Ok(())
}

fn hash_mid(market_id: &str) -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    market_id.hash(&mut h);
    let x = h.finish();
    25 + (x % 50) as u32 // 25–74¢ implied center
}

/// Deterministic pseudo-UUIDs so each resting order has a distinct user (avoids STP and rate limits).
fn oid(mi: usize, seq: u32) -> Uuid {
    Uuid::from_u128(((mi as u128) << 96) | (seq as u128))
}

fn seed_one_market(app: &AppState, market_id: &str, mi: usize) -> Result<()> {
    let mid = hash_mid(market_id);

    // Bid ladder (buy YES): below mid
    for k in 0..6u32 {
        let price = mid.saturating_sub(k + 1).max(1);
        let qty = 12 + ((mi * 13 + k as usize) % 120) as u32;
        place_internal(
            app,
            NewOrder {
                id: oid(mi, 100 + k),
                user_id: oid(mi, 100 + k),
                market_id: market_id.to_string(),
                side: Side::Buy,
                price,
                qty,
                tif: Tif::Gtc,
                idempotency: None,
            },
        )?;
    }
    // Ask ladder (sell YES): above mid
    for k in 0..6u32 {
        let price = (mid + k + 1).min(99);
        let qty = 12 + ((mi * 17 + k as usize) % 120) as u32;
        place_internal(
            app,
            NewOrder {
                id: oid(mi, 200 + k),
                user_id: oid(mi, 200 + k),
                market_id: market_id.to_string(),
                side: Side::Sell,
                price,
                qty,
                tif: Tif::Gtc,
                idempotency: None,
            },
        )?;
    }

    // IOC taker lifts offers (volume + last trade)
    place_internal(
        app,
        NewOrder {
            id: oid(mi, 300),
            user_id: oid(mi, 300),
            market_id: market_id.to_string(),
            side: Side::Buy,
            price: 99,
            qty: 45,
            tif: Tif::Ioc,
            idempotency: None,
        },
    )?;

    // IOC taker hits bids
    place_internal(
        app,
        NewOrder {
            id: oid(mi, 301),
            user_id: oid(mi, 301),
            market_id: market_id.to_string(),
            side: Side::Sell,
            price: 1,
            qty: 40,
            tif: Tif::Ioc,
            idempotency: None,
        },
    )?;

    Ok(())
}

fn place_internal(app: &AppState, o: NewOrder) -> Result<PlaceResult> {
    let market_id = o.market_id.clone();
    app.ledger.check_intent(
        o.user_id,
        o.side,
        o.qty,
        o.price as i64,
        &market_id,
        app.risk.caps(),
    )?;
    let outcome = app.engine.place_order(o)?;
    let fills = outcome.fills.clone();
    app.ledger.apply_fills(&fills);
    let trade_hint = fills.last().map(|f| (f.price, f.qty));
    if let Some((_, _, seq)) = app.engine.get_market_snapshot(&market_id, 20) {
        for f in &fills {
            app.md.publish_trade(&market_id, seq, f.price, f.qty);
        }
    }
    emit_book_update(app, &market_id, trade_hint);
    Ok(outcome)
}
