
use engine::types::*;
use engine::orderbook::OrderBook;
use proptest::prelude::*;
use uuid::Uuid;

fn mk_market() -> Market { Market { id: "TEST".into(), name: "Test".into(), tick_size: 1 } }

proptest! {
    #[test]
    fn no_crossed_book(seq in prop::collection::vec(0u8..100, 1..100)) {
        let mut ob = OrderBook::new(mk_market());
        for x in seq {
            let id = Uuid::new_v4();
            if x % 3 == 0 {
                let side = if x % 2 == 0 { Side::Buy } else { Side::Sell };
                let price = 30 + (x as u32 % 40);
                let _ = ob.place(NewOrder { id, user_id: Uuid::new_v4(), market_id: "TEST".into(), side, price, qty: 10, tif: Tif::Gtc, idempotency: None });
            } else { let _ = ob.cancel(Uuid::new_v4()); }
            let bb = ob.state.bids.keys().rev().next().cloned();
            let ba = ob.state.asks.keys().next().cloned();
            if let (Some(b), Some(a)) = (bb, ba) { prop_assert!(b < a, "crossed book: bid {} >= ask {}", b, a); }
        }
    }
}
