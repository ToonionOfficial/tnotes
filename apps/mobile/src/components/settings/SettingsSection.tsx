import type { ReactNode } from "react"
import { memo } from "react"
import { Text, View } from "react-native"

export interface SettingsSectionProps {
  title?: string
  children: ReactNode
  className?: string
}

export const SettingsSection = memo(function SettingsSection({
  title,
  children,
  className = "",
}: SettingsSectionProps) {
  return (
    <View className={`mb-5 ${className}`}>
      {title && (
        <Text className="mb-2 px-2 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/60">
          {title}
        </Text>
      )}
      <View className="overflow-hidden rounded-3xl bg-card border border-border/40">
        {children}
      </View>
    </View>
  )
})
