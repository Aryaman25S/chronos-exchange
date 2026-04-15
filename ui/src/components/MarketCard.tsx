import { ArrowRight } from 'lucide-react'
import { Link } from 'react-router-dom'
import type { Market } from '@/api/types'
import { formatVolUsd } from '@/lib/formatVol'
import { cn } from '@/lib/utils'

function primaryCategory(tags: string[] | undefined): string | null {
  if (!tags?.length) return null
  return tags[0].toUpperCase()
}

export function MarketCard({ m }: { m: Market }) {
  const s = m.stats
  const settled = m.settled
  const yesPct =
    settled === true ? 100 : settled === false ? 0 : s?.yes_implied_pct ?? null
  const noPct = yesPct != null ? Math.max(0, Math.min(100, 100 - yesPct)) : null
  const spread =
    s?.best_bid_cents != null && s?.best_ask_cents != null
      ? s.best_ask_cents - s.best_bid_cents
      : null
  const cat = primaryCategory(m.tags)

  return (
    <li>
      <Link
        to={`/m/${encodeURIComponent(m.id)}`}
        className={cn(
          'group flex flex-col rounded-xl border border-slate-800/90 bg-gradient-to-b from-slate-900/80 to-surface-muted/50 p-0 overflow-hidden transition-all',
          'hover:border-slate-600 hover:shadow-lg hover:shadow-black/20 focus:outline-none focus:ring-2 focus:ring-accent h-full',
          'border-t-[3px] border-t-accent/40'
        )}
      >
        <div className="px-4 pt-3 pb-2 flex items-center justify-between gap-2 text-[10px] uppercase tracking-wider text-slate-500">
          <span className="flex items-center gap-1.5 min-w-0">
            {cat ? (
              <span className="text-slate-400 font-semibold truncate">{cat}</span>
            ) : (
              <span className="text-slate-500">Market</span>
            )}
          </span>
          <span className="shrink-0 flex items-center gap-2">
            {settled == null ? (
              <span className="inline-flex items-center gap-1 rounded-full bg-emerald-500/15 px-2 py-0.5 text-emerald-400 font-medium">
                <span className="h-1.5 w-1.5 rounded-full bg-emerald-400 animate-pulse" aria-hidden />
                Sim
              </span>
            ) : (
              <span className="rounded-full bg-amber-500/15 px-2 py-0.5 text-amber-200 font-medium">
                Resolved
              </span>
            )}
            <span className="text-slate-600 tabular-nums">{m.tick_size}¢ tick</span>
          </span>
        </div>

        <div className="px-4 pb-3">
          <h2 className="font-semibold text-slate-50 group-hover:text-white leading-snug line-clamp-2 text-[15px]">
            {m.name}
          </h2>
          {m.description ? (
            <p className="mt-1.5 text-xs text-slate-500 line-clamp-2">{m.description}</p>
          ) : null}
        </div>

        {settled == null && (
          <div className="px-4 pb-3">
            {yesPct != null && noPct != null ? (
              <>
                <div className="grid grid-cols-2 gap-2">
                  <div className="rounded-lg bg-emerald-950/50 border border-emerald-800/40 px-3 py-2">
                    <div className="text-[10px] font-medium uppercase tracking-wide text-emerald-400/90">
                      Yes
                    </div>
                    <div className="text-xl font-semibold tabular-nums text-emerald-100">
                      {yesPct.toFixed(0)}%
                    </div>
                  </div>
                  <div className="rounded-lg bg-rose-950/50 border border-rose-800/40 px-3 py-2">
                    <div className="text-[10px] font-medium uppercase tracking-wide text-rose-400/90">
                      No
                    </div>
                    <div className="text-xl font-semibold tabular-nums text-rose-100">
                      {noPct.toFixed(0)}%
                    </div>
                  </div>
                </div>
                <div
                  className="mt-2 flex h-1.5 w-full overflow-hidden rounded-full bg-slate-800"
                  aria-hidden
                >
                  <div className="bg-emerald-500/90" style={{ width: `${yesPct}%` }} />
                  <div className="bg-rose-500/90" style={{ width: `${noPct}%` }} />
                </div>
                {spread != null && spread > 0 ? (
                  <p className="mt-1.5 text-[11px] text-slate-500">
                    Spread {spread}¢ · Mid {s?.mid_cents != null ? `${s.mid_cents.toFixed(1)}¢` : '—'}
                  </p>
                ) : (
                  <p className="mt-1.5 text-[11px] text-slate-500">
                    {s?.last_trade_cents != null
                      ? `Last ${s.last_trade_cents}¢`
                      : 'No trades yet'}
                  </p>
                )}
              </>
            ) : (
              <p className="text-xs text-slate-500">Waiting for quotes…</p>
            )}
          </div>
        )}

        {settled != null && (
          <div className="px-4 pb-3">
            <p className="rounded-lg border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-sm text-amber-100">
              Outcome: <strong>{settled ? 'YES' : 'NO'}</strong> — book closed
            </p>
          </div>
        )}

        <div className="mt-auto flex items-center justify-between gap-2 border-t border-slate-800/80 px-4 py-2.5 bg-slate-950/40">
          <div className="text-[11px] text-slate-500">
            <span className="text-slate-400 font-medium tabular-nums">
              {formatVolUsd(s?.volume_usd ?? 0)}
            </span>
            <span className="text-slate-600"> vol</span>
            {s != null && s.fill_count > 0 ? (
              <span className="text-slate-600"> · {s.fill_count} trades</span>
            ) : null}
          </div>
          <span className="inline-flex items-center gap-1 text-xs font-medium text-accent">
            {settled != null ? 'Details' : 'Trade'}
            <ArrowRight className="h-3.5 w-3.5 transition-transform group-hover:translate-x-0.5" />
          </span>
        </div>
      </Link>
    </li>
  )
}
