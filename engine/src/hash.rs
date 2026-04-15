use blake3::Hasher;
use std::collections::HashMap;
pub fn state_hash(markets: &HashMap<String, crate::orderbook::OrderBook>) -> [u8; 32] {
    let mut h = Hasher::new();
    let mut keys: Vec<_> = markets.keys().cloned().collect();
    keys.sort();
    for k in keys {
        let st = &markets.get(&k).unwrap().state;
        let bytes = bincode::serialize(st).unwrap();
        h.update(&bytes);
    }
    *h.finalize().as_bytes()
}
