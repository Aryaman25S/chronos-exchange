import { useQuery } from '@tanstack/react-query'
import { ChevronLeft, ChevronRight, Loader2 } from 'lucide-react'
import { useCallback, useMemo, useRef, useState } from 'react'
import { marketsQueryOptions } from '@/api/queries'
import { MarketCard } from '@/components/MarketCard'

const topicPill =
  'shrink-0 rounded-full border px-2.5 py-0.5 text-xs transition-colors whitespace-nowrap'

export function MarketsPage() {
  const { data, isLoading, error } = useQuery(marketsQueryOptions())
  const [q, setQ] = useState('')
  const [tag, setTag] = useState<string | null>(null)
  const topicsScrollRef = useRef<HTMLDivElement>(null)

  const scrollTopics = useCallback((dir: 'left' | 'right') => {
    const el = topicsScrollRef.current
    if (!el) return
    const delta = dir === 'right' ? 280 : -280
    el.scrollBy({ left: delta, behavior: 'smooth' })
  }, [])

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
        <h1 className="text-2xl font-bold tracking-tight text-slate-50 sm:text-3xl">
          Browse markets
        </h1>
        <p className="text-slate-400 max-w-2xl text-sm sm:text-base">
          In play-money binary contracts, prices imply event probability. Volume and odds update from
          the simulator&apos;s order book and trade history.
        </p>
      </div>

      {!isLoading && !error && (data?.length ?? 0) > 0 && (
        <div className="flex flex-col gap-4 max-w-full min-w-0">
          <label className="flex flex-col gap-1 text-xs text-slate-500 max-w-xl w-full">
            Search
            <input
              type="search"
              value={q}
              onChange={(e) => setQ(e.target.value)}
              placeholder="Events, topics, or keywords"
              className="rounded-lg border border-slate-700 bg-slate-950/50 px-3 py-2 text-sm text-slate-100 placeholder:text-slate-600 focus:outline-none focus:ring-2 focus:ring-accent"
            />
          </label>
          {allTags.length > 0 && (
            <div className="min-w-0">
              <div className="text-xs text-slate-500 mb-1.5">Topics</div>
              <div className="relative flex items-stretch gap-1 min-w-0">
                <button
                  type="button"
                  aria-label="Scroll topics left"
                  onClick={() => scrollTopics('left')}
                  className="hidden sm:flex shrink-0 items-center justify-center rounded-lg border border-slate-700 bg-slate-900/80 px-1.5 text-slate-400 hover:bg-slate-800 hover:text-slate-200"
                >
                  <ChevronLeft className="h-4 w-4" aria-hidden />
                </button>
                <div
                  ref={topicsScrollRef}
                  className="flex min-w-0 flex-1 flex-nowrap gap-2 overflow-x-auto scroll-smooth py-0.5 [scrollbar-width:none] [-ms-overflow-style:none] [&::-webkit-scrollbar]:hidden"
                >
                  <button
                    type="button"
                    onClick={() => setTag(null)}
                    className={`${topicPill} ${
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
                      className={`${topicPill} ${
                        tag === t
                          ? 'border-accent text-accent'
                          : 'border-slate-600 text-slate-400 hover:border-slate-500'
                      }`}
                    >
                      {t}
                    </button>
                  ))}
                </div>
                <button
                  type="button"
                  aria-label="Scroll topics right"
                  onClick={() => scrollTopics('right')}
                  className="flex shrink-0 items-center justify-center rounded-lg border border-slate-700 bg-slate-900/80 px-1.5 text-slate-400 hover:bg-slate-800 hover:text-slate-200"
                >
                  <ChevronRight className="h-4 w-4" aria-hidden />
                </button>
              </div>
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

      <ul className="grid gap-5 sm:grid-cols-2 xl:grid-cols-3">
        {filtered.map((m) => (
          <MarketCard key={m.id} m={m} />
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
