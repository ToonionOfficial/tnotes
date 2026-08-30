import { zodResolver } from "@hookform/resolvers/zod"
import * as Haptics from "expo-haptics"
import { Camera, Globe, KeyRound } from "lucide-react-native"
import { memo } from "react"
import { Controller, useForm } from "react-hook-form"
import { Pressable, Text, TextInput, View } from "react-native"
import { z } from "zod"

const serverPairingSchema = z.object({
  url: z
    .string()
    .trim()
    .min(1, "Server URL is required")
    .refine(
      (val) => {
        try {
          const parsed = new URL(val.startsWith("http") ? val : `https://${val}`)
          return Boolean(parsed.hostname)
        } catch {
          return false
        }
      },
      { message: "Enter a valid URL" },
    ),
  codeOrToken: z.string().trim().min(1, "Pairing code or token is required"),
})

export type ServerPairingFormData = z.infer<typeof serverPairingSchema>

export interface ManualServerFormProps {
  onConnect: (payload: { url: string; token: string }) => void
  onSwitchToScanner: () => void
}

export const ManualServerForm = memo(function ManualServerForm({
  onConnect,
  onSwitchToScanner,
}: ManualServerFormProps) {
  const {
    control,
    handleSubmit,
    formState: { errors, isValid, isSubmitting },
  } = useForm<ServerPairingFormData>({
    resolver: zodResolver(serverPairingSchema),
    mode: "onChange",
    defaultValues: {
      url: "",
      codeOrToken: "",
    },
  })

  const onSubmit = (data: ServerPairingFormData) => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
    let cleanUrl = data.url.trim()
    if (!cleanUrl.startsWith("http://") && !cleanUrl.startsWith("https://")) {
      cleanUrl = `https://${cleanUrl}`
    }
    onConnect({
      url: cleanUrl,
      token: data.codeOrToken.trim(),
    })
  }

  return (
    <View className="flex-1 justify-center px-6 py-8">
      <Text className="text-[22px] font-bold text-white">Manual Connection</Text>
      <Text className="mt-1 text-[13px] text-muted-foreground">
        Enter your server address and the 6-digit pairing code or token from your dashboard.
      </Text>

      <View className="mt-6 gap-3">
        <View>
          <View
            className={`flex-row items-center rounded-2xl border bg-white/7 px-4 py-3 ${
              errors.url ? "border-red-400/60" : "border-transparent"
            }`}
          >
            <Globe size={18} color="#8E8D94" />
            <Controller
              control={control}
              name="url"
              render={({ field: { onChange, onBlur, value } }) => (
                <TextInput
                  value={value}
                  onChangeText={onChange}
                  onBlur={onBlur}
                  placeholder="https://sync.example.com"
                  placeholderTextColor="#8E8D94"
                  autoCapitalize="none"
                  autoCorrect={false}
                  keyboardType="url"
                  className="ml-3 flex-1 text-[16px] text-white"
                />
              )}
            />
          </View>
          {errors.url && (
            <Text className="mt-1 ml-3 text-[12px] text-red-400">{errors.url.message}</Text>
          )}
        </View>

        <View>
          <View
            className={`flex-row items-center rounded-2xl border bg-white/7 px-4 py-3 ${
              errors.codeOrToken ? "border-red-400/60" : "border-transparent"
            }`}
          >
            <KeyRound size={18} color="#8E8D94" />
            <Controller
              control={control}
              name="codeOrToken"
              render={({ field: { onChange, onBlur, value } }) => (
                <TextInput
                  value={value}
                  onChangeText={onChange}
                  onBlur={onBlur}
                  placeholder="6-Digit Code or Pairing Token"
                  placeholderTextColor="#8E8D94"
                  autoCapitalize="none"
                  autoCorrect={false}
                  className="ml-3 flex-1 text-[16px] text-white"
                />
              )}
            />
          </View>
          {errors.codeOrToken && (
            <Text className="mt-1 ml-3 text-[12px] text-red-400">{errors.codeOrToken.message}</Text>
          )}
        </View>
      </View>

      <View className="mt-6 gap-3">
        <Pressable
          disabled={!isValid || isSubmitting}
          onPress={handleSubmit(onSubmit)}
          className={`h-12 items-center justify-center rounded-full bg-[#CABEFF] ${
            !isValid || isSubmitting ? "opacity-40" : "active:opacity-85"
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
