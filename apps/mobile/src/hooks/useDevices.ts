import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { getOrCreateDeviceId, getSyncMeta } from "@/db/queries"

export interface ConnectedDevice {
  id: string
  name: string
  platform: string
  last_seen_at: number
  created_at: number
  is_current: boolean
}

export const deviceKeys = {
  all: ["connectedDevices"] as const,
}

export async function fetchConnectedDevicesAsync(): Promise<ConnectedDevice[]> {
  const serverUrl = getSyncMeta("server_url")
  const authToken = getSyncMeta("auth_token")
  const deviceId = getOrCreateDeviceId()

  if (!serverUrl || !authToken) {
    return []
  }

  const res = await fetch(`${serverUrl}/api/devices`, {
    headers: {
      Authorization: `Bearer ${authToken}`,
      "X-Device-ID": deviceId,
    },
  })

  if (!res.ok) {
    if (res.status === 401) {
      return []
    }
    throw new Error(`Failed to fetch connected devices: ${res.statusText}`)
  }

  return (await res.json()) as ConnectedDevice[]
}

export async function revokeDeviceAsync(targetDeviceId: string): Promise<void> {
  const serverUrl = getSyncMeta("server_url")
  const authToken = getSyncMeta("auth_token")
  const deviceId = getOrCreateDeviceId()

  if (!serverUrl || !authToken) {
    throw new Error("Device is not connected to a server")
  }

  const res = await fetch(`${serverUrl}/api/devices/${targetDeviceId}`, {
    method: "DELETE",
    headers: {
      Authorization: `Bearer ${authToken}`,
      "X-Device-ID": deviceId,
    },
  })

  if (!res.ok) {
    throw new Error(`Failed to revoke device: ${res.statusText}`)
  }
}

export function useDevicesQuery(enabled = true) {
  const serverUrl = getSyncMeta("server_url")
  const authToken = getSyncMeta("auth_token")
  const isConnected = Boolean(serverUrl && authToken)

  return useQuery({
    queryKey: deviceKeys.all,
    queryFn: fetchConnectedDevicesAsync,
    enabled: isConnected && enabled,
    staleTime: 1000 * 30, // 30s
    refetchOnMount: "always",
  })
}

export function useRevokeDeviceMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (targetDeviceId: string) => revokeDeviceAsync(targetDeviceId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: deviceKeys.all })
    },
  })
}
