import { useCallback, useEffect, useRef, useState } from "react";
import { getSettings, patchSettings, saveSettings } from "./api";
import type { AppSettings } from "./types";

export interface UseSettingsResult {
  settings: AppSettings | null;
  loading: boolean;
  error: string | null;
  patch: (partial: Partial<AppSettings>, immediate?: boolean) => void;
  save: (next: AppSettings) => Promise<void>;
  reload: () => Promise<void>;
}

export function useSettings(): UseSettingsResult {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const pendingRef = useRef<Partial<AppSettings>>({});

  const reload = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const cfg = await getSettings();
      setSettings(cfg);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    reload();
  }, [reload]);

  const flushPending = useCallback(async (base: AppSettings) => {
    const patch = pendingRef.current;
    pendingRef.current = {};
    if (Object.keys(patch).length === 0) return base;
    try {
      const next = await patchSettings(patch);
      setSettings(next);
      return next;
    } catch (e) {
      setError(String(e));
      return base;
    }
  }, []);

  const save = useCallback(async (next: AppSettings) => {
    setError(null);
    try {
      await saveSettings(next);
      setSettings(next);
    } catch (e) {
      setError(String(e));
      throw e;
    }
  }, []);

  const patch = useCallback(
    (partial: Partial<AppSettings>, immediate = false) => {
      setSettings((prev) => (prev ? { ...prev, ...partial } : prev));
      pendingRef.current = { ...pendingRef.current, ...partial };

      if (debounceRef.current) clearTimeout(debounceRef.current);

      if (immediate) {
        void flushPending({ ...(settings ?? ({} as AppSettings)), ...partial });
        return;
      }

      debounceRef.current = setTimeout(() => {
        setSettings((prev) => {
          if (prev) void flushPending(prev);
          return prev;
        });
      }, 300);
    },
    [flushPending, settings],
  );

  useEffect(
    () => () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    },
    [],
  );

  return { settings, loading, error, patch, save, reload };
}
