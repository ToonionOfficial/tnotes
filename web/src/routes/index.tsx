import { createFileRoute, redirect } from '@tanstack/react-router'
import { NotebookPen } from 'lucide-react'
import { apiFetch } from '@/lib/api'
import type { SetupStatusResponse } from '@/lib/types'

export const Route = createFileRoute('/')({
  beforeLoad: async () => {
    try {
      const status = await apiFetch<SetupStatusResponse>('/api/setup/status')
      if (!status.is_configured) {
        throw redirect({ to: '/setup' })
      }
      // When configured, redirect to login for now (or notes when authenticated)
      throw redirect({ to: '/login' })
    } catch (err) {
      if (err && typeof err === 'object' && 'to' in err) {
        throw err
      }
    }
  },
  component: IndexPage,
})

function IndexPage() {
  return (
    <div className="flex min-h-screen items-center justify-center p-5">
      <div className="flex flex-col items-center gap-3 text-center">
        <NotebookPen className="h-12 w-12 text-primary" />
        <h1 className="text-2xl font-bold text-foreground">Notat</h1>
        <p className="text-sm text-muted-foreground">
          Self-hosted markdown notes with sync
        </p>
      </div>
    </div>
  )
}
