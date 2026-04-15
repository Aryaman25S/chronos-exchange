import { useQuery } from '@tanstack/react-query'
import { ArrowRight, Loader2 } from 'lucide-react'
import { useMemo, useState } from 'react'
import { Link } from 'react-router-dom'
import { marketsQueryOptions } from '@/api/queries'
import { Badge } from '@/components/ui/badge'

export function MarketsPage() {
  const { data, isLoading, error } = useQuery(marketsQueryOptions())
  const [q, setQ] = useState('')
  const [tag, setTag] = useState<string | null>(null)

  const allTags = useMemo(() => {
    const s = new Set<string>()
    for (const m of data ?? []) {
      for (const t of m.tags ?? []) s.add(t)
    }
    return [...s].sort((a, b) => a.localeCompare(b))
  }, [data])

  const filtered = useMemo(() => {
    let rows = data ?? []
    const qq = q.trim().toLowerCase()
    if (qq) {
      rows = rows.filter(
        (m) =>
          m.name.toLowerCase().includes(qq) ||
          m.id.toLowerCase().includes(qq) ||
          (m.description?.toLowerCase().includes(qq) ?? false)
      )
    }
    if (tag) {
      rows = rows.filter((m) => m.tags?.includes(tag))
    }
    return [...rows].sort((a, b) => a.name.localeCompare(b.name))
  }, [data, q, tag])

  return (
    <div className="space-y-8 animate-fade-in">
      <div className="space-y-2">
        <h1 className="text-2xl font-bold tracking-tight text-slate-50 sm:text-3xl">Markets</h1>
        <p className="text-slate-400 max-w-xl text-sm sm:text-base">
          Simulation-only prediction contracts. Pick a market to view depth, tape, and place limit
          orders.
        </p>
      </div>

      {!isLoading && !error && (data?.length ?? 0) > 0 && (
        <div className="flex flex-col gap-3 sm:flex-row sm:items-end sm:flex-wrap">
          <label className="flex flex-col gap-1 text-xs text-slate-500 max-w-md flex-1 min-w-[200px]">
            Search
            <input
              type="search"
              value={q}
              onChange={(e) => setQ(e.target.value)}
              placeholder="Name, id, or description"
              className="rounded-lg border border-slate-700 bg-surface-muted/50 px-3 py-2 text-sm text-slate-100 placeholder:text-slate-600 focus:outline-none focus:ring-2 focus:ring-accent"
            />
          </label>
          {allTags.length > 0 && (
            <div className="flex flex-wrap gap-2 items-center">
              <span className="text-xs text-slate-500">Tags</span>
              <button
                type="button"
                onClick={() => setTag(null)}
                className={`rounded-full border px-2.5 py-0.5 text-xs ${
                  tag === null
                    ? 'border-accent text-accent'
                    : 'border-slate-600 text-slate-400 hover:border-slate-500'
                }`}
              >
                All
              </button>
              {allTags.map((t) => (
                <button
                  key={t}
                  type="button"
                  onClick={() => setTag(t === tag ? null : t)}
                  className={`rounded-full border px-2.5 py-0.5 text-xs ${
                    tag === t
                      ? 'border-accent text-accent'
                      : 'border-slate-600 text-slate-400 hover:border-slate-500'
                  }`}
                >
                  {t}
                </button>
              ))}
            </div>
          )}
        </div>
      )}

      {isLoading && (
        <div className="flex items-center gap-2 text-slate-400">
          <Loader2 className="h-5 w-5 animate-spin" aria-hidden />
          Loading markets…
        </div>
      )}
      {error && (
        <p className="text-red-400 text-sm" role="alert">
          Failed to load markets. Is the gateway running?
        </p>
      )}

      <ul className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {filtered.map((m) => (
          <li key={m.id}>
            <Link
              to={`/m/${encodeURIComponent(m.id)}`}
              className="group flex flex-col rounded-xl border border-slate-800 bg-surface-muted/40 p-4 transition-colors hover:border-slate-600 hover:bg-surface-muted/70 focus:outline-none focus:ring-2 focus:ring-accent h-full"
            >
              <div className="flex items-start justify-between gap-2">
                <span className="font-medium text-slate-100 group-hover:text-white line-clamp-2">
                  {m.name}
                </span>
                <Badge variant="outline" className="shrink-0 text-[10px]">
                  tick {m.tick_size}¢
                </Badge>
              </div>
              {m.description ? (
                <p className="mt-2 text-xs text-slate-400 line-clamp-3">{m.description}</p>
              ) : null}
              {m.tags && m.tags.length > 0 && (
                <div className="mt-2 flex flex-wrap gap-1">
                  {m.tags.map((t) => (
                    <span
                      key={t}
                      className="rounded bg-slate-800/80 px-1.5 py-0.5 text-[10px] text-slate-400"
                    >
                      {t}
                    </span>
                  ))}
                </div>
              )}
              <span className="mt-2 font-mono text-[11px] text-slate-500 truncate">{m.id}</span>
              {m.settled != null && (
                <p className="mt-1 text-xs text-amber-200/90">
                  Resolved: {m.settled ? 'YES' : 'NO'}
                </p>
              )}
              <span className="mt-4 inline-flex items-center gap-1 text-sm font-medium text-accent">
                {m.settled != null ? 'View' : 'Open'}
                <ArrowRight className="h-4 w-4 transition-transform group-hover:translate-x-0.5" />
              </span>
            </Link>
          </li>
        ))}
      </ul>

      {!isLoading && !error && filtered.length === 0 && (data?.length ?? 0) > 0 && (
        <p className="text-slate-500 text-sm">No markets match your filters.</p>
      )}

      {!isLoading && !error && (data?.length ?? 0) === 0 && (
        <p className="text-slate-500 text-sm">No markets returned from the API.</p>
      )}
    </div>
  )
}
