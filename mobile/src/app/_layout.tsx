import "../../global.css"
import { BottomSheetModalProvider } from "@gorhom/bottom-sheet"
import { useMigrations } from "drizzle-orm/expo-sqlite/migrator"
import { Stack } from "expo-router"
import { ActivityIndicator, Text, View } from "react-native"
import { GestureHandlerRootView } from "react-native-gesture-handler"
import { KeyboardProvider } from "react-native-keyboard-controller"
import { db } from "@/db"
import migrations from "@/drizzle/migrations"

export default function RootLayout() {
  const { success, error } = useMigrations(
    db,
    migrations as unknown as Parameters<typeof useMigrations>[1],
  )

  if (error) {
    return (
      <View className="flex-1 items-center justify-center bg-background p-4">
        <Text className="text-red-500 font-semibold text-center">
          Migration Error: {error.message}
        </Text>
      </View>
    )
  }

  if (!success) {
    return (
      <View className="flex-1 items-center justify-center bg-background">
        <ActivityIndicator size="large" color="#ffffff" />
      </View>
    )
  }

  return (
    <GestureHandlerRootView style={{ flex: 1 }}>
      <KeyboardProvider>
        <BottomSheetModalProvider>
          <Stack screenOptions={{ headerShown: false }}>
            <Stack.Screen name="index" />
          </Stack>
        </BottomSheetModalProvider>
      </KeyboardProvider>
    </GestureHandlerRootView>
  )
}
