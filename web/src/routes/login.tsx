import { useForm } from '@tanstack/react-form'
import { createFileRoute, redirect, useNavigate } from '@tanstack/react-router'
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

  // Client-side direct entry / refresh check: prevent authenticated access to /login
  useEffect(() => {
    let mounted = true
    apiFetch<MeResponse>('/api/me')
      .then(() => {
        if (mounted) {
          navigate({ to: '/', replace: true })
        }
      })
      .catch(() => {
        if (mounted) {
          setCheckingAuth(false)
        }
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
        setServerError(err instanceof Error ? err.message : 'Invalid credentials')
      }
    },
  })

  if (checkingAuth) {
    return (
      <div className="flex min-h-screen items-center justify-center p-5">
        <Loader2 className="h-8 w-8 animate-spin text-primary" />
      </div>
    )
  }

  return (
    <div className="flex min-h-screen items-center justify-center p-5">
      <Card className="w-full max-w-md shadow-2xl">
        <CardHeader className="text-center">
          <div className="mx-auto mb-2 flex h-12 w-12 items-center justify-center rounded-xl bg-primary/10 text-primary">
            <NotebookPen className="h-6 w-6" />
          </div>
          <CardTitle className="text-2xl font-bold">Sign In to TNotes</CardTitle>
          <CardDescription>Enter your credentials to access your notes</CardDescription>
        </CardHeader>
        <CardContent>
          <Shake signal={serverError}>
            <form
              onSubmit={(e) => {
                e.preventDefault()
                e.stopPropagation()
                form.handleSubmit()
              }}
              className="space-y-4"
            >
              {serverError && (
                <div className="rounded-md border border-destructive/50 bg-destructive/10 px-4 py-3 text-sm text-destructive">
                  {serverError}
                </div>
              )}

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
                      type="text"
                      placeholder="Username"
                      autoComplete="username"
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
        </CardContent>
      </Card>
    </div>
  )
}
