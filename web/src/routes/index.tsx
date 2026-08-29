import { createFileRoute, redirect, useNavigate } from '@tanstack/react-router'
import { Activity, FolderSync, LogOut, PanelLeft, Radio, StickyNote } from 'lucide-react'
import { useState } from 'react'
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
import { useWebSocketSync, type WsSyncPayload } from '@/lib/useWebSocketSync'

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

interface SyncLogEntry {
  id: string
  time: string
  message: string
  count?: number
}

function DashboardPage() {
  const navigate = useNavigate()
  const context = Route.useRouteContext()
  const me = context?.me

  const [wsStatus, setWsStatus] = useState<'connected' | 'connecting' | 'disconnected'>(
    'connecting',
  )
  const [syncLogs, setSyncLogs] = useState<SyncLogEntry[]>([])

  useWebSocketSync({
    enabled: Boolean(me),
    onStatusChange: setWsStatus,
    onSyncNotification: (payload?: WsSyncPayload) => {
      const now = new Date().toLocaleTimeString()
      const deviceLabel = payload?.sender_device_id
        ? `device "${payload.sender_device_id}"`
        : 'another device'
      const changeText =
        payload?.count !== undefined
          ? ` (${payload.count} change${payload.count === 1 ? '' : 's'})`
          : ''

      const newEntry: SyncLogEntry = {
        id: `${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
        time: now,
        message: `Sync broadcast received from ${deviceLabel}${changeText}`,
        count: payload?.count,
      }

      setSyncLogs((prev) => [newEntry, ...prev].slice(0, 10))
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

      {/* Main Workspace Placeholder & Live Sync Stream */}
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

        {/* Real-time Broadcast Activity Feed */}
        <div className="w-full max-w-md rounded-2xl border border-border/80 bg-card/60 p-4 shadow-sm backdrop-blur-sm">
          <div className="flex items-center justify-between pb-3 border-b border-border/60">
            <div className="flex items-center gap-2">
              <Radio className="h-4 w-4 text-primary animate-pulse" />
              <span className="text-xs font-semibold uppercase tracking-wider text-foreground">
                Live Sync Activity
              </span>
            </div>
            <span className="text-[11px] text-muted-foreground">
              {syncLogs.length} event{syncLogs.length === 1 ? '' : 's'}
            </span>
          </div>

          <div className="pt-3 space-y-2 max-h-48 overflow-y-auto">
            {syncLogs.length === 0 ? (
              <div className="py-4 text-center text-xs text-muted-foreground">
                Waiting for device sync broadcasts...
                <br />
                <span className="text-[11px] opacity-70">
                  (Make a change or tap Sync on your phone to see it appear here live)
                </span>
              </div>
            ) : (
              syncLogs.map((log) => (
                <div
                  key={log.id}
                  className="flex items-start gap-2.5 rounded-lg bg-muted/40 p-2.5 text-xs text-foreground animate-in fade-in duration-300"
                >
                  <Activity className="h-3.5 w-3.5 text-emerald-500 mt-0.5 shrink-0" />
                  <div className="flex-1 min-w-0">
                    <p className="font-medium text-foreground leading-snug">{log.message}</p>
                    <p className="text-[10px] text-muted-foreground mt-0.5">{log.time}</p>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      </main>
    </div>
  )
}
