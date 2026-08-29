import { useQuery } from "@tanstack/react-query"
import { getDatabaseStatsAsync } from "@/db/queries"

export const statsKeys = {
  all: ["databaseStats"] as const,
}

export function useDatabaseStats() {
  return useQuery({
    queryKey: statsKeys.all,
    queryFn: getDatabaseStatsAsync,
    staleTime: 0,
    refetchOnMount: "always",
  })
}
