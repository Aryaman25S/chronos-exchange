import type { FillRow } from '@/api/types'

export function ActivityFeed({
  fills,
  isLoading,
  error,
}: {
  fills: FillRow[] | undefined
  isLoading: boolean
  error: unknown
}) {
  const rows = fills ?? []

  return (
    <div className="rounded-xl border border-slate-800 bg-surface-muted/30 overflow-hidden">
      <div className="border-b border-slate-800 px-3 py-2">
        <h3 className="text-sm font-semibold text-slate-200">Activity</h3>
        <p className="text-[11px] text-slate-500">Recent fills (all users, sim)</p>
      </div>
      <div className="max-h-48 overflow-y-auto p-2 text-xs font-mono">
        {isLoading && <p className="text-slate-500 px-1 py-2">Loading…</p>}
        {error != null && (
          <p className="text-red-400 px-1 py-2" role="alert">
            Could not load fills.
          </p>
        )}
        {!isLoading && error == null && rows.length === 0 && (
          <p className="text-slate-500 px-1 py-2">No fills yet.</p>
        )}
        {!isLoading &&
          error == null &&
          rows.map((f, i) => (
            <div
              key={`${f.market_id}-${f.price}-${i}`}
              className="flex justify-between gap-2 border-b border-slate-800/50 py-1.5 last:border-0"
            >
              <span className="text-slate-400 truncate">{f.market_id}</span>
              <span>
                {(f.price / 100).toFixed(2)} ×{f.qty}
              </span>
            </div>
          ))}
      </div>
    </div>
  )
}
