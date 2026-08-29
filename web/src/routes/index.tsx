import { createFileRoute, redirect, useNavigate } from '@tanstack/react-router'
import { FolderSync, LogOut, PanelLeft, StickyNote } from 'lucide-react'
import { useCallback, useEffect, useState } from 'react'
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
import type { LogoutResponse, MeResponse, SetupStatusResponse, StatsResponse } from '@/lib/types'
import { useWebSocketSync } from '@/lib/useWebSocketSync'

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

  const [wsStatus, setWsStatus] = useState<'connected' | 'connecting' | 'disconnected'>(
    'connecting',
  )
  const [notesCount, setNotesCount] = useState<number>(me?.notes_count ?? 0)
  const [foldersCount, setFoldersCount] = useState<number>(me?.folders_count ?? 0)

  const refreshStats = useCallback(async () => {
    try {
      const stats = await apiFetch<StatsResponse>('/api/stats')
      if (typeof stats?.notes_count === 'number') {
        setNotesCount(stats.notes_count)
      }
      if (typeof stats?.folders_count === 'number') {
        setFoldersCount(stats.folders_count)
      }
    } catch {}
  }, [])

  useEffect(() => {
    void refreshStats()
  }, [refreshStats])

  useWebSocketSync({
    enabled: Boolean(me),
    onStatusChange: setWsStatus,
    onSyncNotification: () => {
      void refreshStats()
    },
  })

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
          {/* Live WebSocket Status Pill */}
          <div className="flex items-center gap-1.5 rounded-full bg-muted/60 border border-border px-2.5 py-1 text-xs text-muted-foreground">
            <span
              className={`inline-block h-2 w-2 rounded-full ${
                wsStatus === 'connected'
                  ? 'bg-emerald-500 animate-pulse'
                  : wsStatus === 'connecting'
                    ? 'bg-amber-500 animate-pulse'
                    : 'bg-zinc-500'
              }`}
            />
            <span className="hidden sm:inline font-medium">
              {wsStatus === 'connected'
                ? 'Live Sync Connected'
                : wsStatus === 'connecting'
                  ? 'Connecting WS...'
                  : 'Offline'}
            </span>
          </div>

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
      <main className="flex flex-1 flex-col items-center justify-center p-6 gap-6">
        <div className="flex flex-col items-center gap-4 text-center max-w-md">
          <div className="flex h-16 w-16 items-center justify-center rounded-3xl bg-muted/60 text-muted-foreground border border-border">
            <StickyNote className="h-8 w-8" />
          </div>
          <div>
            <h2 className="text-lg font-semibold text-foreground">
              Welcome back{me?.username ? `, ${me.username}` : ''}!
            </h2>
            <p className="text-sm text-muted-foreground mt-1">
              Your server is securely connected and ready. Real-time WebSocket sync is active.
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

        {/* Live Vault Metrics (Notes & Folders Count) */}
        <div className="grid grid-cols-2 gap-3 w-full max-w-md">
          <div className="flex items-center justify-between rounded-2xl border border-border/80 bg-card/60 p-4 shadow-sm backdrop-blur-sm">
            <div>
              <p className="text-[11px] font-medium uppercase tracking-wider text-muted-foreground">
                Total Notes
              </p>
              <div className="flex items-baseline gap-1.5 mt-1">
                <span className="text-2xl font-bold tracking-tight text-foreground">
                  {notesCount.toLocaleString()}
                </span>
                <span className="text-[11px] text-muted-foreground">synced</span>
              </div>
            </div>
            <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-primary/10 text-primary">
              <StickyNote className="h-5 w-5" />
            </div>
          </div>

          <div className="flex items-center justify-between rounded-2xl border border-border/80 bg-card/60 p-4 shadow-sm backdrop-blur-sm">
            <div>
              <p className="text-[11px] font-medium uppercase tracking-wider text-muted-foreground">
                Folders
              </p>
              <div className="flex items-baseline gap-1.5 mt-1">
                <span className="text-2xl font-bold tracking-tight text-foreground">
                  {foldersCount.toLocaleString()}
                </span>
                <span className="text-[11px] text-muted-foreground">active</span>
              </div>
            </div>
            <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-muted text-muted-foreground">
              <FolderSync className="h-5 w-5" />
            </div>
          </div>
        </div>
      </main>
    </div>
  )
}
