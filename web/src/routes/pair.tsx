import { createFileRoute, redirect, useNavigate, useRouter } from '@tanstack/react-router'
import { Check, Copy, FolderSync, Globe, RefreshCw, Smartphone, Sparkles, User } from 'lucide-react'
import { useEffect, useState } from 'react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { apiFetch } from '@/lib/api'
import type { MeResponse, PairingDataResponse } from '@/lib/types'

interface PairSearch {
  welcome?: boolean
}

export const Route = createFileRoute('/pair')({
  validateSearch: (search: Record<string, unknown>): PairSearch => ({
    welcome: search.welcome === true || search.welcome === 'true' ? true : undefined,
  }),
  beforeLoad: async () => {
    if (typeof window === 'undefined') return
    try {
      await apiFetch<MeResponse>('/api/me')
    } catch {
      throw redirect({ to: '/login' })
    }
  },
  loader: async () => {
    const pairData = await apiFetch<PairingDataResponse>('/api/pair')
    return { pairData }
  },
  component: PairPage,
})

function PairPage() {
  const navigate = useNavigate()
  const router = useRouter()
  const search = Route.useSearch()
  const { pairData } = Route.useLoaderData()
  const isWelcome = search.welcome === true

  const [copied, setCopied] = useState<'code' | 'url' | null>(null)
  const [isRefreshing, setIsRefreshing] = useState(false)
  const [timeLeft, setTimeLeft] = useState<number>(() => {
    return Math.max(0, Math.floor((pairData.expires_at - Date.now()) / 1000))
  })

  useEffect(() => {
    const interval = setInterval(() => {
      const remaining = Math.max(0, Math.floor((pairData.expires_at - Date.now()) / 1000))
      setTimeLeft(remaining)
    }, 1000)

    return () => clearInterval(interval)
  }, [pairData.expires_at])

  function formatTimer(seconds: number) {
    const m = Math.floor(seconds / 60)
    const s = seconds % 60
    return `${m}:${s < 10 ? '0' : ''}${s}`
  }

  function handleCopy(type: 'code' | 'url', text: string) {
    navigator.clipboard.writeText(text)
    setCopied(type)
    setTimeout(() => setCopied(null), 2000)
  }

  async function handleRegenerate() {
    setIsRefreshing(true)
    await router.invalidate()
    setIsRefreshing(false)
  }

  return (
    <div className="flex min-h-screen items-center justify-center p-4 sm:p-6 bg-background">
      <Card className="w-full max-w-lg border-border/80 shadow-2xl">
        <CardHeader className="text-center pb-3">
          <div className="mx-auto mb-3 flex h-14 w-14 items-center justify-center rounded-2xl bg-primary/10 text-primary ring-1 ring-primary/20">
            {isWelcome ? <Sparkles className="h-7 w-7" /> : <FolderSync className="h-7 w-7" />}
          </div>
          <CardTitle className="text-2xl font-bold">
            {isWelcome ? "You're all set!" : 'Pair a Mobile Device'}
          </CardTitle>
          <CardDescription className="text-sm">
            {isWelcome
              ? 'Your vault is ready. Scan with the mobile app to sync your notes, or continue to the web editor.'
              : 'Scan this QR code with the TNotes Mobile App to instantly pair your device.'}
          </CardDescription>
        </CardHeader>

        <CardContent className="space-y-5">
          {/* QR Code Display */}
          <div className="flex flex-col items-center justify-center rounded-2xl border border-border bg-card/60 p-5 shadow-inner">
            <div className="flex flex-col items-center gap-4">
              {/* QR SVG */}
              <div
                className="rounded-xl bg-white p-3 shadow-md [&_svg]:h-56 [&_svg]:w-56 [&_svg]:max-w-full"
                dangerouslySetInnerHTML={{ __html: pairData.qr_svg }}
              />

              {/* Account & Server Meta */}
              <div className="flex flex-wrap items-center justify-center gap-3 text-xs text-muted-foreground">
                <div className="flex items-center gap-1">
                  <User className="h-3.5 w-3.5 text-primary" />
                  <span className="font-semibold text-foreground">{pairData.username}</span>
                </div>
                <span>•</span>
                <div className="flex items-center gap-1">
                  <Globe className="h-3.5 w-3.5 text-muted-foreground" />
                  <button
                    type="button"
                    onClick={() => handleCopy('url', pairData.url)}
                    className="hover:text-foreground underline decoration-dotted transition-colors"
                    title="Copy server URL"
                  >
                    {pairData.url}
                  </button>
                  {copied === 'url' && <Check className="h-3 w-3 text-success" />}
                </div>
              </div>

              {/* 6-Digit Manual Code */}
              <div className="w-full space-y-2 pt-1 border-t border-border/50">
                <div className="flex items-center justify-between">
                  <span className="text-xs text-muted-foreground">Manual pairing code:</span>
                  <button
                    type="button"
                    onClick={() => handleCopy('code', pairData.pairing_code)}
                    className="inline-flex items-center gap-1.5 rounded-lg border border-border bg-muted/80 px-3 py-1 font-mono text-sm font-bold tracking-widest text-foreground hover:bg-muted transition-colors shadow-sm"
                  >
                    {pairData.pairing_code}
                    {copied === 'code' ? (
                      <Check className="h-3.5 w-3.5 text-success" />
                    ) : (
                      <Copy className="h-3.5 w-3.5 text-muted-foreground" />
                    )}
                  </button>
                </div>

                <div className="flex items-center justify-between text-xs text-muted-foreground pt-0.5">
                  <span>
                    {timeLeft > 0 ? (
                      <>
                        Expires in{' '}
                        <span className="font-mono font-medium text-foreground">
                          {formatTimer(timeLeft)}
                        </span>
                      </>
                    ) : (
                      <span className="text-destructive font-medium">Code expired</span>
                    )}
                  </span>
                  <button
                    type="button"
                    onClick={handleRegenerate}
                    disabled={isRefreshing}
                    className="inline-flex items-center gap-1 hover:text-foreground transition-colors"
                  >
                    <RefreshCw className={`h-3 w-3 ${isRefreshing ? 'animate-spin' : ''}`} />
                    Refresh Code
                  </button>
                </div>
              </div>
            </div>
          </div>

          {/* Quick Instructions */}
          <div className="rounded-xl bg-muted/40 border border-border/50 p-3.5 text-xs text-muted-foreground space-y-1.5">
            <div className="flex items-center gap-2 font-medium text-foreground">
              <Smartphone className="h-4 w-4 text-primary" />
              <span>How to pair your phone:</span>
            </div>
            <ol className="list-decimal list-inside space-y-1 pl-1 text-[11px] leading-relaxed">
              <li>
                Open the <strong>TNotes</strong> app on your iOS or Android device.
              </li>
              <li>
                Open <strong>Settings</strong> $\rightarrow$ tap <strong>"Connect Server"</strong>.
              </li>
              <li>Scan the QR code or enter the 6-digit code manually.</li>
            </ol>
          </div>

          {/* Action Buttons */}
          <div className="space-y-2 pt-1">
            <Button
              onClick={() => navigate({ to: '/' })}
              className="w-full shadow-md font-medium"
              size="lg"
            >
              {isWelcome ? 'Open My Vault' : 'Done'}
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
