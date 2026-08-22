import { createFileRoute, Link, redirect, useNavigate } from '@tanstack/react-router'
import { useEffect, useState } from 'react'
import { NotebookPen, Loader2 } from 'lucide-react'
import { useForm } from '@tanstack/react-form'
import { z } from 'zod'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { apiFetch } from '@/lib/api'
import type { MeResponse, SetupResponse, SetupStatusResponse } from '@/lib/types'

export const Route = createFileRoute('/setup')({
  beforeLoad: async () => {
    // Only perform checks on client where cookies/network exist
    if (typeof window === 'undefined') return

    let isConfigured = false
    try {
      const status = await apiFetch<SetupStatusResponse>('/api/setup/status')
      isConfigured = status.is_configured
    } catch {
      // In case of network error during prefetch, allow page to handle it
    }

    if (isConfigured) {
      throw redirect({ to: '/login' })
    }
  },
  component: SetupPage,
})

const usernameSchema = z
  .string()
  .trim()
  .min(1, 'Username is required')
  .max(64, 'Username must be at most 64 characters')

const passwordSchema = z
  .string()
  .min(8, 'Password must be at least 8 characters long')
  .max(128, 'Password must be at most 128 characters long')

function SetupPage() {
  const navigate = useNavigate()
  const [checkingStatus, setCheckingStatus] = useState(true)
  const [serverError, setServerError] = useState('')

  // Client check: if already configured or logged in, redirect away
  useEffect(() => {
    let mounted = true
    Promise.allSettled([
      apiFetch<SetupStatusResponse>('/api/setup/status'),
      apiFetch<MeResponse>('/api/me'),
    ]).then(([statusRes, meRes]) => {
      if (!mounted) return

      if (meRes.status === 'fulfilled') {
        navigate({ to: '/', replace: true })
        return
      }

      if (statusRes.status === 'fulfilled' && statusRes.value.is_configured) {
        navigate({ to: '/login', replace: true })
        return
      }

      setCheckingStatus(false)
    })

    return () => {
      mounted = false
    }
  }, [navigate])

  const form = useForm({
    defaultValues: {
      username: '',
      password: '',
      confirmPassword: '',
    },
    onSubmit: async ({ value }) => {
      setServerError('')
      if (value.password !== value.confirmPassword) {
        return
      }

      try {
        await apiFetch<SetupResponse>('/api/setup', {
          method: 'POST',
          body: JSON.stringify({
            username: value.username.trim(),
            password: value.password,
          }),
        })
        // Automatically logged in via httpOnly cookie -> navigate to pair onboarding
        navigate({ to: '/pair', search: { welcome: true } })
      } catch (err) {
        setServerError(err instanceof Error ? err.message : 'Setup failed')
      }
    },
  })

  if (checkingStatus) {
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
          <CardTitle className="text-2xl font-bold">Welcome to TNotes</CardTitle>
          <CardDescription>
            Create your owner account to start taking notes and syncing devices.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
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
                    placeholder="e.g. alex"
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
                    placeholder="At least 8 characters"
                    autoComplete="new-password"
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
              name="confirmPassword"
              validators={{
                onChangeListenTo: ['password'],
                onChange: ({ value, fieldApi }) => {
                  if (!value) return 'Please confirm your password'
                  if (value !== fieldApi.form.getFieldValue('password')) {
                    return 'Passwords do not match'
                  }
                  return undefined
                },
              }}
            >
              {(field) => (
                <div className="space-y-2">
                  <Label htmlFor={field.name}>Confirm Password</Label>
                  <Input
                    id={field.name}
                    name={field.name}
                    type="password"
                    placeholder="Re-enter your password"
                    autoComplete="new-password"
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

            <form.Subscribe
              selector={(state) => [state.canSubmit, state.isSubmitting]}
            >
              {([canSubmit, isSubmitting]) => (
                <Button
                  type="submit"
                  className="w-full"
                  disabled={!canSubmit || isSubmitting}
                >
                  {isSubmitting ? (
                    <>
                      <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                      Setting up…
                    </>
                  ) : (
                    'Create Account & Continue'
                  )}
                </Button>
              )}
            </form.Subscribe>
          </form>

          <div className="text-center text-xs text-muted-foreground pt-1 border-t border-border/50">
            Already have an account?{' '}
            <Link
              to="/login"
              className="font-medium text-primary hover:underline underline-offset-4"
            >
              Sign in
            </Link>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
