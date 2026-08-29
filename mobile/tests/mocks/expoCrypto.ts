export const getRandomValues = <T extends ArrayBufferView>(array: T): T => {
  return globalThis.crypto.getRandomValues(array as never) as T
}
