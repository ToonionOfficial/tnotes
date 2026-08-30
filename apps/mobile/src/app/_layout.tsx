import "../../global.css"
import { BottomSheetModalProvider } from "@gorhom/bottom-sheet"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { useMigrations } from "drizzle-orm/expo-sqlite/migrator"
import { Stack } from "expo-router"
import { StatusBar } from "expo-status-bar"
import { ActivityIndicator, Text, View } from "react-native"
import { GestureHandlerRootView } from "react-native-gesture-handler"
import { KeyboardProvider } from "react-native-keyboard-controller"
import { db } from "@/db"
import migrations from "@/drizzle/migrations"
import { AppThemeProvider, useAppTheme } from "@/hooks/useAppTheme"
import { useAutoSyncRunner, useWebSocketSync } from "@/services/sync"

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60,
      gcTime: 1000 * 60 * 5,
      networkMode: "always",
    },
    mutations: {
      networkMode: "always",
    },
  },
})

function AutoSyncManager() {
  useAutoSyncRunner()
  useWebSocketSync()
  return null
}

function RootNavigation() {
  const { colors, isDarkMode } = useAppTheme()

  return (
    <BottomSheetModalProvider>
      <StatusBar style={isDarkMode ? "light" : "dark"} />
      <Stack
        screenOptions={{
          headerStyle: { backgroundColor: colors.background },
          headerTintColor: colors.foreground,
          headerTitleStyle: { color: colors.foreground, fontWeight: "bold" },
          headerLargeTitleStyle: {
            color: colors.foreground,
            fontWeight: "bold",
          },
          headerShadowVisible: false,
          contentStyle: { backgroundColor: colors.background },
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
            headerBackButtonDisplayMode: "minimal",
          }}
        />
        <Stack.Screen
          name="notes/[id]"
          options={{
            headerShown: false,
          }}
        />
        <Stack.Screen
          name="settings"
          options={{
            presentation: "modal",
            title: "Settings",
            headerShown: true,
          }}
        />
        <Stack.Screen
          name="account"
          options={{
            presentation: "modal",
            title: "Account",
            headerLargeTitle: true,
            headerShown: true,
          }}
        />
      </Stack>
    </BottomSheetModalProvider>
  )
}

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
      <AppThemeProvider>
        <AutoSyncManager />
        <GestureHandlerRootView style={{ flex: 1 }}>
          <KeyboardProvider>
            <RootNavigation />
          </KeyboardProvider>
        </GestureHandlerRootView>
      </AppThemeProvider>
    </QueryClientProvider>
  )
}
