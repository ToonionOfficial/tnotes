import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/login')({
  component: LoginPage,
})

function LoginPage() {
  return (
    <div className="flex min-h-screen items-center justify-center p-5">
      <p className="text-muted-foreground">Login page — coming next</p>
    </div>
  )
}
