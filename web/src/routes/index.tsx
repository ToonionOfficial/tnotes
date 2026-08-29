import { createFileRoute, redirect, useNavigate } from '@tanstack/react-router'
import { FolderSync, LogOut, PanelLeft, StickyNote } from 'lucide-react'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { apiFetch } from '@/lib/api'
import type { LogoutResponse, MeResponse, SetupStatusResponse } from '@/lib/types'

export const Route = createFileRoute('/')({
  beforeLoad: async () => {
    // Only perform checks on client where cookies exist
    if (typeof window === 'undefined') return

    // 1. Check if server is configured
    try {
      const status = await apiFetch<SetupStatusResponse>('/api/setup/status')
      if (!status.is_configured) {
        throw redirect({ to: '/setup' })
      }
    } catch (err) {
      if (err && typeof err === 'object' && 'to' in err) throw err
    }

    // 2. Check if logged in
    try {
      const me = await apiFetch<MeResponse>('/api/me')
      return { me }
    } catch (err) {
      if (err && typeof err === 'object' && 'to' in err) throw err
      throw redirect({ to: '/login' })
    }
  },
  loader: ({ context }) => context,
  component: DashboardPage,
})

function DashboardPage() {
  const navigate = useNavigate()
  const context = Route.useRouteContext()
  const me = context?.me

  async function handleLogout() {
    try {
      await apiFetch<LogoutResponse>('/api/logout', { method: 'POST' })
    } catch {
      // Proceed with navigation regardless
    }
    navigate({ to: '/login' })
  }

  return (
    <div className="flex min-h-screen flex-col bg-background text-foreground">
      {/* Top Navigation Bar */}
      <header className="sticky top-0 z-20 flex h-14 items-center justify-between border-b border-border/80 bg-background/80 px-4 sm:px-6 backdrop-blur-md">
        <div className="flex items-center gap-2.5">
          <Button
            variant="ghost"
            size="icon-sm"
            className="text-muted-foreground hover:text-foreground"
            title="Toggle Sidebar"
          >
            <PanelLeft className="h-4 w-4" />
          </Button>
          <img src="/logo-transparent.png" alt="TNotes logo" className="h-8 w-8" />
          <span className="font-bold tracking-tight text-foreground">TNotes</span>
        </div>

        <div className="flex items-center gap-3">
          {!me?.has_paired_devices && (
            <Button
              variant="outline"
              size="sm"
              className="h-8 gap-1.5 text-xs font-medium"
              onClick={() => navigate({ to: '/pair' })}
            >
              <FolderSync className="h-3.5 w-3.5 text-primary" />
              <span className="hidden sm:inline">Pair Mobile App</span>
              <span className="sm:hidden">Pair</span>
            </Button>
          )}

          {!me?.has_paired_devices && <div className="h-4 w-px bg-border" />}

          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className="h-8 w-8 rounded-full p-0"
                title={me?.username ?? 'Account'}
              >
                <span className="flex h-8 w-8 items-center justify-center rounded-full bg-primary text-xs font-bold text-primary-foreground uppercase">
                  {me?.username?.charAt(0) ?? '?'}
                </span>
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-48">
              <DropdownMenuLabel>Profile</DropdownMenuLabel>
              <DropdownMenuItem disabled>{me?.username}</DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem onClick={() => navigate({ to: '/pair' })}>
                <FolderSync className="mr-2 h-4 w-4 text-muted-foreground" />
                Pair Device
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem variant="destructive" onClick={handleLogout}>
                <LogOut className="mr-2 h-4 w-4" />
                Sign out
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </header>

      {/* Main Workspace Placeholder */}
      <main className="flex flex-1 items-center justify-center p-6">
        <div className="flex flex-col items-center gap-4 text-center max-w-md">
          <div className="flex h-16 w-16 items-center justify-center rounded-3xl bg-muted/60 text-muted-foreground border border-border">
            <StickyNote className="h-8 w-8" />
          </div>
          <div>
            <h2 className="text-lg font-semibold text-foreground">
              Welcome back{me?.username ? `, ${me.username}` : ''}!
            </h2>
            <p className="text-sm text-muted-foreground mt-1">
              Your server is securely connected and ready. Phase 2 notes list and Markdown editor
              coming next.
            </p>
          </div>
          {!me?.has_paired_devices && (
            <div className="flex gap-2.5 pt-2">
              <Button
                variant="default"
                size="sm"
                className="gap-1.5"
                onClick={() => navigate({ to: '/pair' })}
              >
                <FolderSync className="h-4 w-4" />
                Pair Phone
              </Button>
            </div>
          )}
        </div>
      </main>
    </div>
  )
}
