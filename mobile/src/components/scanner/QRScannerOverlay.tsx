import * as Haptics from "expo-haptics"
import { Check, Keyboard, X, Zap, ZapOff } from "lucide-react-native"
import { memo } from "react"
import { Pressable, StyleSheet, Text, View } from "react-native"
import { useSafeAreaInsets } from "react-native-safe-area-context"

export interface QRScannerOverlayProps {
  torchOn: boolean
  isSuccess?: boolean
  onToggleTorch: () => void
  onClose: () => void
  onManualEntry: () => void
}

const FRAME_SIZE = 260

export const QRScannerOverlay = memo(function QRScannerOverlay({
  torchOn,
  isSuccess = false,
  onToggleTorch,
  onClose,
  onManualEntry,
}: QRScannerOverlayProps) {
  const insets = useSafeAreaInsets()
  const frameColor = isSuccess ? "#22C55E" : "#CABEFF"

  return (
    <View style={StyleSheet.absoluteFill} pointerEvents="box-none">
      {/* Top Header Bar */}
      <View
        style={{ paddingTop: Math.max(insets.top, 16) }}
        className="flex-row items-center justify-between px-6 pb-4"
      >
        <Pressable
          onPress={() => {
            void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
            onClose()
          }}
          hitSlop={8}
          className="size-10 items-center justify-center rounded-full bg-black/50 active:bg-black/70"
        >
          <X size={20} color="#FFFFFF" />
        </Pressable>

        <Text className="text-[17px] font-semibold text-white">Pair Server</Text>

        <Pressable
          onPress={() => {
            void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
            onToggleTorch()
          }}
          hitSlop={8}
          className="size-10 items-center justify-center rounded-full bg-black/50 active:bg-black/70"
        >
          {torchOn ? (
            <Zap size={20} color="#FFC107" fill="#FFC107" />
          ) : (
            <ZapOff size={20} color="#FFFFFF" />
          )}
        </Pressable>
      </View>

      {/* Center Viewfinder Target */}
      <View className="flex-1 items-center justify-center">
        <View
          style={{ width: FRAME_SIZE, height: FRAME_SIZE }}
          className="relative items-center justify-center"
        >
          {/* 4-Corner Target Brackets */}
          <View
            style={{ borderColor: frameColor }}
            className="absolute left-0 top-0 size-8 border-l-[3.5px] border-t-[3.5px] rounded-tl-2xl"
          />
          <View
            style={{ borderColor: frameColor }}
            className="absolute right-0 top-0 size-8 border-r-[3.5px] border-t-[3.5px] rounded-tr-2xl"
          />
          <View
            style={{ borderColor: frameColor }}
            className="absolute bottom-0 left-0 size-8 border-b-[3.5px] border-l-[3.5px] rounded-bl-2xl"
          />
          <View
            style={{ borderColor: frameColor }}
            className="absolute bottom-0 right-0 size-8 border-b-[3.5px] border-r-[3.5px] rounded-br-2xl"
          />

          {isSuccess && (
            <View className="size-16 items-center justify-center rounded-full bg-[#22C55E]">
              <Check size={32} color="#FFFFFF" strokeWidth={3} />
            </View>
          )}
        </View>

        {/* Guidance Text */}
        <Text className="mt-8 text-center text-[17px] font-semibold text-white">
          {isSuccess ? "Server Pairing Verified" : "Scan Pairing Code"}
        </Text>
        <Text className="mt-1 px-8 text-center text-[13px] text-white/70">
          {isSuccess
            ? "Connecting to your sync backend..."
            : "Point camera at the QR code on your server or web dashboard"}
        </Text>
      </View>

      {/* Bottom Manual Entry Action */}
      <View
        style={{ paddingBottom: Math.max(insets.bottom + 16, 24) }}
        className="items-center px-6"
      >
        <Pressable
          onPress={() => {
            void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
            onManualEntry()
          }}
          className="flex-row items-center gap-2 rounded-full bg-black/60 px-5 py-3 active:bg-black/80"
        >
          <Keyboard size={17} color="#E6E1E9" />
          <Text className="text-[14px] font-medium text-white">Enter Server URL Manually</Text>
        </Pressable>
      </View>
    </View>
  )
})
