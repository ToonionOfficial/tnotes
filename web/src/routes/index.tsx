import { createFileRoute } from '@tanstack/react-router'
import { NotebookPen } from 'lucide-react'

export const Route = createFileRoute('/')({
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
