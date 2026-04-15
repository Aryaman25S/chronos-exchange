import type { MarketPosition } from '@/api/types'

export function PortfolioPanel({
  positions,
  isLoading,
  error,
}: {
  positions: Record<string, MarketPosition> | undefined
  isLoading: boolean
  error: unknown
}) {
  const entries = positions ? Object.entries(positions) : []

  return (
    <div className="rounded-xl border border-slate-800 bg-surface-muted/30 overflow-hidden">
      <div className="border-b border-slate-800 px-3 py-2">
        <h3 className="text-sm font-semibold text-slate-200">Positions</h3>
        <p className="text-[11px] text-slate-500">
          Net YES contracts (sim). Mark-to-mid is indicative; final payoff follows settlement.
        </p>
      </div>
      <div className="p-3">
        {isLoading && <p className="text-sm text-slate-500">Loading…</p>}
        {error != null && (
          <p className="text-sm text-red-400" role="alert">
            Could not load positions.
          </p>
        )}
        {!isLoading && error == null && entries.length === 0 && (
          <p className="text-sm text-slate-500 py-2">No open positions yet.</p>
        )}
        {!isLoading && error == null && entries.length > 0 && (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="text-left text-[11px] uppercase tracking-wide text-slate-500 border-b border-slate-800">
                  <th className="pb-2 pr-2">Market</th>
                  <th className="pb-2 pr-2">Qty</th>
                  <th className="pb-2">Avg ¢</th>
                </tr>
              </thead>
              <tbody>
                {entries.map(([id, p]) => (
                  <tr key={id} className="border-b border-slate-800/60 last:border-0">
                    <td className="py-2 pr-2 font-mono text-xs text-slate-300 truncate max-w-[140px]">
                      {id}
                    </td>
                    <td className="py-2 pr-2 tabular-nums">{p.qty_yes}</td>
                    <td className="py-2 tabular-nums text-slate-400">{p.avg_price_cents}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  )
}
