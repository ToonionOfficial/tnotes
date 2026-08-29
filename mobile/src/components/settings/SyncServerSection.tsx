import { Cloud, Globe, RefreshCw } from "lucide-react-native"
import { memo, useState } from "react"
import { Text, View } from "react-native"
import { SettingsRow } from "./SettingsRow"
import { SettingsSection } from "./SettingsSection"

export interface SyncServerSectionProps {
  isConnected?: boolean
  serverUrl?: string
  onPressConnectServer?: () => void
}

export const SyncServerSection = memo(function SyncServerSection({
  isConnected = false,
  serverUrl,
  onPressConnectServer,
}: SyncServerSectionProps) {
  const [autoSync, setAutoSync] = useState(true)

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
        subtitle={isConnected ? serverUrl : undefined}
        badge={statusBadge}
        showDivider
      />
      <SettingsRow
        icon={<Globe size={20} color="#E6E1E9" />}
        title="Connect Server"
        onPress={onPressConnectServer}
        showDivider
      />
      <SettingsRow
        icon={<RefreshCw size={19} color="#E6E1E9" />}
        title="Auto-Sync"
        isSwitch
        switchValue={autoSync}
        onSwitchChange={setAutoSync}
      />
    </SettingsSection>
  )
})
