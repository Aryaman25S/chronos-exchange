
import React, { useEffect, useState } from 'react'
type Level = [number, number]
function useWs(url: string) {
  const [snapshot, setSnapshot] = useState<any>(null)
  const [deltas, setDeltas] = useState<any[]>([])
  useEffect(() => {
    const ws = new WebSocket(url)
    ws.onmessage = (ev) => { const msg = JSON.parse(ev.data); if (msg.type==='snapshot') setSnapshot(msg); else if (msg.type==='delta') setDeltas(d=>[...d,msg]) }
    return () => ws.close()
  }, [url])
  return { snapshot, deltas }
}
function Ladder({ bids, asks }: { bids: Level[]; asks: Level[] }) {
  return (<div className="grid grid-cols-2 gap-2">
    <div><h3 className="text-green-400 mb-1">Bids</h3><div className="space-y-1">{bids.map(([p,q],i)=>(<div key={i} className="flex justify-between bg-green-950/40 rounded px-2 py-1"><span>{(p/100).toFixed(2)}</span><span>{q}</span></div>))}</div></div>
    <div><h3 className="text-red-400 mb-1">Asks</h3><div className="space-y-1">{asks.map(([p,q],i)=>(<div key={i} className="flex justify-between bg-red-950/40 rounded px-2 py-1"><span>{(p/100).toFixed(2)}</span><span>{q}</span></div>))}</div></div>
  </div>)
}
function Ticket() {
  const [side, setSide] = useState<'buy'|'sell'>('buy')
  const [price, setPrice] = useState(50)
  const [qty, setQty] = useState(1)
  const submit = async () => {
    await fetch('/v1/orders',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({market_id:'OKC_WIN_YESNO',side,price,qty,tif:'GTC',idempotency:crypto.randomUUID()})})
  }
  return (<div className="bg-slate-900 rounded-xl p-3 space-y-2">
    <div className="flex gap-2"><button onClick={()=>setSide('buy')} className={`px-3 py-1 rounded ${side==='buy'?'bg-green-700':'bg-slate-700'}`}>YES</button><button onClick={()=>setSide('sell')} className={`px-3 py-1 rounded ${side==='sell'?'bg-red-700':'bg-slate-700'}`}>NO</button></div>
    <div className="flex gap-2 items-center"><label className="w-16">Price</label><input type="number" value={price} onChange={e=>setPrice(parseInt(e.target.value||'0'))} className="bg-slate-800 rounded px-2 py-1 w-24"/><span className="opacity-60">$1 ticks</span></div>
    <div className="flex gap-2 items-center"><label className="w-16">Qty</label><input type="number" value={qty} onChange={e=>setQty(parseInt(e.target.value||'0'))} className="bg-slate-800 rounded px-2 py-1 w-24"/></div>
    <button onClick={submit} className="w-full bg-blue-600 hover:bg-blue-500 rounded px-3 py-2">Place Order</button>
  </div>)
}
export default function App() {
  const { snapshot } = useWs(`ws://${location.hostname}:8081/ws?market=OKC_WIN_YESNO`)
  const bids: Level[] = snapshot?.bids || []
  const asks: Level[] = snapshot?.asks || []
  return (<div className="max-w-6xl mx-auto p-6 grid grid-cols-1 md:grid-cols-3 gap-6">
    <div className="md:col-span-2"><h1 className="text-2xl font-bold mb-4">Chronos Exchange (Sim)</h1><Ladder bids={bids} asks={asks} /></div>
    <div className="md:col-span-1 space-y-4"><Ticket /><div className="bg-slate-900 rounded-xl p-3">Recent Trades — soon</div></div>
  </div>)
}
