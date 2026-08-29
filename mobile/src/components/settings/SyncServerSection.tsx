import { Cloud, Globe, LogOut, RefreshCw, RotateCw } from "lucide-react-native"
import { memo } from "react"
import { ActivityIndicator, Text, View } from "react-native"
import { SettingsRow } from "./SettingsRow"
import { SettingsSection } from "./SettingsSection"

export interface SyncServerSectionProps {
  isConnected?: boolean
  serverUrl?: string | null
  isSyncing?: boolean
  autoSync?: boolean
  onToggleAutoSync?: (enabled: boolean) => void
  onPressConnectServer?: () => void
  onPressSyncNow?: () => void
  onPressDisconnect?: () => void
}

export const SyncServerSection = memo(function SyncServerSection({
  isConnected = false,
  serverUrl,
  isSyncing = false,
  autoSync = true,
  onToggleAutoSync,
  onPressConnectServer,
  onPressSyncNow,
  onPressDisconnect,
}: SyncServerSectionProps) {
  const statusBadge = (
    <View className="flex-row items-center gap-1.5 rounded-full bg-white/10 px-2.5 py-1">
      <View className={`size-2 rounded-full ${isConnected ? "bg-[#22C55E]" : "bg-[#FFC107]"}`} />
      <Text className="text-[12px] font-medium text-white">
        {isConnected ? "Connected" : "Offline"}
      </Text>
    </View>
  )

  return (
    <SettingsSection title="Sync">
      <SettingsRow
        icon={<Cloud size={20} color="#E6E1E9" />}
        title="Sync Status"
        subtitle={isConnected && serverUrl ? serverUrl : undefined}
        badge={statusBadge}
        showDivider
      />

      <SettingsRow
        icon={<RefreshCw size={19} color="#E6E1E9" />}
        title="Auto-Sync"
        subtitle="Sync changes automatically when connected"
        isSwitch
        switchValue={autoSync}
        onSwitchChange={onToggleAutoSync}
        showDivider={isConnected}
      />

      {!isConnected ? (
        <SettingsRow
          icon={<Globe size={20} color="#E6E1E9" />}
          title="Connect Server"
          onPress={onPressConnectServer}
        />
      ) : (
        <>
          <SettingsRow
            icon={<RotateCw size={19} color="#CABEFF" />}
            title="Sync Now"
            subtitle="Send & receive latest changes"
            badge={isSyncing ? <ActivityIndicator size="small" color="#CABEFF" /> : undefined}
            onPress={onPressSyncNow}
            disabled={isSyncing}
            showDivider
          />
          <SettingsRow
            icon={<LogOut size={20} color="#FF5A52" />}
            title="Disconnect Server"
            isDestructive
            onPress={onPressDisconnect}
          />
        </>
      )}
    </SettingsSection>
  )
})
