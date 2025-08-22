
use axum::{routing::{get, post, delete}, Router, extract::{State, Path, Query, ws::{WebSocketUpgrade, Message}}, response::IntoResponse, Json};
use serde::Deserialize;
use std::{net::SocketAddr, path::PathBuf, time::Duration};
use tracing::info;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;
use engine::{Engine, types::*};
use risk::Risk;
use md::Broadcaster;

#[derive(Clone)]
struct AppState { engine: Engine, risk: Risk, md: Broadcaster }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env().add_directive("gateway=info".parse().unwrap())).init();
    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "./data_runtime".into());
    let engine = Engine::new(PathBuf::from(&data_dir))?;
    engine.ensure_market(Market { id: "OKC_WIN_YESNO".into(), name: "OKC wins YES/NO".into(), tick_size: 1 });
    engine.restore_from_latest()?;
    let risk = Risk::new(risk::Caps { max_position: 10_000, max_notional_cents: 1_000_000, rate_per_sec: 1000, burst: 2000 });
    let md = Broadcaster::new(10_000);

    // 20ms snapshot publisher
    let eng_clone = engine.clone(); let md_clone = md.clone();
    tokio::spawn(async move {
        loop {
            let book = { let inner = eng_clone.inner.lock(); inner.markets.get("OKC_WIN_YESNO").cloned() };
            if let Some(ob) = book { let l2 = ob.l2(20); let last = ob.state.last_trade; md_clone.publish_snapshot(ob.state.seq, l2, last); }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    });

    let state = AppState { engine, risk, md };
    let app = Router::new()
        .route("/v1/orders", post(place_order))
        .route("/v1/orders/:id", delete(cancel_order))
        .route("/v1/markets", get(list_markets))
        .route("/ws", get(ws_handler))
        .with_state(state.clone());
    let addr = SocketAddr::from(([0,0,0,0], 8081)); info!("gateway listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?; Ok(())
}

#[derive(Deserialize)]
struct PlaceReq { market_id: String, side: String, price: u32, qty: u32, tif: String, idempotency: Option<String> }
async fn place_order(State(app): State<AppState>, Json(r): Json<PlaceReq>) -> impl IntoResponse {
    let user = Uuid::new_v4();
    if let Some(key) = &r.idempotency { if let Err(e) = app.risk.check_idempotency(key) { return (axum::http::StatusCode::CONFLICT, e.to_string()); } }
    if let Err(e) = app.risk.check_rate_limit(user) { return (axum::http::StatusCode::TOO_MANY_REQUESTS, e.to_string()); }
    let side = if r.side.eq_ignore_ascii_case("buy") { Side::Buy } else { Side::Sell };
    let tif = match r.tif.as_str() { "IOC" => Tif::Ioc, "FOK" => Tif::Fok, _ => Tif::Gtc };
    if let Err(e) = app.risk.check_position(user, if matches!(side, Side::Buy) { r.qty as i64 } else { -(r.qty as i64) }, r.price as i64) { return (axum::http::StatusCode::FORBIDDEN, e.to_string()); }
    let o = NewOrder { id: Uuid::new_v4(), user_id: user, market_id: r.market_id, side, price: r.price, qty: r.qty, tif, idempotency: r.idempotency.clone() };
    match app.engine.place_order(o) { Ok(_) => (axum::http::StatusCode::OK, "ok".into()), Err(e) => (axum::http::StatusCode::BAD_REQUEST, e.to_string()) }
}

async fn cancel_order(State(app): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let oid = Uuid::parse_str(&id).unwrap_or(Uuid::nil());
    match app.engine.cancel_order("OKC_WIN_YESNO".into(), oid) { Ok(_) => (axum::http::StatusCode::OK, "ok".into()), Err(e) => (axum::http::StatusCode::BAD_REQUEST, e.to_string()) }
}
async fn list_markets() -> impl IntoResponse { Json(vec![engine::types::Market { id:"OKC_WIN_YESNO".into(), name:"OKC wins YES/NO".into(), tick_size:1 }]) }

#[derive(Deserialize)] struct WsQuery { market: Option<String>, from_seq: Option<u64> }
async fn ws_handler(State(app): State<AppState>, _q: axum::extract::Query<WsQuery>, ws: WebSocketUpgrade) -> impl IntoResponse { ws.on_upgrade(move |socket| handle_ws(app, socket)) }
async fn handle_ws(app: AppState, mut socket: axum::extract::ws::WebSocket) {
    let book = { let inner = app.engine.inner.lock(); inner.markets.get("OKC_WIN_YESNO").cloned() };
    if let Some(ob) = book {
        let l2 = ob.l2(20); let last = ob.state.last_trade;
        let snap = serde_json::json!({ "type":"snapshot","seq":ob.state.seq,"bids": l2.bids.into_iter().map(|l| [l.price,l.qty]).collect::<Vec<_>>(), "asks": l2.asks.into_iter().map(|l| [l.price,l.qty]).collect::<Vec<_>>(), "last_trade": last }).to_string();
        let _ = socket.send(Message::Text(snap)).await;
    }
    let mut rx = app.md.subscribe();
    loop { if let Ok(msg) = rx.recv().await { let _ = socket.send(Message::Text(msg)).await; } else { break; } }
}
