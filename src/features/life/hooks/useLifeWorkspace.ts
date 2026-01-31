import { useEffect, useState } from "react";
import type { LifeDomain, LifeTimeRange } from "../types";

export function useLifeWorkspace(workspaceId: string | null) {
  const [activeDomain, setActiveDomain] = useState<LifeDomain | null>(null);
  const [timeRange, setTimeRange] = useState<LifeTimeRange>("today");

  useEffect(() => {
    if (!workspaceId) {
      setActiveDomain(null);
      setTimeRange("today");
      return;
    }
    setActiveDomain("stream");
  }, [workspaceId]);

  return {
    activeDomain,
    setActiveDomain,
    timeRange,
    setTimeRange,
  };
}
