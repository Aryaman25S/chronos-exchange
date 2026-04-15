#!/usr/bin/env python3
"""
Generate docs/Demo_Mock_Interview_Walkthrough.pdf (gitignored; run locally).
Requires: pip install fpdf2 (e.g. python3 -m venv .venv-pdf && pip install fpdf2)
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
        "Mock interview demo (UI walkthrough)",
        "Chronos Exchange: what to show and what to point at so the system looks real",
    )

    para(
        pdf,
        "Goal: in ~3 to 5 minutes, prove that clicks in the UI drive a real order book: resting "
        "liquidity, matching, fills, and session stats. Everything below is observable in the app "
        "when the gateway and UI are running.",
    )

    h1(pdf, "0. Setup (15 seconds)")
    bullet(
        pdf,
        [
            "Open a market from Browse (e.g. OKC_WIN_YESNO) and land on the trading view.",
            "Mention that identity is X-User-Id (stored in localStorage); same browser session keeps one sim user.",
        ],
    )

    h1(pdf, "1. Depth ladder: resting bids and asks")
    para(
        pdf,
        "Place orders that REST (GTC) so they stay on the book instead of instantly trading.",
    )
    h2(pdf, "Do")
    bullet(
        pdf,
        [
            "Buy YES at a price clearly below the current best ask (bid-side liquidity).",
            "Sell YES at a price clearly above the current best bid (ask-side liquidity).",
        ],
    )
    h2(pdf, "Point to")
    bullet(
        pdf,
        [
            "The depth ladder: new price levels with non-zero size on bids and asks.",
            "Optional: cancel one resting order and watch that level shrink or disappear; replace price and watch the level move.",
        ],
    )
    h2(pdf, "Why it matters")
    para(
        pdf,
        "The ladder is live L2 from the engine via WebSocket. If your actions did nothing, levels would not stick.",
    )

    h1(pdf, "2. Mid, spread, implied YES %")
    h2(pdf, "Do")
    para(
        pdf,
        "Improve the top of book: e.g. raise your best bid or lower your best resting ask (without crossing yet).",
    )
    h2(pdf, "Point to")
    bullet(
        pdf,
            [
            "Mid and spread (and YES implied % if shown) updating as the top of book changes.",
        ],
    )

    h1(pdf, "3. Tape and Activity: prove a match happened")
    h2(pdf, "Do")
    para(
        pdf,
        "Cross the spread: buy at or above the best ask, or sell at or below the best bid, so an immediate trade occurs.",
    )
    h2(pdf, "Point to")
    bullet(
        pdf,
            [
                "Tape: a new trade line at the fill price and size.",
                "Activity: consistent narrative of what happened.",
            ],
    )
    h2(pdf, "Why it matters")
    para(
        pdf,
        "Fills come from the matcher after Wal-backed placement; tape/trade events are published from the gateway path.",
    )

    h1(pdf, "4. Positions and open orders")
    h2(pdf, "Point to")
    bullet(
        pdf,
            [
                "Positions: quantity and average price change after fills.",
                "Open orders: your resting GTC orders listed; cancel and see them leave; replace and see price/qty change.",
            ],
    )

    h1(pdf, "5. Volume and session stats")
    h2(pdf, "Point to")
    bullet(
        pdf,
            [
                "On the market view (and cards on Browse): volume and trade count ticking up after your fills.",
            ],
    )
    para(
        pdf,
        "Session stats come from the in-memory ledger in the gateway, so they move when real fills occur.",
    )

    h1(pdf, "6. Optional: developer trust line")
    para(
        pdf,
        "If asked how you know it is not fake front-end state: offer to open DevTools, Network, and show "
        "POST /v1/orders responses and the open WebSocket receiving snapshot/delta/trade frames. The UI cues "
        "above are usually enough for a short demo.",
    )

    h1(pdf, "7. Minimum credible story (one sentence)")
    para(
        pdf,
        "Rest two-sided liquidity on the ladder, then cross once and watch tape, positions, and volume move together.",
    )

    return pdf


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    out = root / "docs" / "Demo_Mock_Interview_Walkthrough.pdf"
    pdf = build_pdf()
    pdf.output(str(out))
    print(f"Wrote {out}")


if __name__ == "__main__":
    main()
