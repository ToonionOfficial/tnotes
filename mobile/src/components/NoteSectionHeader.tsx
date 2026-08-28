import React from "react"
import { Text, View } from "react-native"

interface NoteSectionHeaderProps {
  title: string
}

function NoteSectionHeader({ title }: NoteSectionHeaderProps) {
  return (
    <View className="mb-1.5 mt-5 px-1">
      <Text className="text-[13px] font-semibold uppercase tracking-wider text-muted-foreground/80">
        {title}
      </Text>
    </View>
  )
}

export default React.memo(NoteSectionHeader)
