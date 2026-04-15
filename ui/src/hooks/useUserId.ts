import { useMemo } from 'react'

const USER_KEY = 'chronos_user_id'

export function getOrCreateUserId(): string {
  let id = localStorage.getItem(USER_KEY)
  if (!id) {
    id = crypto.randomUUID()
    localStorage.setItem(USER_KEY, id)
  }
  return id
}

export function useUserId(): string {
  return useMemo(() => getOrCreateUserId(), [])
}
