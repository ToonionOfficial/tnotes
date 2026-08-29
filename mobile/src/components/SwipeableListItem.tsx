import * as Haptics from "expo-haptics"
import type { ReactNode } from "react"
import { memo, useCallback, useRef } from "react"
import { Pressable, Text, View } from "react-native"
import ReanimatedSwipeable, {
  type SwipeableMethods,
} from "react-native-gesture-handler/ReanimatedSwipeable"
import Animated, {
  interpolate,
  runOnJS,
  type SharedValue,
  useAnimatedReaction,
  useAnimatedStyle,
} from "react-native-reanimated"

export interface SwipeAction {
  label: string
  color: string
  icon: ReactNode
  onPress: () => void
}

interface SwipeableListItemProps {
  children: ReactNode
  leftAction?: SwipeAction
  rightAction?: SwipeAction
  rounded?: "only" | "first" | "middle" | "last"
}

const SWIPE_TO_CONFIRM_DISTANCE = 150

interface SwipeActionViewProps {
  action: SwipeAction
  side: "left" | "right"
  progress: SharedValue<number>
  translation: SharedValue<number>
  onDragUpdate: (value: number) => void
  onPress: () => void
}

function SwipeActionView({
  action,
  side,
  progress,
  translation,
  onDragUpdate,
  onPress,
}: SwipeActionViewProps) {
  const triggerHaptic = useCallback(() => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Heavy)
  }, [])

  useAnimatedReaction(
    () => Math.abs(translation.value),
    (current, previous) => {
      if (
        current >= SWIPE_TO_CONFIRM_DISTANCE &&
        (previous === null || previous < SWIPE_TO_CONFIRM_DISTANCE)
      ) {
        runOnJS(triggerHaptic)()
      }
      if (current > 0) {
        runOnJS(onDragUpdate)(current)
      }
    },
    [triggerHaptic, onDragUpdate],
  )

  const animatedStyle = useAnimatedStyle(() => {
    const opacity = interpolate(progress.value, [0, 1], [0, 1])
    const translateX = interpolate(progress.value, [0, 1], [side === "left" ? -20 : 20, 0])
    return {
      opacity,
      transform: [{ translateX }],
    }
  })

  return (
    <Animated.View
      className="w-[76px] items-center justify-center"
      style={[
        {
          backgroundColor: action.color,
        },
        animatedStyle,
      ]}
    >
      <View
        style={{
          position: "absolute",
          top: 0,
          bottom: 0,
          backgroundColor: action.color,
          width: 500,
          [side === "left" ? "right" : "left"]: 0,
        }}
      />
      <Pressable
        accessibilityLabel={action.label}
        onPress={onPress}
        className="flex-1 items-center justify-center px-1"
      >
        {action.icon}
        <Text className="mt-1 text-xs font-medium text-white">{action.label}</Text>
      </Pressable>
    </Animated.View>
  )
}

export const SwipeableListItem = memo(function SwipeableListItem({
  children,
  leftAction,
  rightAction,
  rounded = "middle",
}: SwipeableListItemProps) {
  const swipeableRef = useRef<SwipeableMethods>(null)
  const maxSwipeDistance = useRef(0)

  const handleDragUpdate = useCallback((value: number) => {
    maxSwipeDistance.current = Math.max(maxSwipeDistance.current, value)
  }, [])

  const renderAction = useCallback(
    (
      action: SwipeAction | undefined,
      side: "left" | "right",
      progress: SharedValue<number>,
      translation: SharedValue<number>,
      swipeableMethods: SwipeableMethods,
    ) => {
      if (!action) return null

      return (
        <SwipeActionView
          action={action}
          side={side}
          progress={progress}
          translation={translation}
          onDragUpdate={handleDragUpdate}
          onPress={() => {
            void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Heavy)
            swipeableMethods.close()
            maxSwipeDistance.current = 0
            action.onPress()
          }}
        />
      )
    },
    [handleDragUpdate],
  )

  const handleWillOpen = useCallback(
    (direction: "left" | "right") => {
      if (maxSwipeDistance.current >= SWIPE_TO_CONFIRM_DISTANCE) {
        const action = direction === "right" ? leftAction : rightAction
        swipeableRef.current?.close()
        maxSwipeDistance.current = 0
        action?.onPress()
      }
    },
    [leftAction, rightAction],
  )

  const handleClose = useCallback(() => {
    maxSwipeDistance.current = 0
  }, [])

  return (
    <ReanimatedSwipeable
      ref={swipeableRef}
      renderLeftActions={
        leftAction
          ? (progress, translation, methods) =>
              renderAction(leftAction, "left", progress, translation, methods)
          : undefined
      }
      renderRightActions={
        rightAction
          ? (progress, translation, methods) =>
              renderAction(rightAction, "right", progress, translation, methods)
          : undefined
      }
      onSwipeableWillOpen={handleWillOpen}
      onSwipeableClose={handleClose}
      containerStyle={{
        overflow: "hidden",
        borderTopLeftRadius: rounded === "only" || rounded === "first" ? 24 : 0,
        borderTopRightRadius: rounded === "only" || rounded === "first" ? 24 : 0,
        borderBottomLeftRadius: rounded === "only" || rounded === "last" ? 24 : 0,
        borderBottomRightRadius: rounded === "only" || rounded === "last" ? 24 : 0,
      }}
      friction={1}
      overshootLeft={Boolean(leftAction)}
      overshootRight={Boolean(rightAction)}
      overshootFriction={1}
    >
      {children}
    </ReanimatedSwipeable>
  )
})
