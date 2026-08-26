import { Text, View } from "react-native"
import { BottomBar } from "@/components/BottomBar"

export default function Index() {
  return (
    <View className="flex-1 bg-background">
      <View className="flex-1 items-center justify-center">
        <Text className="text-foreground">Notes list</Text>
      </View>

      <BottomBar onSearchChange={() => {}} searchValue="" />
    </View>
  )
}
