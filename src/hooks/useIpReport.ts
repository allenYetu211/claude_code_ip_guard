import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { IpReport } from "../types";

export function useIpReport() {
  const [report, setReport] = useState<IpReport | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetch = useCallback(async (force: boolean) => {
    setRefreshing(true);
    setError(null);
    try {
      const r = await invoke<IpReport>("get_ip_report", { forceRefresh: force });
      setReport(r);
    } catch (e) {
      setError(String(e));
    } finally {
      setRefreshing(false);
    }
  }, []);

  // 用户主动点刷新 → 强制重新检测（绕过缓存）
  const refresh = useCallback(() => fetch(true), [fetch]);

  // 启动时一次软加载（命中缓存就直接用）
  useEffect(() => {
    fetch(false);
  }, [fetch]);

  return { report, refreshing, error, refresh, setReport };
}
