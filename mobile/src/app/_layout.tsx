import "../../global.css"
import { Stack } from "expo-router"
import { KeyboardProvider } from "react-native-keyboard-controller"

export default function RootLayout() {
  return (
    <KeyboardProvider>
      <Stack screenOptions={{ headerShown: false }}>
        <Stack.Screen name="index" />
      </Stack>
    </KeyboardProvider>
  )
}
