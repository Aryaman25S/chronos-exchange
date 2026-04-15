/** WebSocket URL for market data. Same-origin `/ws` in dev (Vite proxies to gateway). Override with `VITE_WS_URL`. */
export function getWebSocketUrl(marketId: string): string {
  const explicit = import.meta.env.VITE_WS_URL
  if (typeof explicit === 'string' && explicit.length > 0) {
    try {
      const u = new URL(explicit)
      u.searchParams.set('market_id', marketId)
      return u.toString()
    } catch {
      const sep = explicit.includes('?') ? '&' : '?'
      return `${explicit}${sep}${new URLSearchParams({ market_id: marketId }).toString()}`
    }
  }
  const { protocol, host } = window.location
  const wsProto = protocol === 'https:' ? 'wss:' : 'ws:'
  const base = `${wsProto}//${host}/ws`
  return `${base}?${new URLSearchParams({ market_id: marketId }).toString()}`
}
