mod dummy_seed;
mod ledger;
mod metrics;

use axum::{
    body::Body,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use engine::types::*;
use engine::Engine;
use ledger::Ledger;
use md::{Broadcaster, MktDelta};
use metrics::Metrics;
use risk::Risk;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};
use tracing::info;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

const DEFAULT_MARKET: &str = "OKC_WIN_YESNO";

#[derive(Deserialize)]
struct MarketsSeedFile {
    markets: Vec<Market>,
}

fn load_markets_seed() -> Vec<Market> {
    let path = std::env::var("MARKETS_SEED").unwrap_or_else(|_| "markets_seed.json".into());
    let path = PathBuf::from(&path);
    if !path.exists() {
        info!("MARKETS_SEED not found at {}, using built-in default", path.display());
        return vec![default_primary_market()];
    }
    match std::fs::read_to_string(&path) {
        Ok(s) => match serde_json::from_str::<MarketsSeedFile>(&s) {
            Ok(f) if !f.markets.is_empty() => {
                info!("loaded {} markets from {}", f.markets.len(), path.display());
                f.markets
            }
            Ok(_) => {
                tracing::warn!("empty markets in {}, using default", path.display());
                vec![default_primary_market()]
            }
            Err(e) => {
                tracing::warn!("parse {}: {}, using default", path.display(), e);
                vec![default_primary_market()]
            }
        },
        Err(e) => {
            tracing::warn!("read {}: {}, using default", path.display(), e);
            vec![default_primary_market()]
        }
    }
}

fn default_primary_market() -> Market {
    Market {
        id: DEFAULT_MARKET.into(),
        name: "OKC wins YES/NO".into(),
        tick_size: 1,
        description: "Simulation contract; not real trading.".into(),
        tags: vec!["Demo".into(), "NBA".into()],
    }
}

#[derive(Clone)]
struct AppState {
    engine: Engine,
    risk: Arc<Risk>,
    ledger: Arc<Ledger>,
    md: Broadcaster,
    metrics: Arc<Metrics>,
}

pub(crate) fn emit_book_update(state: &AppState, market_id: &str, trade_hint: Option<(u32, u32)>) {
    let Some((l2, last_px, seq)) = state.engine.get_market_snapshot(market_id, 50) else {
        return;
    };
    let bids_upd: Vec<(u32, u64)> = l2.bids.iter().map(|l| (l.price, l.qty)).collect();
    let asks_upd: Vec<(u32, u64)> = l2.asks.iter().map(|l| (l.price, l.qty)).collect();
    let last_trade = trade_hint.or_else(|| last_px.map(|p| (p, 1)));
    state.md.publish_delta(&MktDelta {
        market_id: market_id.to_string(),
        seq,
        bids_upd,
        asks_upd,
        last_trade,
    });
}

fn user_from_headers(headers: &HeaderMap) -> UserId {
    headers
        .get("X-User-Id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or_else(Uuid::new_v4)
}

fn init_app_state(data_dir: PathBuf, catalog: &[Market]) -> anyhow::Result<AppState> {
    let engine = Engine::new(data_dir)?;
    for m in catalog {
        engine.ensure_market(m.clone());
    }
    engine.restore_from_latest()?;

    let risk = Arc::new(Risk::new(risk::Caps {
        max_position: 10_000,
        max_notional_cents: 1_000_000,
        rate_per_sec: 1000,
        burst: 2000,
    }));

    let md = Broadcaster::new(10_000);
    let ledger = Arc::new(Ledger::default());
    let metrics = Metrics::new()?;

    Ok(AppState {
        engine,
        risk,
        ledger,
        md,
        metrics,
    })
}

/// Returns `Router<()>` (state applied); required for [`axum::serve`].
fn make_app(state: AppState) -> Router {
    Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/v1/orders", get(list_open_orders).post(place_order))
        .route("/v1/orders/{id}", delete(cancel_order).put(replace_order))
        .route("/v1/markets", get(list_markets))
        .route("/v1/positions", get(get_positions))
        .route("/v1/fills", get(get_fills))
        .route("/v1/admin/markets/{market_id}/settle", post(admin_settle))
        .route("/ws", get(ws_handler_with_query))
        .with_state(state)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("gateway=info".parse().unwrap()),
        )
        .init();

    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "./data_runtime".into());
    let catalog = load_markets_seed();
    let state = init_app_state(PathBuf::from(&data_dir), &catalog)?;

    let eng_clone = state.engine.clone();
    let md_clone = state.md.clone();
    tokio::spawn(async move {
        loop {
            for (m, _) in eng_clone.list_markets_detail() {
                if let Some((l2, last, seq)) = eng_clone.get_market_snapshot(&m.id, 20) {
                    md_clone.publish_snapshot(&m.id, seq, l2, last);
                }
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    });

    let market_ids: Vec<String> = catalog.iter().map(|m| m.id.clone()).collect();
    dummy_seed::maybe_seed_dummy_trades(&state, std::path::Path::new(&data_dir), &market_ids)?;

    let app = make_app(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8081));
    info!("gateway listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

async fn metrics_handler(State(app): State<AppState>) -> Response<Body> {
    match app.metrics.encode() {
        Ok(s) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/plain; version=0.0.4")
            .body(Body::from(s))
            .unwrap(),
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(e.to_string()))
            .unwrap(),
    }
}

#[derive(Deserialize)]
struct PlaceReq {
    market_id: String,
    side: String,
    price: u32,
    qty: u32,
    tif: String,
    idempotency: Option<String>,
}

#[derive(Serialize)]
struct PlaceResp {
    fills: Vec<Fill>,
    status: &'static str,
    /// Incoming order canceled vs your own resting quote (no fill, nothing rests).
    self_trade_prevented: bool,
    /// Remaining size posted on the book (GTC).
    rested: bool,
}

async fn place_order(
    State(app): State<AppState>,
    headers: HeaderMap,
    Json(r): Json<PlaceReq>,
) -> impl IntoResponse {
    let user = user_from_headers(&headers);

    if let Some(key) = &r.idempotency {
        if let Err(e) = app.risk.check_idempotency(key) {
            app.metrics.orders_reject.inc();
            return (StatusCode::CONFLICT, e.to_string()).into_response();
        }
    }
    if let Err(e) = app.risk.check_rate_limit(user) {
        app.metrics.orders_reject.inc();
        return (StatusCode::TOO_MANY_REQUESTS, e.to_string()).into_response();
    }

    let side = if r.side.eq_ignore_ascii_case("buy") {
        Side::Buy
    } else {
        Side::Sell
    };
    let tif = match r.tif.as_str() {
        "IOC" => Tif::Ioc,
        "FOK" => Tif::Fok,
        _ => Tif::Gtc,
    };

    if let Err(e) = app.ledger.check_intent(
        user,
        side,
        r.qty,
        r.price as i64,
        &r.market_id,
        app.risk.caps(),
    ) {
        app.metrics.orders_reject.inc();
        return (StatusCode::FORBIDDEN, e.to_string()).into_response();
    }

    let o = NewOrder {
        id: Uuid::new_v4(),
        user_id: user,
        market_id: r.market_id.clone(),
        side,
        price: r.price,
        qty: r.qty,
        tif,
        idempotency: r.idempotency.clone(),
    };

    let start = std::time::Instant::now();
    let res = app.engine.place_order(o);
    let elapsed = start.elapsed().as_secs_f64();
    app.metrics.match_seconds.observe(elapsed);

    match res {
        Ok(outcome) => {
            app.metrics.orders_ok.inc();
            let fills = outcome.fills.clone();
            app.ledger.apply_fills(&fills);
            let trade_hint = fills.last().map(|f| (f.price, f.qty));
            if let Some((_, _, seq)) = app.engine.get_market_snapshot(&r.market_id, 20) {
                for f in &fills {
                    app.md.publish_trade(&r.market_id, seq, f.price, f.qty);
                }
            }
            emit_book_update(&app, &r.market_id, trade_hint);
            Json(PlaceResp {
                fills,
                status: "ok",
                self_trade_prevented: outcome.self_trade_prevented,
                rested: outcome.rested,
            })
            .into_response()
        }
        Err(e) => {
            app.metrics.orders_reject.inc();
            (StatusCode::BAD_REQUEST, e.to_string()).into_response()
        }
    }
}

#[derive(Deserialize)]
struct CancelQ {
    market_id: Option<String>,
}

async fn cancel_order(
    State(app): State<AppState>,
    Path(id): Path<String>,
    Query(q): Query<CancelQ>,
) -> impl IntoResponse {
    let oid = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "bad order id").into_response();
        }
    };
    let mid = q.market_id.unwrap_or_else(|| DEFAULT_MARKET.into());
    match app.engine.cancel_order(mid.clone(), oid) {
        Ok(_) => {
            emit_book_update(&app, &mid, None);
            (StatusCode::OK, "ok").into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

#[derive(Deserialize)]
struct ReplaceReq {
    market_id: String,
    new_price: Option<u32>,
    new_qty: Option<u32>,
}

async fn replace_order(
    State(app): State<AppState>,
    Path(id): Path<String>,
    Json(r): Json<ReplaceReq>,
) -> impl IntoResponse {
    let oid = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return (StatusCode::BAD_REQUEST, "bad order id").into_response(),
    };
    let rep = ReplaceOrder {
        market_id: r.market_id.clone(),
        order_id: oid,
        new_price: r.new_price,
        new_qty: r.new_qty,
    };
    match app.engine.replace_order(rep) {
        Ok(_) => {
            emit_book_update(&app, &r.market_id, None);
            (StatusCode::OK, "ok").into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

#[derive(Serialize)]
struct MarketStats {
    /// Simulated notional traded (dollars): sum over fills of (price/100)×qty.
    volume_usd: f64,
    fill_count: u64,
    best_bid_cents: Option<u32>,
    best_ask_cents: Option<u32>,
    /// Mid price in cents when both sides exist; else best bid or ask alone.
    mid_cents: Option<f64>,
    /// Implied YES probability (0–100), same as mid for a binary YES contract in cents.
    yes_implied_pct: Option<f64>,
    last_trade_cents: Option<u32>,
}

fn book_stats(engine: &Engine, market_id: &str) -> MarketStats {
    let Some((l2, last_trade, _seq)) = engine.get_market_snapshot(market_id, 50) else {
        return MarketStats {
            volume_usd: 0.0,
            fill_count: 0,
            best_bid_cents: None,
            best_ask_cents: None,
            mid_cents: None,
            yes_implied_pct: None,
            last_trade_cents: None,
        };
    };
    let bb = l2.bids.first().map(|l| l.price);
    let ba = l2.asks.first().map(|l| l.price);
    let (mid, yes) = match (bb, ba) {
        (Some(b), Some(a)) => {
            let m = (b + a) as f64 / 2.0;
            (Some(m), Some(m))
        }
        (Some(b), None) => (Some(b as f64), Some(b as f64)),
        (None, Some(a)) => (Some(a as f64), Some(a as f64)),
        (None, None) => (None, None),
    };
    let yes_implied = yes.or_else(|| last_trade.map(|p| p as f64));
    MarketStats {
        volume_usd: 0.0,
        fill_count: 0,
        best_bid_cents: bb,
        best_ask_cents: ba,
        mid_cents: mid,
        yes_implied_pct: yes_implied,
        last_trade_cents: last_trade,
    }
}

#[derive(Serialize)]
struct MarketRow {
    #[serde(flatten)]
    market: Market,
    settled: Option<bool>,
    stats: MarketStats,
}

async fn list_markets(State(app): State<AppState>) -> impl IntoResponse {
    let trade_by_m = app.ledger.per_market_trade_stats();
    let rows: Vec<MarketRow> = app
        .engine
        .list_markets_detail()
        .into_iter()
        .map(|(market, settled)| {
            let mut bs = book_stats(&app.engine, &market.id);
            let (vol, n) = trade_by_m
                .get(&market.id)
                .copied()
                .unwrap_or((0.0, 0));
            bs.volume_usd = vol;
            bs.fill_count = n as u64;
            MarketRow {
                market,
                settled,
                stats: bs,
            }
        })
        .collect();
    Json(rows)
}

#[derive(Deserialize)]
struct OrdersQ {
    market_id: Option<String>,
}

async fn list_open_orders(
    State(app): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<OrdersQ>,
) -> impl IntoResponse {
    let user = user_from_headers(&headers);
    Json(app.engine.resting_orders_for_user(user, q.market_id.as_deref()))
}

async fn get_positions(State(app): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let user = user_from_headers(&headers);
    Json(app.ledger.positions_for_user(user))
}

#[derive(Deserialize)]
struct FillsQ {
    limit: Option<usize>,
}

async fn get_fills(State(app): State<AppState>, Query(q): Query<FillsQ>) -> impl IntoResponse {
    let lim = q.limit.unwrap_or(50).min(500);
    Json(app.ledger.recent_fills(lim))
}

#[derive(Deserialize)]
struct SettleReq {
    resolve_yes: bool,
}

async fn admin_settle(
    State(app): State<AppState>,
    headers: HeaderMap,
    Path(market_id): Path<String>,
    Json(body): Json<SettleReq>,
) -> impl IntoResponse {
    let token = std::env::var("ADMIN_TOKEN").unwrap_or_default();
    let got = headers
        .get("X-Admin-Token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if token.is_empty() || got != token {
        return (StatusCode::UNAUTHORIZED, "unauthorized").into_response();
    }
    match app
        .engine
        .settle_market(market_id.clone(), body.resolve_yes)
    {
        Ok(_) => {
            app.ledger.apply_settlement(&market_id, body.resolve_yes);
            emit_book_update(&app, &market_id, None);
            (StatusCode::OK, "ok").into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

#[derive(Deserialize)]
struct WsConnQ {
    market_id: Option<String>,
}

async fn ws_handler_with_query(
    State(app): State<AppState>,
    Query(q): Query<WsConnQ>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let mid = q.market_id.unwrap_or_else(|| DEFAULT_MARKET.into());
    ws.on_upgrade(move |socket| handle_ws(app, socket, mid))
}

struct WsLive(Arc<Metrics>);
impl Drop for WsLive {
    fn drop(&mut self) {
        self.0.ws_clients.dec();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn get_markets_returns_json_array() {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("CHRONOS_SEED_DUMMY_TRADES", "0");
        let catalog = vec![default_primary_market()];
        let state = init_app_state(dir.path().to_path_buf(), &catalog).unwrap();
        let app = make_app(state);
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/v1/markets")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(v.is_array());
        assert!(!v.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn post_order_place_gtc() {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("CHRONOS_SEED_DUMMY_TRADES", "0");
        let catalog = vec![default_primary_market()];
        let state = init_app_state(dir.path().to_path_buf(), &catalog).unwrap();
        let app = make_app(state);
        let uid = Uuid::new_v4();
        let payload = serde_json::json!({
            "market_id": DEFAULT_MARKET,
            "side": "buy",
            "price": 45,
            "qty": 3,
            "tif": "GTC"
        });
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/orders")
                    .header("content-type", "application/json")
                    .header("X-User-Id", uid.to_string())
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["status"], "ok");
        assert_eq!(v["rested"], true);
    }
}

async fn handle_ws(app: AppState, mut socket: WebSocket, market_id: String) {
    app.metrics.ws_clients.inc();
    let _live = WsLive(app.metrics.clone());
    let maybe_first = tokio::time::timeout(Duration::from_millis(250), socket.recv()).await;
    let mut from_seq = 0u64;
    if let Ok(Some(Ok(Message::Text(t)))) = maybe_first {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&t) {
            if v.get("type").and_then(|x| x.as_str()) == Some("resync") {
                from_seq = v.get("from_seq").and_then(|x| x.as_u64()).unwrap_or(0);
            }
        }
    }

    if let Some((l2, last, seq)) = app.engine.get_market_snapshot(&market_id, 20) {
        let snap = json!({
            "type": "snapshot",
            "market_id": market_id,
            "seq": seq,
            "bids": l2.bids.into_iter().map(|l| (l.price, l.qty)).collect::<Vec<(u32,u64)>>(),
            "asks": l2.asks.into_iter().map(|l| (l.price, l.qty)).collect::<Vec<(u32,u64)>>(),
            "last_trade": last
        })
        .to_string();
        let _ = socket.send(Message::Text(snap.into())).await;
    }

    for msg in app.md.snapshot_from_seq(from_seq) {
        let _ = socket.send(Message::Text(msg.into())).await;
    }

    let mut rx = app.md.subscribe();
    loop {
        match rx.recv().await {
            Ok(msg) => {
                let _ = socket.send(Message::Text(msg.into())).await;
            }
            Err(_) => break,
        }
    }
}
