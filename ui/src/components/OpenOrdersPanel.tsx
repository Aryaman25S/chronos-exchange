import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Trash2 } from 'lucide-react'
import type { OpenOrderRow } from '@/api/types'
import { Button } from '@/components/ui/button'

export function OpenOrdersPanel({
  marketId,
  userId,
  orders,
  isLoading,
  error,
}: {
  marketId: string
  userId: string
  orders: OpenOrderRow[] | undefined
  isLoading: boolean
  error: Error | null
}) {
  const qc = useQueryClient()
  const cancel = useMutation({
    mutationFn: async (orderId: string) => {
      const r = await fetch(
        `/v1/orders/${orderId}?${new URLSearchParams({ market_id: marketId })}`,
        { method: 'DELETE', headers: { 'X-User-Id': userId } }
      )
      if (!r.ok) throw new Error(await r.text())
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['orders', userId, marketId] })
      qc.invalidateQueries({ queryKey: ['positions', userId] })
    },
  })

  if (error) {
    return <p className="text-red-400 text-sm">Could not load open orders.</p>
  }
  if (isLoading) {
    return <p className="text-slate-500 text-sm">Loading open orders…</p>
  }
  if (!orders?.length) {
    return <p className="text-slate-500 text-sm">No resting orders in this market.</p>
  }

  return (
    <div className="rounded-xl border border-slate-800 bg-surface-muted/40 overflow-hidden">
      <div className="px-4 py-2 border-b border-slate-800 text-sm font-medium text-slate-200">
        Open orders
      </div>
      <ul className="divide-y divide-slate-800/80 max-h-48 overflow-y-auto">
        {orders.map((o) => (
          <li
            key={o.order_id}
            className="flex items-center justify-between gap-2 px-4 py-2 text-sm"
          >
            <div className="min-w-0">
              <span
                className={o.side.toLowerCase() === 'buy' ? 'text-bid' : 'text-ask'}
              >
                {o.side}
              </span>
              <span className="text-slate-400 mx-1">·</span>
              <span className="font-mono text-slate-200">{o.price}¢</span>
              <span className="text-slate-500 mx-1">×</span>
              <span className="text-slate-300">{o.qty}</span>
            </div>
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="shrink-0 text-slate-400 hover:text-red-300"
              disabled={cancel.isPending}
              onClick={() => cancel.mutate(o.order_id)}
              aria-label={`Cancel order ${o.order_id}`}
            >
              <Trash2 className="h-4 w-4" />
            </Button>
          </li>
        ))}
      </ul>
    </div>
  )
}
