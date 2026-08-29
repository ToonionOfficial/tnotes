import * as Haptics from "expo-haptics"
import { ChevronRight } from "lucide-react-native"
import type { ReactNode } from "react"
import { memo } from "react"
import { Pressable, Switch, Text, View } from "react-native"

export interface SettingsRowProps {
  icon?: ReactNode
  title: string
  subtitle?: string
  value?: string
  badge?: ReactNode
  isSwitch?: boolean
  switchValue?: boolean
  onSwitchChange?: (value: boolean) => void
  showChevron?: boolean
  onPress?: () => void
  isDestructive?: boolean
  showDivider?: boolean
  disabled?: boolean
}

export const SettingsRow = memo(function SettingsRow({
  icon,
  title,
  subtitle,
  value,
  badge,
  isSwitch = false,
  switchValue = false,
  onSwitchChange,
  showChevron = true,
  onPress,
  isDestructive = false,
  showDivider = false,
  disabled = false,
}: SettingsRowProps) {
  const handlePress = () => {
    if (disabled) return
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    onPress?.()
  }

  const handleSwitchChange = (val: boolean) => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    onSwitchChange?.(val)
  }

  return (
    <View>
      <Pressable
        disabled={disabled || isSwitch || !onPress}
        onPress={handlePress}
        className={`flex-row items-center justify-between px-4 py-3.5 ${
          onPress && !disabled ? "active:bg-white/10" : ""
        } ${disabled ? "opacity-40" : ""}`}
      >
        <View className="flex-1 flex-row items-center gap-3">
          {icon && <View className="w-7 items-center justify-center">{icon}</View>}
          <View className="flex-1">
            <Text
              className={`text-[16px] font-medium ${
                isDestructive ? "text-[#FF5A52]" : "text-white"
              }`}
              numberOfLines={1}
            >
              {title}
            </Text>
            {subtitle ? (
              <Text className="text-[12px] text-muted-foreground pt-0.5" numberOfLines={1}>
                {subtitle}
              </Text>
            ) : null}
          </View>
        </View>

        <View className="flex-row items-center gap-2 pl-3">
          {badge}
          {value ? <Text className="text-[14px] text-muted-foreground">{value}</Text> : null}
          {isSwitch ? (
            <Switch
              value={switchValue}
              onValueChange={handleSwitchChange}
              trackColor={{ false: "#3E3D46", true: "#CABEFF" }}
              thumbColor={switchValue ? "#141318" : "#E6E1E9"}
            />
          ) : showChevron && onPress ? (
            <ChevronRight size={16} color="#6E6D77" />
          ) : null}
        </View>
      </Pressable>

      {showDivider && <View className="ml-14 h-[0.5px] bg-white/8" />}
    </View>
  )
})
