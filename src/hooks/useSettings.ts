import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Settings } from "../types";

export function useSettings() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [saving, setSaving] = useState(false);

  const reload = useCallback(async () => {
    const s = await invoke<Settings>("get_settings");
    setSettings(s);
  }, []);

  useEffect(() => {
    reload();
  }, [reload]);

  const save = useCallback(async (next: Settings) => {
    setSaving(true);
    try {
      await invoke("set_settings", { next });
      setSettings(next);
    } finally {
      setSaving(false);
    }
  }, []);

  return { settings, saving, save, reload };
}
