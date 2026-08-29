import * as Haptics from "expo-haptics"
import { Stack } from "expo-router"
import { Globe, KeyRound, Laptop, Monitor, Smartphone, Trash2, User } from "lucide-react-native"
import { ActivityIndicator, Alert, Pressable, ScrollView, Text, View } from "react-native"
import { SettingsRow } from "@/components/settings/SettingsRow"
import { SettingsSection } from "@/components/settings/SettingsSection"
import { type ConnectedDevice, useDevicesQuery, useRevokeDeviceMutation } from "@/hooks/useDevices"
import { useSyncState } from "@/hooks/useSyncState"

function getPlatformIcon(platform: string) {
  const p = platform.toLowerCase()
  if (p.includes("mobile") || p.includes("ios") || p.includes("android") || p.includes("phone")) {
    return <Smartphone size={20} color="#CABEFF" />
  }
  if (p.includes("web") || p.includes("browser")) {
    return <Globe size={20} color="#CABEFF" />
  }
  if (
    p.includes("desktop") ||
    p.includes("macos") ||
    p.includes("windows") ||
    p.includes("linux")
  ) {
    return <Laptop size={20} color="#CABEFF" />
  }
  return <Monitor size={20} color="#CABEFF" />
}

function formatRelativeTime(timestampMs: number): string {
  const now = Date.now()
  const diffMs = now - timestampMs

  if (diffMs < 60 * 1000) return "Active now"
  const diffMins = Math.floor(diffMs / (60 * 1000))
  if (diffMins < 60) return `${diffMins}m ago`
  const diffHours = Math.floor(diffMins / 60)
  if (diffHours < 24) return `${diffHours}h ago`
  const diffDays = Math.floor(diffHours / 24)
  if (diffDays === 1) return "Yesterday"
  return `${diffDays}d ago`
}

export default function AccountScreen() {
  const { data: syncStatus } = useSyncState()
  const isConnected = syncStatus?.isConnected ?? false
  const { data: devices = [], isLoading: isLoadingDevices } = useDevicesQuery(isConnected)
  const revokeMutation = useRevokeDeviceMutation()

  const handleChangePassword = () => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    Alert.alert(
      "Change Password",
      "Password management from the mobile app is coming soon. Please change your password from the web dashboard.",
      [{ text: "OK" }],
    )
  }

  const handleRevokeDevice = (device: ConnectedDevice) => {
    Alert.alert(
      "Revoke Device",
      `Are you sure you want to remove "${device.name}"? It will be disconnected from your account.`,
      [
        { text: "Cancel", style: "cancel" },
        {
          text: "Remove",
          style: "destructive",
          onPress: () => {
            void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
            void revokeMutation.mutateAsync(device.id)
          },
        },
      ],
    )
  }

  return (
    <View className="flex-1 bg-background">
      <Stack.Screen
        options={{
          title: "Account",
          headerLargeTitle: true,
          headerShadowVisible: false,
          headerStyle: { backgroundColor: "#141318" },
          headerTintColor: "#ffffff",
          headerTitleStyle: { color: "#FFFFFF", fontWeight: "bold" },
        }}
      />

      <ScrollView
        className="flex-1"
        contentInsetAdjustmentBehavior="automatic"
        contentContainerStyle={{
          paddingHorizontal: 20,
          paddingTop: 12,
          paddingBottom: 60,
        }}
      >
        {/* Profile Card */}
        <SettingsSection title="Profile">
          <View className="flex-row items-center gap-4 p-4">
            <View className="size-14 items-center justify-center rounded-2xl bg-[#CABEFF]/15">
              <User size={28} color="#CABEFF" />
            </View>
            <View className="flex-1">
              <Text className="text-[18px] font-bold text-white">
                {syncStatus?.username ?? "Local Account"}
              </Text>
              <Text className="pt-0.5 text-[13px] text-muted-foreground">
                {isConnected ? syncStatus?.serverUrl : "Offline / On-Device"}
              </Text>
            </View>
            <View className="flex-row items-center gap-1.5 rounded-full bg-white/10 px-2.5 py-1">
              <View
                className={`size-2 rounded-full ${isConnected ? "bg-[#22C55E]" : "bg-[#FFC107]"}`}
              />
              <Text className="text-[12px] font-medium text-white">
                {isConnected ? "Connected" : "Offline"}
              </Text>
            </View>
          </View>
        </SettingsSection>

        {/* Security Section */}
        <SettingsSection title="Security">
          <SettingsRow
            icon={<KeyRound size={19} color="#CABEFF" />}
            title="Change Password"
            subtitle="Update your master account password"
            onPress={handleChangePassword}
          />
        </SettingsSection>

        {/* Connected Devices Section */}
        <SettingsSection
          title={isConnected ? `Connected Devices (${devices.length})` : "Connected Devices"}
        >
          {!isConnected ? (
            <View className="p-5 items-center justify-center">
              <Smartphone size={32} color="#8E8D94" />
              <Text className="mt-3 text-center text-[14px] font-medium text-white">
                No Server Connected
              </Text>
              <Text className="mt-1 text-center text-[12px] text-muted-foreground px-4">
                Connect this device to a sync server in Settings to view and manage all linked
                devices.
              </Text>
            </View>
          ) : isLoadingDevices ? (
            <View className="py-8 items-center justify-center">
              <ActivityIndicator size="small" color="#CABEFF" />
            </View>
          ) : devices.length === 0 ? (
            <View className="p-4">
              <Text className="text-[14px] text-muted-foreground text-center">
                No connected devices found.
              </Text>
            </View>
          ) : (
            devices.map((device, index) => {
              const isCurrent = device.is_current
              const relativeTime = formatRelativeTime(device.last_seen_at)

              const badgeNode = isCurrent ? (
                <View className="rounded-full bg-[#22C55E]/15 px-2.5 py-0.5">
                  <Text className="text-[11px] font-semibold text-[#22C55E]">This Device</Text>
                </View>
              ) : (
                <Pressable
                  onPress={() => handleRevokeDevice(device)}
                  hitSlop={8}
                  className="p-1.5 active:opacity-50"
                >
                  <Trash2 size={16} color="#FF5A52" />
                </Pressable>
              )

              return (
                <SettingsRow
                  key={device.id}
                  icon={getPlatformIcon(device.platform)}
                  title={device.name}
                  subtitle={isCurrent ? "Active now" : `Last active ${relativeTime}`}
                  badge={badgeNode}
                  showDivider={index < devices.length - 1}
                  showChevron={false}
                />
              )
            })
          )}
        </SettingsSection>
      </ScrollView>
    </View>
  )
}
