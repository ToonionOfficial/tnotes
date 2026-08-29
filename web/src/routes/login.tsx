import { useForm } from '@tanstack/react-form'
import { createFileRoute, Link, redirect, useNavigate } from '@tanstack/react-router'
import { Loader2, NotebookPen } from 'lucide-react'
import { useEffect, useState } from 'react'
import { z } from 'zod'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import Shake from '@/components/ui/shake'
import { apiFetch } from '@/lib/api'
import type { LoginResponse, MeResponse, SetupStatusResponse } from '@/lib/types'

export const Route = createFileRoute('/login')({
  beforeLoad: async () => {
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

    // 2. Client navigation check: If already logged in, redirect to workspace
    try {
      await apiFetch<MeResponse>('/api/me')
      throw redirect({ to: '/' })
    } catch (err) {
      if (err && typeof err === 'object' && 'to' in err) throw err
    }
  },
  component: LoginPage,
})

const usernameSchema = z.string().min(1, 'Username is required')
const passwordSchema = z.string().min(1, 'Password is required')

function getOrCreateDeviceId(): string {
  if (typeof window === 'undefined') return 'web_ssr'
  let id = localStorage.getItem('tnotes_device_id')
  if (!id) {
    id = `web_${Math.random().toString(36).substring(2, 11)}`
    localStorage.setItem('tnotes_device_id', id)
  }
  return id
}

function LoginPage() {
  const navigate = useNavigate()
  const [checkingAuth, setCheckingAuth] = useState(true)
  const [serverError, setServerError] = useState('')

  useEffect(() => {
    let mounted = true
    Promise.allSettled([
      apiFetch<SetupStatusResponse>('/api/setup/status'),
      apiFetch<MeResponse>('/api/me'),
    ]).then(([statusRes, meRes]) => {
      if (!mounted) return

      if (statusRes.status === 'fulfilled' && !statusRes.value.is_configured) {
        navigate({ to: '/setup', replace: true })
        return
      }

      if (meRes.status === 'fulfilled') {
        navigate({ to: '/', replace: true })
        return
      }

      setCheckingAuth(false)
    })

    return () => {
      mounted = false
    }
  }, [navigate])

  const form = useForm({
    defaultValues: {
      username: '',
      password: '',
    },
    onSubmit: async ({ value }) => {
      setServerError('')

      try {
        await apiFetch<LoginResponse>('/api/login', {
          method: 'POST',
          body: JSON.stringify({
            username: value.username.trim(),
            password: value.password,
            device_id: getOrCreateDeviceId(),
            device_name: 'Web Browser',
            platform: 'web',
          }),
        })

        navigate({ to: '/' })
      } catch (err) {
        const message = err instanceof Error ? err.message : 'Invalid username or password'
        setServerError(message)
      }
    },
  })

  if (checkingAuth) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-background">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-4 sm:p-6">
      <Card className="w-full max-w-md border-border/80 shadow-lg">
        <CardHeader className="space-y-2 text-center">
          <div className="mx-auto flex h-12 w-12 items-center justify-center rounded-xl bg-primary/10 text-primary">
            <NotebookPen className="h-6 w-6" />
          </div>
          <CardTitle className="text-2xl font-bold tracking-tight">Welcome Back</CardTitle>
          <CardDescription>Sign in to access and sync your notes</CardDescription>
        </CardHeader>

        <CardContent>
          <Shake signal={serverError}>
            {serverError && (
              <div className="mb-4 rounded-md bg-destructive/10 p-3 text-sm text-destructive font-medium border border-destructive/20">
                {serverError}
              </div>
            )}

            <form
              onSubmit={(e) => {
                e.preventDefault()
                e.stopPropagation()
                form.handleSubmit()
              }}
              className="space-y-4"
            >
              <form.Field
                name="username"
                validators={{
                  onChange: ({ value }) => {
                    const res = usernameSchema.safeParse(value)
                    return res.success ? undefined : res.error.issues[0]?.message
                  },
                }}
              >
                {(field) => (
                  <div className="space-y-2">
                    <Label htmlFor={field.name}>Username</Label>
                    <Input
                      id={field.name}
                      name={field.name}
                      placeholder="Username"
                      autoComplete="username"
                      autoFocus
                      value={field.state.value}
                      onBlur={field.handleBlur}
                      onChange={(e) => field.handleChange(e.target.value)}
                    />
                    {field.state.meta.errors.length > 0 && (
                      <p className="text-xs text-destructive">
                        {String(field.state.meta.errors[0])}
                      </p>
                    )}
                  </div>
                )}
              </form.Field>

              <form.Field
                name="password"
                validators={{
                  onChange: ({ value }) => {
                    const res = passwordSchema.safeParse(value)
                    return res.success ? undefined : res.error.issues[0]?.message
                  },
                }}
              >
                {(field) => (
                  <div className="space-y-2">
                    <Label htmlFor={field.name}>Password</Label>
                    <Input
                      id={field.name}
                      name={field.name}
                      type="password"
                      placeholder="Password"
                      autoComplete="current-password"
                      value={field.state.value}
                      onBlur={field.handleBlur}
                      onChange={(e) => field.handleChange(e.target.value)}
                    />
                    {field.state.meta.errors.length > 0 && (
                      <p className="text-xs text-destructive">
                        {String(field.state.meta.errors[0])}
                      </p>
                    )}
                  </div>
                )}
              </form.Field>

              <form.Subscribe selector={(state) => [state.canSubmit, state.isSubmitting]}>
                {([canSubmit, isSubmitting]) => (
                  <Button type="submit" className="w-full" disabled={!canSubmit || isSubmitting}>
                    {isSubmitting ? (
                      <>
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        Signing in…
                      </>
                    ) : (
                      'Sign In'
                    )}
                  </Button>
                )}
              </form.Subscribe>
            </form>
          </Shake>

          <div className="mt-4 text-center">
            <Link
              to="/setup"
              className="text-xs text-muted-foreground hover:text-foreground underline transition-colors"
            >
              First time setup? Create admin account
            </Link>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
