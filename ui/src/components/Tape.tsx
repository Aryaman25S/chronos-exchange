import { cn } from '@/lib/utils'

export function Tape({
  trades,
  emptyMessage = 'No trades yet',
}: {
  trades: { price: number; qty: number; seq: number }[]
  emptyMessage?: string
}) {
  return (
    <div className="rounded-xl border border-slate-800 bg-surface-muted/30 overflow-hidden">
      <div className="border-b border-slate-800 px-3 py-2">
        <h3 className="text-sm font-semibold text-slate-200">Tape</h3>
        <p className="text-[11px] text-slate-500">Recent prints (sim)</p>
      </div>
      <div
        className="max-h-56 overflow-y-auto text-sm font-mono"
        role="list"
        aria-label="Recent trades"
      >
        {trades.length === 0 ? (
          <p className="px-3 py-6 text-center text-slate-500 text-sm">{emptyMessage}</p>
        ) : (
          <ul className="divide-y divide-slate-800/80">
            {trades.map((t, i) => (
              <li
                key={`${t.seq}-${i}`}
                className="flex items-center justify-between gap-2 px-3 py-2 hover:bg-slate-900/50 transition-colors"
                role="listitem"
              >
                <span className={cn('font-medium tabular-nums', i === 0 && 'text-slate-100')}>
                  {(t.price / 100).toFixed(2)}
                </span>
                <span className="text-slate-400">×{t.qty}</span>
                <span className="text-[10px] text-slate-600">#{t.seq}</span>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  )
}
