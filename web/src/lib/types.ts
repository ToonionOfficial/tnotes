export interface SetupStatusResponse {
  is_configured: boolean
}

export interface SetupResponse {
  ok: boolean
  user_id: string
  username: string
}

export interface LoginResponse {
  token: string
  device_id: string
  expires_at: number
}

export interface PairingDataResponse {
  url: string
  token: string
  device_id: string
  pairing_code: string
  qr_svg: string
  qr_payload: string
  expires_at: number
}
