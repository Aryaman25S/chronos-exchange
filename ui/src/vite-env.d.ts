/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** Full WebSocket URL (e.g. `wss://api.example.com/ws`). If unset, uses same host as the page + `/ws`. */
  readonly VITE_WS_URL?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
