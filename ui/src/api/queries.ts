import { useQuery } from '@tanstack/react-query'
import type { FillRow, Market, MarketPosition, OpenOrderRow } from './types'

async function json<T>(r: Response | Promise<Response>): Promise<T> {
  const res = await r
  if (!res.ok) throw new Error(`${res.status} ${res.statusText}`)
  return res.json() as Promise<T>
}

export function marketsQueryOptions() {
  return {
    queryKey: ['markets'] as const,
    queryFn: () => json<Market[]>(fetch('/v1/markets')),
    staleTime: 30_000,
    refetchInterval: 45_000,
  }
}

export function useMarkets() {
  return useQuery(marketsQueryOptions())
}

export function usePositions(userId: string) {
  return useQuery({
    queryKey: ['positions', userId] as const,
    queryFn: () =>
      json<Record<string, MarketPosition>>(
        fetch('/v1/positions', { headers: { 'X-User-Id': userId } })
      ),
    enabled: Boolean(userId),
  })
}

export function useFills(limit = 40) {
  return useQuery({
    queryKey: ['fills', limit] as const,
    queryFn: () => json<FillRow[]>(fetch(`/v1/fills?limit=${limit}`)),
    staleTime: 5_000,
  })
}

export function useOpenOrders(userId: string, marketId: string) {
  return useQuery({
    queryKey: ['orders', userId, marketId] as const,
    queryFn: () =>
      json<OpenOrderRow[]>(
        fetch(`/v1/orders?${new URLSearchParams({ market_id: marketId })}`, {
          headers: { 'X-User-Id': userId },
        })
      ),
    enabled: Boolean(userId && marketId),
    staleTime: 3_000,
  })
}
