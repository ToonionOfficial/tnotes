import * as Haptics from "expo-haptics"
import { Camera, Globe, KeyRound } from "lucide-react-native"
import { memo, useState } from "react"
import { Pressable, Text, TextInput, View } from "react-native"

export interface ManualServerFormProps {
  onConnect: (payload: { url: string; token: string }) => void
  onSwitchToScanner: () => void
}

export const ManualServerForm = memo(function ManualServerForm({
  onConnect,
  onSwitchToScanner,
}: ManualServerFormProps) {
  const [url, setUrl] = useState("")
  const [codeOrToken, setCodeOrToken] = useState("")

  const isValid = url.trim().length > 0 && codeOrToken.trim().length > 0

  const handleConnect = () => {
    if (!isValid) return
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
    onConnect({
      url: url.trim(),
      token: codeOrToken.trim(),
    })
  }

  return (
    <View className="flex-1 justify-center px-6 py-8">
      <Text className="text-[22px] font-bold text-white">Manual Connection</Text>
      <Text className="mt-1 text-[13px] text-muted-foreground">
        Enter your server address and the 6-digit pairing code or token from your dashboard.
      </Text>

      <View className="mt-6 gap-3">
        <View className="flex-row items-center rounded-2xl bg-white/7 px-4 py-3">
          <Globe size={18} color="#8E8D94" />
          <TextInput
            value={url}
            onChangeText={setUrl}
            placeholder="https://sync.example.com"
            placeholderTextColor="#8E8D94"
            autoCapitalize="none"
            autoCorrect={false}
            keyboardType="url"
            className="ml-3 flex-1 text-[16px] text-white"
          />
        </View>

        <View className="flex-row items-center rounded-2xl bg-white/7 px-4 py-3">
          <KeyRound size={18} color="#8E8D94" />
          <TextInput
            value={codeOrToken}
            onChangeText={setCodeOrToken}
            placeholder="6-Digit Code or Pairing Token"
            placeholderTextColor="#8E8D94"
            autoCapitalize="none"
            autoCorrect={false}
            className="ml-3 flex-1 text-[16px] text-white"
          />
        </View>
      </View>

      <View className="mt-6 gap-3">
        <Pressable
          disabled={!isValid}
          onPress={handleConnect}
          className={`h-12 items-center justify-center rounded-full bg-[#CABEFF] ${
            !isValid ? "opacity-40" : "active:opacity-85"
          }`}
        >
          <Text className="text-[16px] font-semibold text-[#141318]">Connect Server</Text>
        </Pressable>

        <Pressable
          onPress={() => {
            void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
            onSwitchToScanner()
          }}
          className="h-12 flex-row items-center justify-center gap-2 rounded-full bg-white/10 active:bg-white/15"
        >
          <Camera size={18} color="#E6E1E9" />
          <Text className="text-[15px] font-medium text-white">Switch to QR Scanner</Text>
        </Pressable>
      </View>
    </View>
  )
})
