import { useEffect, useMemo, useRef, useState } from 'react'

export type Level = [number, number]

type WsMsg =
  | {
      type: 'snapshot'
      market_id?: string
      seq: number
      bids: Level[]
      asks: Level[]
      last_trade: number | null
    }
  | {
      type: 'delta'
      market_id?: string
      seq: number
      bids_upd: Level[]
      asks_upd: Level[]
      last_trade: [number, number] | null
    }
  | { type: 'trade'; market_id?: string; seq: number; price: number; qty: number }

function mergeL2(
  prev: { bids: Map<number, number>; asks: Map<number, number> },
  msg: WsMsg
): { bids: Map<number, number>; asks: Map<number, number> } {
  if (msg.type === 'snapshot') {
    const bids = new Map<number, number>()
    const asks = new Map<number, number>()
    for (const [p, q] of msg.bids) bids.set(p, q)
    for (const [p, q] of msg.asks) asks.set(p, q)
    return { bids, asks }
  }
  if (msg.type === 'delta') {
    const bids = new Map<number, number>()
    const asks = new Map<number, number>()
    for (const [p, q] of msg.bids_upd) bids.set(p, q)
    for (const [p, q] of msg.asks_upd) asks.set(p, q)
    return { bids, asks }
  }
  return prev
}

export type WsStatus = 'connecting' | 'open' | 'closed' | 'error'

export function useMarketWebSocket(
  url: string,
  resyncOnConnect: boolean,
  marketId: string
) {
  const [book, setBook] = useState<{ bids: Level[]; asks: Level[] }>({ bids: [], asks: [] })
  const [seq, setSeq] = useState(0)
  const seqRef = useRef(0)
  const [trades, setTrades] = useState<{ price: number; qty: number; seq: number }[]>([])
  const [lastTradePx, setLastTradePx] = useState<number | null>(null)
  const [status, setStatus] = useState<WsStatus>('connecting')

  const stableUrl = useMemo(() => url, [url])
  const mid = useMemo(() => marketId, [marketId])

  useEffect(() => {
    setStatus('connecting')
    const ws = new WebSocket(stableUrl)
    ws.onopen = () => {
      setStatus('open')
      if (resyncOnConnect && seqRef.current > 0) {
        ws.send(JSON.stringify({ type: 'resync', from_seq: seqRef.current }))
      }
    }
    ws.onerror = () => setStatus('error')
    ws.onclose = () => setStatus('closed')
    let maps = { bids: new Map<number, number>(), asks: new Map<number, number>() }
    ws.onmessage = (ev) => {
      const msg = JSON.parse(ev.data) as WsMsg
      if (msg.market_id != null && msg.market_id !== mid) {
        return
      }
      if (msg.type === 'trade') {
        setTrades((t) => [{ price: msg.price, qty: msg.qty, seq: msg.seq }, ...t].slice(0, 80))
        setLastTradePx(msg.price)
        return
      }
      maps = mergeL2(maps, msg)
      const bids: Level[] = [...maps.bids.entries()]
        .sort((a, b) => b[0] - a[0])
        .map(([p, q]) => [p, q])
      const asks: Level[] = [...maps.asks.entries()]
        .sort((a, b) => a[0] - b[0])
        .map(([p, q]) => [p, q])
      setBook({ bids, asks })
      if ('seq' in msg) {
        setSeq(msg.seq)
        seqRef.current = msg.seq
      }
      if (msg.type === 'snapshot' && msg.last_trade != null) {
        setLastTradePx(msg.last_trade)
      }
    }
    return () => {
      ws.close()
    }
  }, [stableUrl, resyncOnConnect, mid])

  return { book, seq, trades, lastTradePx, status }
}
