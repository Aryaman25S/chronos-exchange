/** Compact play-money volume (simulated USD). */
export function formatVolUsd(n: number): string {
  if (!Number.isFinite(n) || n < 0) return '$0'
  if (n >= 1_000_000) return `$${(n / 1_000_000).toFixed(2)}M`
  if (n >= 10_000) return `$${Math.round(n / 1_000)}k`
  if (n >= 1_000) return `$${(n / 1_000).toFixed(1)}k`
  if (n >= 1) return `$${n.toFixed(2)}`
  return `$${n.toFixed(2)}`
}
