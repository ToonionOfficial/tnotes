export interface SetupStatusResponse {
  is_configured: boolean
}

export interface SetupResponse {
  ok: boolean
  user_id: string
  username: string
  token: string
  device_id: string
  expires_at: number
}

export interface RegisterResponse {
  ok: boolean
  user_id: string
  username: string
  token: string
  device_id: string
  expires_at: number
}

export interface LoginResponse {
  token: string
  device_id: string
  expires_at: number
}

export interface MeResponse {
  user_id: string
  username: string
  device_id: string
  device_name: string
  platform: string
  has_paired_devices?: boolean
  paired_devices_count?: number
}

export interface LogoutResponse {
  ok: boolean
}

export interface PairingDataResponse {
  url: string
  token: string
  device_id: string
  user_id: string
  username: string
  pairing_code: string
  qr_svg: string
  qr_payload: string
  expires_at: number
}

export interface PairStatusResponse {
  paired: boolean
  device_id?: string
  device_name?: string
  username?: string
}
