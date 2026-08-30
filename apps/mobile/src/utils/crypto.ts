import { blake3 } from "@noble/hashes/blake3.js"
import { bytesToHex, utf8ToBytes } from "@noble/hashes/utils.js"

/**
 * Computes a Blake3 checksum for the given content string, matching the Rust core engine.
 */
export function computeChecksum(content: string): string {
  return bytesToHex(blake3(utf8ToBytes(content)))
}
