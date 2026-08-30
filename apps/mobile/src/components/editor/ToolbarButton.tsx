import type { ReactNode } from "react"
import { Pressable, Text } from "react-native"
import { useAppTheme } from "@/hooks/useAppTheme"
import { ToolbarIcon, type ToolbarIconName } from "./ToolbarIcon"

interface ToolbarButtonProps {
  icon?: ToolbarIconName
  label?: string
  children?: ReactNode
  isActive?: boolean
  isDisabled?: boolean
  onPress: () => void
  size?: number
  variant?: "pill" | "square" | "compact"
  activeColor?: string
  activeBgColor?: string
}

export function ToolbarButton({
  icon,
  label,
  children,
  isActive = false,
  isDisabled = false,
  onPress,
  size = 21,
  variant = "square",
  activeColor,
  activeBgColor,
}: ToolbarButtonProps) {
  const { colors, isDarkMode } = useAppTheme()

  const resolvedActiveBg = activeBgColor ?? colors.primary
  const resolvedActiveColor = activeColor ?? (isDarkMode ? "#32285F" : "#FFFFFF")

  const iconColor = isDisabled
    ? colors.mutedForeground
    : isActive
      ? resolvedActiveColor
      : colors.foreground

  const buttonDimensions =
    variant === "compact"
      ? "h-10 px-2.5 min-w-[40px]"
      : variant === "pill"
        ? "h-10 px-3.5 min-w-[48px]"
        : "size-10"

  return (
    <Pressable
      onPress={onPress}
      disabled={isDisabled}
      hitSlop={6}
      className={`items-center justify-center rounded-3xl transition-colors active:opacity-60 ${buttonDimensions} ${
        isActive ? "" : "bg-transparent"
      } ${isDisabled ? "opacity-35" : "opacity-100"}`}
      style={
        isActive
          ? {
              backgroundColor: resolvedActiveBg,
            }
          : undefined
      }
    >
      {icon ? (
        <ToolbarIcon name={icon} size={size} color={iconColor} />
      ) : label ? (
        <Text
          className={`text-[15px] font-medium ${isActive ? "font-semibold" : "text-foreground"}`}
          style={isActive ? { color: resolvedActiveColor } : undefined}
        >
          {label}
        </Text>
      ) : (
        children
      )}
    </Pressable>
  )
}
