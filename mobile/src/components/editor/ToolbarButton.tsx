import type { ReactNode } from "react"
import { Pressable, Text } from "react-native"
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
  activeColor = "#32285F",
  activeBgColor = "#CABEFF",
}: ToolbarButtonProps) {
  const iconColor = isDisabled ? "#605D66" : isActive ? activeColor : "#E6E1E9"

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
              backgroundColor: activeBgColor,
            }
          : undefined
      }
    >
      {icon ? (
        <ToolbarIcon name={icon} size={size} color={iconColor} />
      ) : label ? (
        <Text
          className={`text-[15px] font-medium ${
            isActive ? "font-semibold text-[#32285F]" : "text-foreground"
          }`}
          style={isActive ? { color: activeColor } : undefined}
        >
          {label}
        </Text>
      ) : (
        children
      )}
    </Pressable>
  )
}
