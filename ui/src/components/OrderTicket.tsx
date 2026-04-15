import { useMutation, useQueryClient } from '@tanstack/react-query'
import * as Label from '@radix-ui/react-label'
import { ChevronDown, Loader2 } from 'lucide-react'
import { useState } from 'react'
import { toast } from 'sonner'
import type { PlaceOrderResponse } from '@/api/types'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

export function OrderTicket({
  marketId,
  userId,
  className,
}: {
  marketId: string
  userId: string
  className?: string
}) {
  const qc = useQueryClient()
  const [side, setSide] = useState<'buy' | 'sell'>('buy')
  const [price, setPrice] = useState(50)
  const [qty, setQty] = useState(1)
  const [hint, setHint] = useState<string | null>(null)

  const mutation = useMutation({
    mutationFn: async (): Promise<PlaceOrderResponse> => {
      const r = await fetch('/v1/orders', {
        method: 'POST',
        headers: {
          'content-type': 'application/json',
          'X-User-Id': userId,
        },
        body: JSON.stringify({
          market_id: marketId,
          side,
          price,
          qty,
          tif: 'GTC',
          idempotency: crypto.randomUUID(),
        }),
      })
      const raw = await r.text()
      if (!r.ok) throw new Error(raw || r.statusText)
      return JSON.parse(raw) as PlaceOrderResponse
    },
    onSuccess: (j) => {
      if (j.self_trade_prevented) {
        const msg =
          'Self-trade blocked: your order would only match your own resting quote. Change price, wait for others, or use another user id.'
        setHint(msg)
        toast.message('Self-trade prevented', { description: msg.slice(0, 120) })
      } else if (j.rested && (!j.fills || j.fills.length === 0)) {
        setHint('Order is live (no immediate fill). Positions update when you trade.')
        toast.success('Order resting')
      } else if (j.fills && j.fills.length > 0) {
        setHint('Trade matched.')
        toast.success('Trade matched')
      } else {
        setHint(null)
      }
      qc.invalidateQueries({ queryKey: ['positions', userId] })
      qc.invalidateQueries({ queryKey: ['fills'] })
      qc.invalidateQueries({ queryKey: ['orders', userId, marketId] })
    },
    onError: (e: Error) => {
      setHint(e.message)
      toast.error('Order failed', { description: e.message })
    },
  })

  const priceDollars = price / 100
  const maxLossBuy = priceDollars * qty
  const maxLossSell = (1 - priceDollars) * qty

  return (
    <form
      className={cn(
        'rounded-xl border border-slate-800 bg-surface-muted/40 p-4 space-y-4',
        className
      )}
      onSubmit={(e) => {
        e.preventDefault()
        if (!mutation.isPending) {
          setHint(null)
          mutation.mutate()
        }
      }}
    >
      <div>
        <h3 className="text-sm font-semibold text-slate-200">Trade</h3>
        <p className="text-[11px] text-slate-500">Limit · GTC · binary YES outcome</p>
      </div>

      <div className="grid grid-cols-2 gap-2" role="group" aria-label="Outcome side">
        <Button
          type="button"
          variant={side === 'buy' ? 'bid' : 'secondary'}
          className="h-12 text-base font-semibold"
          onClick={() => setSide('buy')}
        >
          Buy YES
        </Button>
        <Button
          type="button"
          variant={side === 'sell' ? 'ask' : 'secondary'}
          className="h-12 text-base font-semibold"
          onClick={() => setSide('sell')}
        >
          Sell YES
        </Button>
      </div>

      <div className="space-y-3">
        <div className="space-y-1.5">
          <Label.Root htmlFor="px" className="text-xs text-slate-400">
            Limit price (¢)
          </Label.Root>
          <input
            id="px"
            type="number"
            min={1}
            max={99}
            value={price}
            onChange={(e) => setPrice(parseInt(e.target.value || '0', 10))}
            className="w-full rounded-md border border-slate-700 bg-slate-950/80 px-3 py-2 text-sm tabular-nums focus:outline-none focus:ring-2 focus:ring-accent"
          />
        </div>
        <div className="space-y-1.5">
          <Label.Root htmlFor="qty" className="text-xs text-slate-400">
            Contracts
          </Label.Root>
          <input
            id="qty"
            type="number"
            min={1}
            value={qty}
            onChange={(e) => setQty(parseInt(e.target.value || '0', 10))}
            className="w-full rounded-md border border-slate-700 bg-slate-950/80 px-3 py-2 text-sm tabular-nums focus:outline-none focus:ring-2 focus:ring-accent"
          />
        </div>
      </div>

      <div className="rounded-md bg-slate-950/50 border border-slate-800/80 px-3 py-2 text-[11px] text-slate-400 space-y-1">
        <div className="flex justify-between gap-2">
          <span>Notional</span>
          <span className="tabular-nums text-slate-300">${(priceDollars * qty).toFixed(2)}</span>
        </div>
        <div className="flex justify-between gap-2">
          <span>Est. max loss (sim)</span>
          <span className="tabular-nums text-slate-300">
            {side === 'buy' ? `$${maxLossBuy.toFixed(2)}` : `$${maxLossSell.toFixed(2)}`}
          </span>
        </div>
      </div>

      <Button
        type="submit"
        className="w-full h-11 text-base"
        disabled={mutation.isPending}
      >
        {mutation.isPending ? (
          <>
            <Loader2 className="mr-2 h-4 w-4 animate-spin inline" aria-hidden />
            Placing…
          </>
        ) : (
          'Place order'
        )}
      </Button>

      {hint && (
        <p
          className={cn(
            'text-xs leading-snug rounded-md px-2 py-2',
            hint.startsWith('Self-trade') || hint.startsWith('Could')
              ? 'bg-amber-500/10 text-amber-100 border border-amber-500/25'
              : 'bg-slate-800/80 text-slate-200 border border-slate-700'
          )}
          role="status"
        >
          {hint}
        </p>
      )}

      {mutation.isSuccess && mutation.data && (
        <details className="text-xs text-slate-500 group">
          <summary className="cursor-pointer flex items-center gap-1 list-none py-1 hover:text-slate-400">
            <ChevronDown className="h-3.5 w-3.5 transition-transform group-open:rotate-180" />
            Response details
          </summary>
          <pre className="mt-2 overflow-x-auto rounded-md bg-slate-950/80 p-2 text-[10px] border border-slate-800">
            {JSON.stringify(mutation.data, null, 2)}
          </pre>
        </details>
      )}
    </form>
  )
}
