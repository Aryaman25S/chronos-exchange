import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom'
import { Toaster } from 'sonner'
import { AppShell } from '@/components/AppShell'
import { RouteErrorFallback } from '@/components/RouteErrorFallback'
import { MarketsPage } from '@/pages/MarketsPage'
import { MarketPage } from '@/pages/MarketPage'

const queryClient = new QueryClient({
  defaultOptions: {
    queries: { retry: 1, refetchOnWindowFocus: false },
  },
})

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <Toaster richColors closeButton position="top-center" theme="dark" />
        <Routes>
          <Route element={<AppShell />} errorElement={<RouteErrorFallback />}>
            <Route index element={<MarketsPage />} />
            <Route path="m/:marketId" element={<MarketPage />} />
            <Route path="*" element={<Navigate to="/" replace />} />
          </Route>
        </Routes>
      </BrowserRouter>
    </QueryClientProvider>
  )
}
