export function getApiBase() {
  if (typeof window !== 'undefined') {
    return ''
  }
  return process.env.NOTAT_SERVER_URL || 'http://localhost:8787'
}

export async function apiFetch<T>(
  path: string,
  options?: RequestInit,
): Promise<T> {
  const base = getApiBase()
  const res = await fetch(`${base}${path}`, {
    credentials: 'same-origin',
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  })

  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || `Request failed: ${res.status}`)
  }

  return res.json()
}
