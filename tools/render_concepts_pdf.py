#!/usr/bin/env python3
"""
Generate docs/Chronos_Orderbook_and_Engine_Concepts.pdf
Requires: pip install fpdf2 (see project .venv-pdf or: python3 -m venv .venv && pip install fpdf2)
"""
from __future__ import annotations

from pathlib import Path

from fpdf import FPDF


class Doc(FPDF):
    def __init__(self) -> None:
        super().__init__(format="A4")
        self.set_margins(18, 18, 18)

    def footer(self) -> None:
        self.set_y(-15)
        self.set_font("Helvetica", "I", 9)
        self.cell(0, 10, f"Page {self.page_no()}/{{nb}}", align="C")


def add_title(pdf: Doc, title: str, subtitle: str | None = None) -> None:
    pdf.set_x(pdf.l_margin)
    pdf.set_font("Helvetica", "B", 22)
    pdf.multi_cell(pdf.epw, 10, title)
    pdf.set_x(pdf.l_margin)
    pdf.ln(4)
    if subtitle:
        pdf.set_font("Helvetica", "", 12)
        pdf.set_text_color(60, 60, 60)
        pdf.multi_cell(pdf.epw, 6, subtitle)
        pdf.set_text_color(0, 0, 0)
        pdf.set_x(pdf.l_margin)
    pdf.ln(8)


def h1(pdf: Doc, text: str) -> None:
    pdf.ln(6)
    pdf.set_x(pdf.l_margin)
    pdf.set_font("Helvetica", "B", 16)
    pdf.set_fill_color(240, 240, 245)
    pdf.multi_cell(pdf.epw, 10, text, fill=True)
    pdf.set_x(pdf.l_margin)
    pdf.ln(4)


def h2(pdf: Doc, text: str) -> None:
    pdf.ln(5)
    pdf.set_x(pdf.l_margin)
    pdf.set_font("Helvetica", "B", 13)
    pdf.multi_cell(pdf.epw, 8, text)
    pdf.set_x(pdf.l_margin)
    pdf.ln(2)


def para(pdf: Doc, text: str) -> None:
    pdf.set_x(pdf.l_margin)
    pdf.set_font("Helvetica", "", 11)
    pdf.multi_cell(pdf.epw, 6, text)
    pdf.set_x(pdf.l_margin)
    pdf.ln(3)


def bullet(pdf: Doc, items: list[str]) -> None:
    pdf.set_font("Helvetica", "", 11)
    for it in items:
        pdf.set_x(pdf.l_margin)
        pdf.multi_cell(pdf.epw, 6, f"  - {it}")
    pdf.ln(2)


def build_pdf() -> Doc:
    pdf = Doc()
    pdf.set_auto_page_break(auto=True, margin=18)
    pdf.alias_nb_pages()
    pdf.add_page()

    add_title(
        pdf,
        "Order Books, Matching, and Chronos Exchange",
        "Concepts and technical glossary for reading this codebase",
    )

    h1(pdf, "1. What Chronos Exchange Is")
    para(
        pdf,
        "Chronos Exchange is a play-money simulation of event contracts (often framed as YES/NO "
        "binary outcomes). Users place limit-style orders at prices expressed in cents; those prices "
        "can be read as implied probabilities for the YES side. Nothing here involves real money.",
    )
    para(
        pdf,
        "The system has three main layers: (1) a Rust matching engine that owns the order book and "
        "durability, (2) an HTTP/WebSocket gateway that validates requests and streams market data, "
        "and (3) a React web UI for browsing markets and trading.",
    )

    h1(pdf, "2. Order Book Fundamentals")
    h2(pdf, "2.1 Bids and asks")
    para(
        pdf,
        "An order book lists resting orders: buyers submit bids (prices they will pay), sellers "
        "submit asks (prices they require). In Chronos, YES contracts are modeled with Buy YES "
        "and Sell YES sides. The book aggregates quantity at each price level.",
    )
    h2(pdf, "2.2 Best bid, best ask, spread, mid")
    bullet(
        pdf,
        [
            "Best bid: highest buy price with non-zero size.",
            "Best ask: lowest sell price with non-zero size.",
            "Spread: best ask minus best bid (when both exist). Tight spread means agreement is close.",
            "Mid: average of best bid and best ask; often shown as a reference price.",
        ],
    )
    h2(pdf, "2.3 Level 2 (L2) depth")
    para(
        pdf,
        "L2 is a list of price levels with aggregated quantity at each level (not every individual "
        "order ID). The UI depth ladder shows bids and asks as stacked levels. Chronos snapshots "
        "expose L2 to the WebSocket and REST stats.",
    )
    h2(pdf, "2.4 Tick size")
    para(
        pdf,
        "Prices must land on a discrete grid. Tick size is the minimum price increment between "
        "valid quotes. Each market in the seed catalog has a tick_size field (often 1 cent in this project).",
    )

    h1(pdf, "3. Matching Engine Concepts")
    h2(pdf, "3.1 Price-time priority")
    para(
        pdf,
        "When two orders could trade at the same price, the exchange typically matches the oldest "
        "resting order first (time priority at each price). New incoming orders walk the book until "
        "they are filled, canceled per rules, or rest.",
    )
    h2(pdf, "3.2 Maker vs taker")
    bullet(
        pdf,
        [
            "Maker: order that adds visible liquidity to the book (rests) or was already resting.",
            "Taker: incoming order that aggressively crosses the spread and executes against existing quotes.",
        ],
    )
    h2(pdf, "3.3 Fills")
    para(
        pdf,
        "A fill is an execution: a trade at a price and quantity between a buyer and a seller "
        "(identified by order IDs and user IDs). Multiple partial fills can happen from one incoming order.",
    )
    h2(pdf, "3.4 Time in force (TIF)")
    bullet(
        pdf,
        [
            "GTC (Good-Til-Canceled): rest any unfilled quantity on the book until filled or canceled.",
            "IOC (Immediate-Or-Cancel): cross what you can now; cancel any remainder. Nothing rests.",
            "FOK (Fill-Or-Kill): fill the entire size at available prices or reject the whole order "
            "with no partial execution and no book change.",
        ],
    )
    h2(pdf, "3.5 Self-trade prevention (STP)")
    para(
        pdf,
        "If your incoming order would only match against your own resting liquidity at the top of "
        "book, the engine cancels the incoming order instead of creating a wash trade. The API reports "
        "self_trade_prevented when that happens.",
    )
    h2(pdf, "3.6 Cancel and replace")
    para(
        pdf,
        "Resting orders can be canceled or replaced (price/quantity updates) without placing a brand "
        "new order ID in some systems; here the WAL records cancel/replace actions for recovery.",
    )

    h1(pdf, "4. Durability: WAL, Snapshots, Replay")
    h2(pdf, "4.1 Write-ahead log (WAL)")
    para(
        pdf,
        "A WAL is an append-only journal of everything that changes durable state: place order, "
        "cancel, replace, settle. Chronos serializes each record with bincode (binary serde) and "
        "writes length-prefixed entries to engine.wal under the data directory.",
    )
    para(
        pdf,
        "After a crash, the process rebuilds memory by replaying the WAL from the beginning: each "
        "record is deserialized and applied to the order book in order. That is why ordering and "
        "determinism matter.",
    )
    h2(pdf, "4.2 Snapshot")
    para(
        pdf,
        "A snapshot is a compressed point-in-time copy of book state so startup does not need to "
        "replay an extremely long WAL from zero. Chronos writes zstd-compressed snapshot files "
        "(snapshot-latest.bin.zst) containing serialized book state.",
    )
    h2(pdf, "4.3 Replay and the snapshot caveat (read the Architecture doc)")
    para(
        pdf,
        "In a production design, the snapshot usually stores the WAL offset it represents, and "
        "replay only applies WAL records after that offset. This project documents a caveat: if you "
        "both load a snapshot and replay the entire WAL, you can double-apply changes unless you "
        "manage truncation or offsets. For tests, WAL-only recovery is used to assert determinism.",
    )
    h2(pdf, "4.4 Determinism and state_hash")
    para(
        pdf,
        "Integration tests compare a state hash after a live run versus a recovery run from disk. "
        "Order timestamps are derived from the order UUID so replay matches continuous operation.",
    )

    h1(pdf, "5. Gateway, Ledger, and Risk")
    h2(pdf, "5.1 Axum (Rust web framework)")
    para(
        pdf,
        "Axum is the async HTTP router used for REST endpoints and WebSocket upgrade on /ws.",
    )
    h2(pdf, "5.2 REST API (summary)")
    bullet(
        pdf,
        [
            "POST /v1/orders: submit orders; returns fills, rested flag, self_trade_prevented.",
            "GET /v1/orders: list your open resting orders.",
            "DELETE/PUT /v1/orders/:id: cancel or replace.",
            "GET /v1/markets: catalog plus per-market stats (volume, best bid/ask, implied %, etc.).",
            "GET /v1/positions and /v1/fills: portfolio and history for the X-User-Id.",
            "POST /v1/admin/markets/{market_id}/settle: resolve a market (requires admin token).",
        ],
    )
    h2(pdf, "5.3 In-memory ledger")
    para(
        pdf,
        "The gateway maintains simulated positions and session statistics (volume, fill counts) "
        "in memory for API responses. This is separate from the engine book state persisted in WAL; "
        "stats reset on gateway restart unless you add separate persistence.",
    )
    h2(pdf, "5.4 Risk limits")
    para(
        pdf,
        "Per-user caps on position size and notional, plus token-bucket rate limiting and "
        "idempotency keys for order placement, reduce accidental spam and duplicate submits.",
    )
    h2(pdf, "5.5 Idempotency")
    para(
        pdf,
        "Clients may send an idempotency string with an order. The gateway rejects duplicates with "
        "the same key so retries do not double-place orders.",
    )

    h1(pdf, "6. Market Data and WebSocket")
    h2(pdf, "6.1 Broadcaster and ring buffer")
    para(
        pdf,
        "The md crate publishes JSON messages to WebSocket subscribers: periodic full L2 snapshots, "
        "deltas on changes, and per-trade events. A fixed-size ring stores recent (seq, json) pairs "
        "so a client that reconnects can resync from a past sequence number.",
    )
    h2(pdf, "6.2 seq and resync")
    para(
        pdf,
        "Each message carries a monotonic seq. The client may send {type: resync, from_seq: N} right "
        "after connect to receive buffered messages with seq greater than N before live stream.",
    )

    h1(pdf, "7. Observability and Ops")
    h2(pdf, "7.1 Prometheus metrics")
    para(
        pdf,
        "GET /metrics exposes Prometheus text format (counters/histograms) for order acceptance, "
        "rejections, and engine match latency. Grafana can chart these using the provided deploy config.",
    )
    h2(pdf, "7.2 Docker Compose")
    para(
        pdf,
        "deploy/docker-compose.yml builds the gateway image, runs the Vite dev UI, Prometheus, and "
        "Grafana. Environment variables such as DATA_DIR, MARKETS_SEED, and ADMIN_TOKEN configure runtime.",
    )

    h1(pdf, "8. Frontend Stack (UI)")
    bullet(
        pdf,
        [
            "Vite: fast dev server and bundler for the React app.",
            "React + React Router: pages for market list and per-market trading.",
            "TanStack Query: caches REST responses and refetches after mutations.",
            "WebSocket hook: subscribes to book updates for the active market.",
        ],
    )

    h1(pdf, "9. Libraries and repository layout")
    h2(pdf, "9.1 Rust workspace")
    para(
        pdf,
        "Cargo is Rust's build tool. This repo is a workspace with crates: engine (matching + WAL + "
        "snapshots), gateway (HTTP server), risk (limits and idempotency), md (market-data "
        "broadcast), and small tools (loadgen, replay_seeder, state_hash).",
    )
    h2(pdf, "9.2 Serialization and async")
    bullet(
        pdf,
        [
            "Serde: Rust derive macros to serialize/deserialize structs to bincode, JSON, etc.",
            "Bincode: compact binary encoding used for WAL records on disk.",
            "Tokio: async runtime used by the gateway for networking and timers.",
            "Axum: composable HTTP and WebSocket handlers on top of Tokio and hyper.",
        ],
    )
    h2(pdf, "9.3 UI and identity")
    bullet(
        pdf,
        [
            "TypeScript: typed JavaScript used in the ui/ package.",
            "X-User-Id: HTTP header carrying a UUID; stored in browser localStorage for stable sim identity.",
        ],
    )

    h1(pdf, "10. Glossary (project terms)")
    rows = [
        ("Axum", "Rust web framework providing REST routes and WebSocket upgrade."),
        ("bincode", "Binary serialization format used for WAL records and snapshots."),
        ("Broadcaster", "md module component that fans out JSON market data to WS clients."),
        ("Cargo / workspace", "Rust build system; a workspace links multiple crates in one repo."),
        ("Delta (WS)", "Incremental book update after a change, with seq and level updates."),
        ("Dummy seed", "Optional first-run synthetic trades to populate plausible spreads and volume on cards."),
        ("engine.wal", "Append-only journal file on disk under DATA_DIR."),
        ("Fill", "Executed trade between maker and taker orders."),
        ("FOK / GTC / IOC", "Time-in-force modes; see section 3.4."),
        ("Grafana", "Dashboards for Prometheus metrics (used in docker compose)."),
        ("JSON", "Text format for WebSocket payloads and many REST responses."),
        ("L2", "Depth: bids and asks as price levels with sizes."),
        ("Ledger (gateway)", "In-memory positions and session stats, not the engine WAL."),
        ("Maker / Taker", "Liquidity provider vs aggressive order; see section 3.2."),
        ("MARKETS_SEED", "Path to JSON catalog of markets loaded at gateway startup."),
        ("Prometheus", "Metrics scraper; polls /metrics."),
        ("React", "UI library for components and state; used with Vite."),
        ("REST", "HTTP JSON APIs under /v1/..."),
        ("Resync", "Client request to replay ring buffer from a prior seq."),
        ("Serde", "Rust serialization framework; derives Serialize/Deserialize."),
        ("Self-trade prevention (STP)", "Blocks matching against your own resting quote at the top."),
        ("seq", "Sequence number on streamed messages for ordering and gap recovery."),
        ("Snapshot", "Compressed frozen book state for faster startup."),
        ("Spread", "Difference between best ask and best bid."),
        ("STP", "See Self-trade prevention."),
        ("TanStack Query", "React hooks for server state caching and invalidation."),
        ("Tick size", "Minimum price increment for a market."),
        ("Tokio", "Async runtime for Rust (tasks, timers, TCP)."),
        ("UUID", "Universally unique identifier; used for users and order IDs."),
        ("Vite", "Frontend toolchain: dev server with proxy to the gateway."),
        ("WAL", "Write-ahead log of durable engine actions."),
        ("WebSocket /ws", "Streaming market data and trades for a selected market_id."),
        ("zstd", "Compression algorithm used for snapshot files."),
    ]
    pdf.set_font("Helvetica", "", 10)
    for term, defin in rows:
        pdf.set_x(pdf.l_margin)
        pdf.set_font("Helvetica", "B", 10)
        pdf.multi_cell(pdf.epw, 5, term)
        pdf.set_x(pdf.l_margin)
        pdf.set_font("Helvetica", "", 10)
        pdf.multi_cell(pdf.epw, 5, defin)
        pdf.ln(2)

    h1(pdf, "11. Further reading")
    para(
        pdf,
        "See docs/Architecture.md and docs/APIs.md in the repository for exact behavior, headers "
        "(X-User-Id, X-Admin-Token), and endpoint details.",
    )

    return pdf


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    out = root / "docs" / "Chronos_Orderbook_and_Engine_Concepts.pdf"
    pdf = build_pdf()
    pdf.output(str(out))
    print(f"Wrote {out}")


if __name__ == "__main__":
    main()
