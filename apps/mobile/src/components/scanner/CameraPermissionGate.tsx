import * as Haptics from "expo-haptics"
import { Camera, Keyboard } from "lucide-react-native"
import { memo } from "react"
import { Pressable, Text, View } from "react-native"

export interface CameraPermissionGateProps {
  onRequestPermission: () => void
  onManualEntry: () => void
}

export const CameraPermissionGate = memo(function CameraPermissionGate({
  onRequestPermission,
  onManualEntry,
}: CameraPermissionGateProps) {
  const handleRequest = () => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
    onRequestPermission()
  }

  const handleManual = () => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    onManualEntry()
  }

  return (
    <View className="flex-1 items-center justify-center px-6 py-12">
      <View className="mb-6 size-20 items-center justify-center rounded-3xl bg-[#CABEFF]/15">
        <Camera size={36} color="#CABEFF" />
      </View>

      <Text className="text-center text-[22px] font-bold text-white">Camera Access Required</Text>
      <Text className="mt-2 text-center text-[14px] text-muted-foreground leading-relaxed">
        TNotes needs camera access to scan pairing QR codes displayed on your sync server.
      </Text>

      <View className="mt-8 w-full max-w-xs gap-3">
        <Pressable
          onPress={handleRequest}
          className="h-12 items-center justify-center rounded-full bg-[#CABEFF] active:opacity-85"
        >
          <Text className="text-[16px] font-semibold text-[#141318]">Enable Camera</Text>
        </Pressable>

        <Pressable
          onPress={handleManual}
          className="h-12 flex-row items-center justify-center gap-2 rounded-full bg-white/10 active:bg-white/15"
        >
          <Keyboard size={18} color="#E6E1E9" />
          <Text className="text-[15px] font-medium text-white">Enter URL Manually</Text>
        </Pressable>
      </View>
    </View>
  )
})
