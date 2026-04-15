export type Market = {
  id: string
  name: string
  tick_size: number
  description?: string
  tags?: string[]
  /** When present, market is settled: `true` = YES won, `false` = NO won */
  settled?: boolean | null
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
