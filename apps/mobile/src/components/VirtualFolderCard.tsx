import { ChevronRight } from "lucide-react-native"
import type { ReactNode } from "react"
import { memo } from "react"
import { Pressable, Text, View } from "react-native"
import Animated, { useAnimatedStyle, withTiming } from "react-native-reanimated"

export interface VirtualFolderCardProps {
  title: string
  icon: ReactNode
  count: number
  isEditing: boolean
  onPress: () => void
  className?: string
  textColor?: string
}

export const VirtualFolderCard = memo(function VirtualFolderCard({
  title,
  icon,
  count,
  isEditing,
  onPress,
  className = "",
  textColor = "text-foreground",
}: VirtualFolderCardProps) {
  const animatedStyle = useAnimatedStyle(() => ({
    opacity: withTiming(isEditing ? 0.4 : 1, { duration: 250 }),
  }))

  return (
    <Animated.View
      style={animatedStyle}
      className={`overflow-hidden rounded-3xl bg-white/7 ${className}`}
    >
      <Pressable
        disabled={isEditing}
        onPress={onPress}
        className="flex-row items-center justify-between px-4 py-3.5 active:bg-white/12"
      >
        <View className="flex-row items-center gap-3">
          <View className="size-8 items-center justify-center rounded-lg">{icon}</View>
          <Text className={`text-[17px] ${textColor}`}>{title}</Text>
        </View>
        <View className="flex-row items-center gap-1.5">
          <Text className="text-[15px] text-muted-foreground">{count}</Text>
          <ChevronRight size={16} color="#8E8C99" />
        </View>
      </Pressable>
    </Animated.View>
  )
})
