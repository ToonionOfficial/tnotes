import type { ConfigContext, ExpoConfig } from "expo/config"

export default ({ config }: ConfigContext): ExpoConfig => {
  const variant = process.env.APP_VARIANT || "development"
  const isDev = variant === "development"
  const isPreview = variant === "preview"
  const isProd = variant === "production"

  const getAppName = () => {
    if (isProd) return "TNotes"
    if (isPreview) return "TNotes (Preview)"
    return "TNotes (Dev)"
  }

  const getBundleIdentifier = () => {
    if (isProd) return "net.toonion.tnotes"
    if (isPreview) return "net.toonion.tnotes.preview"
    return "net.toonion.tnotes.dev"
  }

  const getPackageName = () => {
    if (isProd) return "net.toonion.tnotes"
    if (isPreview) return "net.toonion.tnotes.preview"
    return "net.toonion.tnotes.dev"
  }

  const getScheme = () => {
    if (isProd) return "tnotes"
    if (isPreview) return "tnotes-preview"
    return "tnotes-dev"
  }

  const getAppIcon = () => {
    if (isDev) return "./assets/images/icon-dev.png"
    return "./assets/images/icon.png"
  }

  const getAndroidForegroundIcon = () => {
    if (isDev) return "./assets/images/android-icon-foreground-dev.png"
    return "./assets/images/android-icon-foreground.png"
  }

  const getSplashIcon = () => {
    if (isDev) return "./assets/images/splash-icon-dev.png"
    return "./assets/images/splash-icon.png"
  }

  return {
    ...config,
    name: getAppName(),
    slug: "tnotes",
    version: config.version || "0.1.0",
    orientation: "portrait",
    icon: getAppIcon(),
    scheme: getScheme(),
    userInterfaceStyle: "automatic",
    ios: {
      ...config.ios,
      icon: getAppIcon(),
      bundleIdentifier: getBundleIdentifier(),
      infoPlist: {
        ITSAppUsesNonExemptEncryption: false,
        ...config.ios?.infoPlist,
      },
    },
    android: {
      ...config.android,
      package: getPackageName(),
      adaptiveIcon: {
        backgroundColor: "#141318",
        foregroundImage: getAndroidForegroundIcon(),
        backgroundImage: "./assets/images/android-icon-background.png",
        monochromeImage: "./assets/images/android-icon-monochrome.png",
      },
      predictiveBackGestureEnabled: false,
    },
    web: {
      output: "static",
      favicon: "./assets/images/favicon.png",
    },
    plugins: [
      "expo-router",
      [
        "expo-splash-screen",
        {
          backgroundColor: "#141318",
          image: getSplashIcon(),
          imageWidth: 100,
        },
      ],
      "expo-sqlite",
      "expo-font",
      "expo-secure-store",
      "expo-sharing",
      [
        "expo-camera",
        {
          cameraPermission:
            "Allow $(PRODUCT_NAME) to access your camera to scan QR codes for device pairing.",
        },
      ],
    ],
    experiments: {
      typedRoutes: true,
      reactCompiler: true,
    },
    extra: {
      ...config.extra,
      router: {},
      eas: {
        projectId: "9dee4e6d-322a-4508-9f21-c8726492a003",
      },
    },
  }
}
