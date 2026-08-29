import * as Haptics from "expo-haptics"
import { Camera, Globe, KeyRound } from "lucide-react-native"
import { memo, useState } from "react"
import { Pressable, Text, TextInput, View } from "react-native"

export interface ManualServerFormProps {
  onConnect: (payload: { url: string; token?: string }) => void
  onSwitchToScanner: () => void
}

export const ManualServerForm = memo(function ManualServerForm({
  onConnect,
  onSwitchToScanner,
}: ManualServerFormProps) {
  const [url, setUrl] = useState("")
  const [token, setToken] = useState("")

  const handleConnect = () => {
    if (!url.trim()) return
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
    onConnect({ url: url.trim(), token: token.trim() || undefined })
  }

  return (
    <View className="flex-1 justify-center px-6 py-8">
      <Text className="text-[22px] font-bold text-white">Manual Connection</Text>
      <Text className="mt-1 text-[13px] text-muted-foreground">
        Enter the address and access token of your self-hosted sync server.
      </Text>

      {/* Input Group */}
      <View className="mt-6 gap-3">
        {/* Server URL Input */}
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

        {/* Token Input */}
        <View className="flex-row items-center rounded-2xl bg-white/7 px-4 py-3">
          <KeyRound size={18} color="#8E8D94" />
          <TextInput
            value={token}
            onChangeText={setToken}
            placeholder="Pairing Token (Optional)"
            placeholderTextColor="#8E8D94"
            autoCapitalize="none"
            autoCorrect={false}
            secureTextEntry
            className="ml-3 flex-1 text-[16px] text-white"
          />
        </View>
      </View>

      {/* Actions */}
      <View className="mt-6 gap-3">
        <Pressable
          disabled={!url.trim()}
          onPress={handleConnect}
          className={`h-12 items-center justify-center rounded-full bg-[#CABEFF] ${
            !url.trim() ? "opacity-40" : "active:opacity-85"
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
