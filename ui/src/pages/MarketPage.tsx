import { useQuery } from '@tanstack/react-query'
import { Activity, ChevronLeft, Radio } from 'lucide-react'
import { useMemo, useState } from 'react'
import { Link, useParams } from 'react-router-dom'
import { marketsQueryOptions, useFills, useOpenOrders, usePositions } from '@/api/queries'
import { ActivityFeed } from '@/components/ActivityFeed'
import { DepthLadder } from '@/components/DepthLadder'
import { OrderTicket } from '@/components/OrderTicket'
import { PortfolioPanel } from '@/components/PortfolioPanel'
import { Sheet } from '@/components/ui/sheet'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { OpenOrdersPanel } from '@/components/OpenOrdersPanel'
import { Tape } from '@/components/Tape'
import { useMarketWebSocket } from '@/hooks/useMarketWebSocket'
import { useUserId } from '@/hooks/useUserId'
import { cn } from '@/lib/utils'
import { getWebSocketUrl } from '@/lib/wsUrl'

export function MarketPage() {
  const { marketId: rawId } = useParams()
  const marketId = rawId ? decodeURIComponent(rawId) : ''
  const userId = useUserId()
  const [replay, setReplay] = useState(false)
  const [sheetOpen, setSheetOpen] = useState(false)

  const { data: markets } = useQuery(marketsQueryOptions())
  const marketMeta = useMemo(
    () => markets?.find((m) => m.id === marketId),
    [markets, marketId]
  )

  const ws = useMarketWebSocket(getWebSocketUrl(marketId), replay, marketId)
  const pos = usePositions(userId)
  const fills = useFills(30)
  const openOrders = useOpenOrders(userId, marketId)

  if (!marketId) {
    return <p className="text-slate-500">Invalid market.</p>
  }

  return (
    <div className="space-y-6 pb-24 md:pb-8">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div className="space-y-1 min-w-0">
          <Link
            to="/"
            className="inline-flex items-center gap-1 text-sm text-slate-400 hover:text-slate-200 mb-2"
          >
            <ChevronLeft className="h-4 w-4" aria-hidden />
            Markets
          </Link>
          <h1 className="text-2xl font-bold text-slate-50 truncate">
            {marketMeta?.name ?? marketId}
          </h1>
          <p className="font-mono text-xs text-slate-500 truncate">{marketId}</p>
          {marketMeta?.description ? (
            <p className="text-sm text-slate-400 max-w-2xl">{marketMeta.description}</p>
          ) : null}
          {marketMeta?.settled != null ? (
            <p className="text-sm text-amber-200/90">
              Resolved: {marketMeta.settled ? 'YES' : 'NO'} won — book is closed for trading.
            </p>
          ) : null}
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <Badge variant="sim">Simulation</Badge>
          <ConnectionPill status={ws.status} />
          <label className="flex items-center gap-2 text-xs text-slate-500 cursor-pointer">
            <input
              type="checkbox"
              checked={replay}
              onChange={(e) => setReplay(e.target.checked)}
              className="rounded border-slate-600"
            />
            Resync
          </label>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-12 lg:gap-8">
        <section className="lg:col-span-7 space-y-4">
          <DepthLadder
            bids={ws.book.bids}
            asks={ws.book.asks}
            lastTradePx={ws.lastTradePx}
          />
          <OpenOrdersPanel
            marketId={marketId}
            userId={userId}
            orders={openOrders.data}
            isLoading={openOrders.isLoading}
            error={openOrders.error as Error | null}
          />
        </section>

        <aside className="hidden lg:block lg:col-span-5 space-y-4">
          <OrderTicket marketId={marketId} userId={userId} />
          <Tape trades={ws.trades} />
          <PortfolioPanel
            positions={pos.data}
            isLoading={pos.isLoading}
            error={pos.error}
          />
          <ActivityFeed
            fills={fills.data}
            isLoading={fills.isLoading}
            error={fills.error}
          />
        </aside>
      </div>

      {/* Mobile: book + tape + portfolio visible; ticket in sheet */}
      <div className="lg:hidden space-y-4">
        <Tape trades={ws.trades} />
        <PortfolioPanel
          positions={pos.data}
          isLoading={pos.isLoading}
          error={pos.error}
        />
        <ActivityFeed
          fills={fills.data}
          isLoading={fills.isLoading}
          error={fills.error}
        />
      </div>

      <div className="fixed bottom-0 inset-x-0 z-30 border-t border-slate-800 bg-[hsl(222_47%_6%)]/95 backdrop-blur p-3 pb-[max(0.75rem,env(safe-area-inset-bottom))] md:hidden">
        <Button className="w-full h-12 text-base" onClick={() => setSheetOpen(true)}>
          Trade
        </Button>
      </div>

      <Sheet open={sheetOpen} onOpenChange={setSheetOpen} title="Place order">
        <OrderTicket marketId={marketId} userId={userId} />
      </Sheet>
    </div>
  )
}

function ConnectionPill({ status }: { status: import('@/hooks/useMarketWebSocket').WsStatus }) {
  const label =
    status === 'open'
      ? 'Live'
      : status === 'connecting'
        ? 'Connecting'
        : status === 'error'
          ? 'Feed error'
          : 'Disconnected'
  return (
    <span
      className={cn(
        'inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs font-medium',
        status === 'open' && 'border-emerald-500/40 bg-emerald-500/10 text-emerald-200',
        status === 'connecting' && 'border-amber-500/40 bg-amber-500/10 text-amber-200',
        (status === 'closed' || status === 'error') &&
          'border-red-500/35 bg-red-500/10 text-red-200'
      )}
    >
      {status === 'open' ? (
        <Activity className="h-3.5 w-3.5" aria-hidden />
      ) : (
        <Radio className="h-3.5 w-3.5" aria-hidden />
      )}
      {label}
    </span>
  )
}
