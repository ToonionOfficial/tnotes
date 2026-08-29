import { CameraView, useCameraPermissions } from "expo-camera"
import * as Haptics from "expo-haptics"
import { memo, useCallback, useState } from "react"
import { KeyboardAvoidingView, Modal, Platform, StyleSheet, View } from "react-native"
import type { QrPairPayload } from "@/db/queries"
import { CameraPermissionGate } from "./CameraPermissionGate"
import { ManualServerForm } from "./ManualServerForm"
import { QRScannerOverlay } from "./QRScannerOverlay"

export type PairPayload = QrPairPayload

export interface PairServerModalProps {
  visible: boolean
  onClose: () => void
  onPairSuccess: (payload: PairPayload) => void
}

export const PairServerModal = memo(function PairServerModal({
  visible,
  onClose,
  onPairSuccess,
}: PairServerModalProps) {
  const [permission, requestPermission] = useCameraPermissions()
  const [isManualMode, setIsManualMode] = useState(false)
  const [torchOn, setTorchOn] = useState(false)
  const [isSuccess, setIsSuccess] = useState(false)
  const [hasScanned, setHasScanned] = useState(false)

  const handleBarcodeScanned = useCallback(
    (result: { data: string }) => {
      if (hasScanned || isSuccess) return
      setHasScanned(true)

      void Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success)
      setIsSuccess(true)

      let parsed: PairPayload = { url: result.data, token: "" }
      try {
        const json = JSON.parse(result.data)
        if (json.url) {
          parsed = json
        }
      } catch {}

      setTimeout(() => {
        setIsSuccess(false)
        setHasScanned(false)
        onPairSuccess(parsed)
      }, 750)
    },
    [hasScanned, isSuccess, onPairSuccess],
  )

  const handleClose = () => {
    setIsManualMode(false)
    setTorchOn(false)
    setIsSuccess(false)
    setHasScanned(false)
    onClose()
  }

  const handleManualConnect = (payload: { url: string; token: string }) => {
    void Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success)
    handleClose()
    onPairSuccess({
      url: payload.url,
      token: payload.token,
    })
  }

  return (
    <Modal
      visible={visible}
      animationType="slide"
      presentationStyle="pageSheet"
      onRequestClose={handleClose}
    >
      <View className="flex-1 bg-[#141318]">
        {isManualMode ? (
          <KeyboardAvoidingView
            behavior={Platform.OS === "ios" ? "padding" : undefined}
            className="flex-1"
          >
            <ManualServerForm
              onConnect={handleManualConnect}
              onSwitchToScanner={() => setIsManualMode(false)}
            />
          </KeyboardAvoidingView>
        ) : !permission?.granted ? (
          <CameraPermissionGate
            onRequestPermission={() => void requestPermission()}
            onManualEntry={() => setIsManualMode(true)}
          />
        ) : (
          <View className="flex-1">
            <CameraView
              style={StyleSheet.absoluteFill}
              facing="back"
              enableTorch={torchOn}
              barcodeScannerSettings={{ barcodeTypes: ["qr"] }}
              onBarcodeScanned={hasScanned ? undefined : handleBarcodeScanned}
            />

            <QRScannerOverlay
              torchOn={torchOn}
              isSuccess={isSuccess}
              onToggleTorch={() => setTorchOn((prev) => !prev)}
              onClose={handleClose}
              onManualEntry={() => setIsManualMode(true)}
            />
          </View>
        )}
      </View>
    </Modal>
  )
})
