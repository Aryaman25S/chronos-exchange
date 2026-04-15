import * as Dialog from '@radix-ui/react-dialog'
import { X } from 'lucide-react'
import * as React from 'react'
import { cn } from '@/lib/utils'

export function Sheet({
  open,
  onOpenChange,
  title,
  children,
  side = 'bottom',
}: {
  open: boolean
  onOpenChange: (o: boolean) => void
  title: string
  children: React.ReactNode
  side?: 'bottom' | 'right'
}) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-50 bg-black/60 data-[state=open]:animate-in data-[state=closed]:animate-out fade-in" />
        <Dialog.Content
          className={cn(
            'fixed z-50 gap-4 border border-slate-700 bg-[hsl(222_47%_8%)] p-4 shadow-lg outline-none',
            side === 'bottom' &&
              'inset-x-0 bottom-0 max-h-[90vh] overflow-y-auto rounded-t-xl border-b-0',
            side === 'right' && 'inset-y-0 right-0 h-full w-full max-w-md border-l'
          )}
        >
          <Dialog.Description className="sr-only">Order entry panel</Dialog.Description>
          <div className="flex items-center justify-between gap-2 border-b border-slate-800 pb-3 mb-2">
            <Dialog.Title className="text-lg font-semibold tracking-tight">{title}</Dialog.Title>
            <Dialog.Close
              className="rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-accent"
              aria-label="Close"
            >
              <X className="h-5 w-5" />
            </Dialog.Close>
          </div>
          {children}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
