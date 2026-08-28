import "../../global.css"
import { BottomSheetModalProvider } from "@gorhom/bottom-sheet"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { useMigrations } from "drizzle-orm/expo-sqlite/migrator"
import { Stack } from "expo-router"
import { ActivityIndicator, Text, View } from "react-native"
import { GestureHandlerRootView } from "react-native-gesture-handler"
import { KeyboardProvider } from "react-native-keyboard-controller"
import { db } from "@/db"
import migrations from "@/drizzle/migrations"

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 0,
      gcTime: 1000 * 60 * 60 * 24,
      networkMode: "always",
    },
    mutations: {
      networkMode: "always",
    },
  },
})

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
    <QueryClientProvider client={queryClient}>
      <GestureHandlerRootView style={{ flex: 1 }}>
        <KeyboardProvider>
          <BottomSheetModalProvider>
            <Stack
              screenOptions={{
                headerStyle: { backgroundColor: "#141318" },
                headerTintColor: "#CABEFF",
                headerTitleStyle: { color: "#FFFFFF", fontWeight: "bold" },
                headerLargeTitleStyle: { color: "#FFFFFF", fontWeight: "bold" },
                headerShadowVisible: false,
                contentStyle: { backgroundColor: "#141318" },
              }}
            >
              <Stack.Screen
                name="index"
                options={{
                  title: "Folders",
                  headerLargeTitle: true,
                  headerShown: true,
                }}
              />
              <Stack.Screen
                name="folders/[id]"
                options={{
                  headerLargeTitle: true,
                  headerShown: true,
                  headerBackTitle: "Folders",
                }}
              />
              <Stack.Screen
                name="notes/[id]"
                options={{
                  headerShown: false,
                }}
              />
            </Stack>
          </BottomSheetModalProvider>
        </KeyboardProvider>
      </GestureHandlerRootView>
    </QueryClientProvider>
  )
}
