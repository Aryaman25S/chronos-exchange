import { Link, Outlet } from 'react-router-dom'
import { Badge } from '@/components/ui/badge'

export function AppShell() {
  return (
    <div className="min-h-screen flex flex-col">
      <header className="sticky top-0 z-40 border-b border-slate-800/80 bg-[hsl(222_47%_6%)]/95 backdrop-blur supports-[backdrop-filter]:bg-[hsl(222_47%_6%)]/80">
        <div className="mx-auto flex h-14 max-w-7xl items-center justify-between gap-4 px-4">
          <div className="flex items-center gap-3 min-w-0">
            <Link to="/" className="font-semibold tracking-tight text-slate-50 truncate">
              Chronos
            </Link>
            <Badge variant="sim">Simulation</Badge>
          </div>
          <nav className="flex items-center gap-3 text-sm">
            <Link
              to="/"
              className="text-slate-400 hover:text-slate-100 transition-colors hidden sm:inline"
            >
              Markets
            </Link>
          </nav>
        </div>
      </header>
      <main className="flex-1 mx-auto w-full max-w-7xl px-4 py-6">
        <Outlet />
      </main>
      <footer className="border-t border-slate-800/60 py-6 text-center text-xs text-slate-500">
        Play-money simulator — not real trading.
      </footer>
    </div>
  )
}
