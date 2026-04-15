import { cn } from '@/lib/utils'
import type { Level } from '@/hooks/useMarketWebSocket'

const ROWS = 12

function computeMid(bids: Level[], asks: Level[]): { mid: number; spread: number } | null {
  if (!bids.length || !asks.length) return null
  const bestBid = bids[0][0]
  const bestAsk = asks[0][0]
  if (bestAsk <= bestBid) return null
  return { mid: (bestBid + bestAsk) / 2, spread: bestAsk - bestBid }
}

export function DepthLadder({
  bids,
  asks,
  lastTradePx,
}: {
  bids: Level[]
  asks: Level[]
  lastTradePx: number | null
}) {
  const askRows = asks.slice(0, ROWS).reverse()
  const bidRows = bids.slice(0, ROWS)
  const midInfo = computeMid(bids, asks)
  const allQty = [...askRows, ...bidRows].map(([, q]) => q)
  const maxQ = Math.max(1, ...allQty)

  const spreadDollars = midInfo ? midInfo.spread / 100 : null

  return (
    <div className="rounded-xl border border-slate-800 bg-surface-muted/40 overflow-hidden">
      <div className="grid grid-cols-2 gap-px bg-slate-800/80 text-[11px] uppercase tracking-wide text-slate-500 px-3 py-2">
        <span>Price</span>
        <span className="text-right">Size</span>
      </div>
      <div className="divide-y divide-slate-800/60">
        {askRows.length === 0 && bidRows.length === 0 ? (
          <p className="px-3 py-8 text-center text-sm text-slate-500">Waiting for book data…</p>
        ) : (
          <>
            {askRows.map(([p, q], i) => (
              <BookRow key={`a-${p}-${i}`} price={p} qty={q} side="ask" maxQ={maxQ} />
            ))}
            <div className="bg-slate-900/90 px-3 py-3 text-center border-y border-slate-700/80">
              {midInfo ? (
                <>
                  <div className="text-[10px] uppercase tracking-wider text-slate-500">Mid / implied</div>
                  <div className="text-lg font-semibold tabular-nums text-slate-100">
                    {(midInfo.mid / 100).toFixed(2)}
                    <span className="text-slate-400 text-sm font-normal ml-2">
                      {midInfo.mid.toFixed(1)}% implied
                    </span>
                  </div>
                  {spreadDollars != null && (
                    <div className="text-xs text-slate-500 mt-1">
                      Spread ${spreadDollars.toFixed(2)}
                    </div>
                  )}
                </>
              ) : (
                <div className="text-sm text-slate-500">
                  {lastTradePx != null ? (
                    <>
                      Last {(lastTradePx / 100).toFixed(2)}{' '}
                      <span className="text-slate-600">(one-sided book)</span>
                    </>
                  ) : (
                    'No crossing book yet'
                  )}
                </div>
              )}
            </div>
            {bidRows.map(([p, q], i) => (
              <BookRow key={`b-${p}-${i}`} price={p} qty={q} side="bid" maxQ={maxQ} />
            ))}
          </>
        )}
      </div>
    </div>
  )
}

function BookRow({
  price,
  qty,
  side,
  maxQ,
}: {
  price: number
  qty: number
  side: 'bid' | 'ask'
  maxQ: number
}) {
  const w = Math.min(100, (qty / maxQ) * 100)
  return (
    <div className="relative grid grid-cols-2 gap-2 px-3 py-1.5 text-sm tabular-nums">
      <span
        className={cn(
          'relative z-10',
          side === 'bid' ? 'text-bid-fg' : 'text-ask-fg'
        )}
      >
        {(price / 100).toFixed(2)}
      </span>
      <span className="relative z-10 text-right text-slate-300">{qty}</span>
      <div
        className={cn(
          'absolute inset-y-0 right-0 opacity-25',
          side === 'bid' ? 'bg-bid' : 'bg-ask'
        )}
        style={{ width: `${w}%` }}
        aria-hidden
      />
    </div>
  )
}
