import * as Crypto from "expo-crypto"
import { monotonicFactory } from "ulidx"

/**
 * Cryptographically secure PRNG using expo-crypto for React Native Hermes.
 */
const prng = () => {
  const buf = new Uint32Array(1)
  Crypto.getRandomValues(buf)
  return (buf[0] ?? 0) / 0xffffffff
}

/**
 * Generates monotonic ULIDs using expo-crypto.
 */
export const ulid = monotonicFactory(prng)
