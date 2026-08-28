import * as Haptics from "expo-haptics"
import type { ReactNode } from "react"
import { memo, useCallback, useEffect, useRef } from "react"
import { Animated, Pressable, Text, View } from "react-native"
import { Swipeable } from "react-native-gesture-handler"

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
  progress: Animated.AnimatedInterpolation<number>
  translation: Animated.AnimatedInterpolation<number>
  onDrag: (value: number) => void
  onPress: () => void
}

function SwipeActionView({
  action,
  side,
  progress,
  translation,
  onDrag,
  onPress,
}: SwipeActionViewProps) {
  const onDragRef = useRef(onDrag)
  onDragRef.current = onDrag

  useEffect(() => {
    const listenerId = translation.addListener(({ value }) => {
      onDragRef.current(value)
    })
    return () => translation.removeListener(listenerId)
  }, [translation])

  return (
    <Animated.View
      className="w-[76px] items-center justify-center"
      style={{
        backgroundColor: action.color,
        opacity: progress.interpolate({ inputRange: [0, 1], outputRange: [0, 1] }),
        transform: [
          {
            translateX: progress.interpolate({
              inputRange: [0, 1],
              outputRange: [side === "left" ? -20 : 20, 0],
            }),
          },
        ],
      }}
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
  const swipeableRef = useRef<Swipeable>(null)
  const maxSwipeDistance = useRef(0)
  const hasTriggeredThreshold = useRef(false)

  const handleDrag = useCallback((value: number) => {
    const abs = Math.abs(value)
    maxSwipeDistance.current = Math.max(maxSwipeDistance.current, abs)

    if (abs >= SWIPE_TO_CONFIRM_DISTANCE && !hasTriggeredThreshold.current) {
      hasTriggeredThreshold.current = true
      void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Heavy)
    } else if (abs < SWIPE_TO_CONFIRM_DISTANCE && hasTriggeredThreshold.current) {
      hasTriggeredThreshold.current = false
    }
  }, [])

  const renderAction = useCallback(
    (
      action: SwipeAction | undefined,
      side: "left" | "right",
      progress: Animated.AnimatedInterpolation<number>,
      translation: Animated.AnimatedInterpolation<number>,
    ) => {
      if (!action) return null

      return (
        <SwipeActionView
          action={action}
          side={side}
          progress={progress}
          translation={translation}
          onDrag={handleDrag}
          onPress={() => {
            void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Heavy)
            swipeableRef.current?.close()
            maxSwipeDistance.current = 0
            hasTriggeredThreshold.current = false
            action.onPress()
          }}
        />
      )
    },
    [handleDrag],
  )

  const handleWillOpen = useCallback(
    (direction: "left" | "right") => {
      if (maxSwipeDistance.current >= SWIPE_TO_CONFIRM_DISTANCE) {
        const action = direction === "left" ? leftAction : rightAction
        swipeableRef.current?.close()
        maxSwipeDistance.current = 0
        hasTriggeredThreshold.current = false
        action?.onPress()
      }
    },
    [leftAction, rightAction],
  )

  const handleClose = useCallback(() => {
    maxSwipeDistance.current = 0
    hasTriggeredThreshold.current = false
  }, [])

  return (
    <Swipeable
      ref={swipeableRef}
      renderLeftActions={(progress, translation) =>
        renderAction(leftAction, "left", progress, translation)
      }
      renderRightActions={(progress, translation) =>
        renderAction(rightAction, "right", progress, translation)
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
      overshootLeft
      overshootRight
      overshootFriction={1}
      useNativeAnimations={false}
    >
      {children}
    </Swipeable>
  )
})
