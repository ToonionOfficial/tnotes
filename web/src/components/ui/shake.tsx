import { type ReactNode, useCallback, useEffect, useRef } from 'react'

import { cn } from '@/lib/utils'

export function ShakeStyles() {
  return (
    <style>{`@keyframes shake-error{0%{transform:perspective(700px) translate3d(0,0,0) rotateY(0deg)}12%{transform:perspective(700px) translate3d(-7px,0,0) rotateY(-5deg)}26%{transform:perspective(700px) translate3d(6px,0,0) rotateY(4deg)}41%{transform:perspective(700px) translate3d(-5px,0,0) rotateY(-3deg)}56%{transform:perspective(700px) translate3d(4px,0,0) rotateY(2deg)}70%{transform:perspective(700px) translate3d(-2px,0,0) rotateY(-1deg)}84%{transform:perspective(700px) translate3d(1px,0,0) rotateY(.5deg)}100%{transform:perspective(700px) translate3d(0,0,0) rotateY(0deg)}}.shake-error{animation:shake-error .5s cubic-bezier(.36,.07,.19,.97) both;transform-style:preserve-3d;backface-visibility:hidden}@media(prefers-reduced-motion:reduce){.shake-error{animation:none}}`}</style>
  )
}

export function useShake<T extends HTMLElement = HTMLDivElement>() {
  const ref = useRef<T>(null)
  const shake = useCallback(() => {
    const el = ref.current
    if (!el) return
    el.classList.remove('shake-error')
    void el.offsetWidth
    el.classList.add('shake-error')
  }, [])
  return { ref, shake }
}

interface ShakeProps {
  signal: unknown
  children: ReactNode
  className?: string
}

export default function Shake({ signal, children, className }: ShakeProps) {
  const { ref, shake } = useShake<HTMLDivElement>()
  const prev = useRef(signal)

  useEffect(() => {
    if (prev.current !== signal && signal) shake()
    prev.current = signal
  }, [signal, shake])

  return (
    <>
      <ShakeStyles />
      <div ref={ref} className={cn(className)}>
        {children}
      </div>
    </>
  )
}
