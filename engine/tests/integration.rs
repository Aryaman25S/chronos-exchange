use engine::types::*;
use engine::Engine;
use tempfile::tempdir;
use uuid::Uuid;

fn mk_m() -> Market {
    Market {
        id: "T".into(),
        name: "Test".into(),
        tick_size: 1,
        description: String::new(),
        tags: vec![],
    }
}

#[test]
fn wal_replay_matches_continuous_state_hash() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("data");
    let h1 = {
        let e = Engine::new(path.clone()).unwrap();
        e.ensure_market(mk_m());
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        e.place_order(NewOrder {
            id: Uuid::new_v4(),
            user_id: a,
            market_id: "T".into(),
            side: Side::Sell,
            price: 50,
            qty: 10,
            tif: Tif::Gtc,
            idempotency: None,
        })
        .unwrap();
        e.place_order(NewOrder {
            id: Uuid::new_v4(),
            user_id: b,
            market_id: "T".into(),
            side: Side::Buy,
            price: 50,
            qty: 4,
            tif: Tif::Gtc,
            idempotency: None,
        })
        .unwrap();
        e.state_hash().unwrap()
    };

    let e2 = Engine::new(path).unwrap();
    e2.ensure_market(mk_m());
    e2.restore_from_latest().unwrap();
    let h2 = e2.state_hash().unwrap();
    assert_eq!(h1, h2, "deterministic replay should match state hash");
}

#[test]
fn fok_rejects_when_not_fully_fillable() {
    let mut ob = engine::orderbook::OrderBook::new(mk_m());
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    ob.place(NewOrder {
        id: Uuid::new_v4(),
        user_id: a,
        market_id: "T".into(),
        side: Side::Sell,
        price: 50,
        qty: 5,
        tif: Tif::Gtc,
        idempotency: None,
    })
    .unwrap();
    let r = ob
        .place(NewOrder {
            id: Uuid::new_v4(),
            user_id: b,
            market_id: "T".into(),
            side: Side::Buy,
            price: 50,
            qty: 10,
            tif: Tif::Fok,
            idempotency: None,
        })
        .unwrap();
    assert!(r.fills.is_empty());
    assert_eq!(ob.state.bids.len() + ob.state.asks.len(), 1);
}

#[test]
fn fok_fills_when_book_has_liquidity() {
    let mut ob = engine::orderbook::OrderBook::new(mk_m());
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    ob.place(NewOrder {
        id: Uuid::new_v4(),
        user_id: a,
        market_id: "T".into(),
        side: Side::Sell,
        price: 50,
        qty: 10,
        tif: Tif::Gtc,
        idempotency: None,
    })
    .unwrap();
    let r = ob
        .place(NewOrder {
            id: Uuid::new_v4(),
            user_id: b,
            market_id: "T".into(),
            side: Side::Buy,
            price: 50,
            qty: 10,
            tif: Tif::Fok,
            idempotency: None,
        })
        .unwrap();
    assert_eq!(r.fills.len(), 1);
    assert_eq!(r.fills[0].qty, 10);
}

#[test]
fn self_trade_prevents_crossing_own_quote() {
    let mut ob = engine::orderbook::OrderBook::new(mk_m());
    let u = Uuid::new_v4();
    ob.place(NewOrder {
        id: Uuid::new_v4(),
        user_id: u,
        market_id: "T".into(),
        side: Side::Buy,
        price: 50,
        qty: 10,
        tif: Tif::Gtc,
        idempotency: None,
    })
    .unwrap();
    let r = ob
        .place(NewOrder {
            id: Uuid::new_v4(),
            user_id: u,
            market_id: "T".into(),
            side: Side::Sell,
            price: 50,
            qty: 2,
            tif: Tif::Gtc,
            idempotency: None,
        })
        .unwrap();
    assert!(r.fills.is_empty());
    assert!(r.self_trade_prevented);
    assert!(!r.rested);
}
