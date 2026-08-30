import * as SecureStore from "expo-secure-store"
import { createContext, type ReactNode, useCallback, useContext, useEffect, useState } from "react"
import { Uniwind } from "uniwind"

export type AppTheme = "dark" | "light"

const THEME_STORAGE_KEY = "tnotes_theme_preference"

export interface ThemeColors {
  background: string
  foreground: string
  card: string
  muted: string
  mutedForeground: string
  primary: string
  border: string
}

export const DARK_COLORS: ThemeColors = {
  background: "#141318",
  foreground: "#E6E1E9",
  card: "#201F24",
  muted: "#201F24",
  mutedForeground: "#C6C2CD",
  primary: "#CABEFF",
  border: "#302E36",
}

export const LIGHT_COLORS: ThemeColors = {
  background: "#F8F7FA",
  foreground: "#1D1B20",
  card: "#FFFFFF",
  muted: "#F3EEF8",
  mutedForeground: "#79747E",
  primary: "#65558F",
  border: "#E6E0E9",
}

interface AppThemeContextValue {
  theme: AppTheme
  isDarkMode: boolean
  colors: ThemeColors
  setTheme: (theme: AppTheme) => void
  toggleTheme: (isDark: boolean) => void
}

const AppThemeContext = createContext<AppThemeContextValue>({
  theme: "dark",
  isDarkMode: true,
  colors: DARK_COLORS,
  setTheme: () => {},
  toggleTheme: () => {},
})

export function AppThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setThemeState] = useState<AppTheme>("dark")

  useEffect(() => {
    async function loadTheme() {
      try {
        const saved = await SecureStore.getItemAsync(THEME_STORAGE_KEY)
        if (saved === "light" || saved === "dark") {
          setThemeState(saved)
          Uniwind.setTheme(saved)
        } else {
          Uniwind.setTheme("dark")
        }
      } catch {
        Uniwind.setTheme("dark")
      }
    }
    void loadTheme()
  }, [])

  const setTheme = useCallback((newTheme: AppTheme) => {
    setThemeState(newTheme)
    Uniwind.setTheme(newTheme)
    void SecureStore.setItemAsync(THEME_STORAGE_KEY, newTheme).catch(() => {})
  }, [])

  const toggleTheme = useCallback(
    (isDark: boolean) => {
      const nextTheme: AppTheme = isDark ? "dark" : "light"
      setTheme(nextTheme)
    },
    [setTheme],
  )

  const isDarkMode = theme === "dark"
  const colors = isDarkMode ? DARK_COLORS : LIGHT_COLORS

  return (
    <AppThemeContext.Provider
      value={{
        theme,
        isDarkMode,
        colors,
        setTheme,
        toggleTheme,
      }}
    >
      {children}
    </AppThemeContext.Provider>
  )
}

export function useAppTheme() {
  return useContext(AppThemeContext)
}
