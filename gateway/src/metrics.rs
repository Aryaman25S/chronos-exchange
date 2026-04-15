use prometheus::{Encoder, Histogram, HistogramOpts, IntCounter, IntGauge, Registry, TextEncoder};
use std::sync::Arc;

pub struct Metrics {
    registry: Registry,
    pub orders_ok: IntCounter,
    pub orders_reject: IntCounter,
    pub match_seconds: Histogram,
    pub ws_clients: IntGauge,
}

impl Metrics {
    pub fn new() -> anyhow::Result<Arc<Self>> {
        let registry = Registry::new();
        let orders_ok = IntCounter::new(
            "chronos_orders_accepted_total",
            "Orders accepted and sent to engine",
        )?;
        let orders_reject = IntCounter::new(
            "chronos_orders_rejected_total",
            "Orders rejected (risk or engine error)",
        )?;
        let match_seconds = Histogram::with_opts(
            HistogramOpts::new(
                "chronos_gateway_engine_match_seconds",
                "Wall time for engine.place_order",
            )
            .buckets(vec![
                1e-6, 5e-6, 1e-5, 5e-5, 1e-4, 5e-4, 1e-3, 5e-3, 0.01, 0.05, 0.1,
            ]),
        )?;
        let ws_clients = IntGauge::new(
            "chronos_ws_connected_clients",
            "WebSocket connections currently active",
        )?;
        registry.register(Box::new(orders_ok.clone()))?;
        registry.register(Box::new(orders_reject.clone()))?;
        registry.register(Box::new(match_seconds.clone()))?;
        registry.register(Box::new(ws_clients.clone()))?;
        Ok(Arc::new(Self {
            registry,
            orders_ok,
            orders_reject,
            match_seconds,
            ws_clients,
        }))
    }

    pub fn encode(&self) -> anyhow::Result<String> {
        let encoder = TextEncoder::new();
        let mut buf = Vec::new();
        encoder.encode(&self.registry.gather(), &mut buf)?;
        Ok(String::from_utf8(buf)?)
    }
}
