/** Snapshot stats from the book + ledger (list markets). */
export type MarketStats = {
  volume_usd: number
  fill_count: number
  best_bid_cents: number | null
  best_ask_cents: number | null
  mid_cents: number | null
  /** Implied YES probability 0–100 from mid (or last trade if book empty). */
  yes_implied_pct: number | null
  last_trade_cents: number | null
}

export type Market = {
  id: string
  name: string
  tick_size: number
  description?: string
  tags?: string[]
  /** When present, market is settled: `true` = YES won, `false` = NO won */
  settled?: boolean | null
  stats?: MarketStats
}

export type OpenOrderRow = {
  market_id: string
  order_id: string
  side: 'Buy' | 'Sell'
  price: number
  qty: number
}

export type MarketPosition = {
  qty_yes: number
  avg_price_cents: number
}

export type FillRow = {
  market_id: string
  price: number
  qty: number
  buyer: string
  seller: string
}

export type PlaceOrderResponse = {
  fills: unknown[]
  status?: string
  self_trade_prevented?: boolean
  rested?: boolean
}
