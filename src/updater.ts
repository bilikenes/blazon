// Otomatik güncelleme: açılıştan kısa süre sonra GitHub Releases'teki latest.json'a bakar.
// İmza (tauri.conf.json > plugins.updater.pubkey) doğrulanmadan hiçbir güncelleme kurulmaz.
import { useCallback, useEffect, useRef, useState } from "react";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export type UpdateState =
  | { kind: "idle" }
  | { kind: "checking" }
  | { kind: "latest" }
  | { kind: "available"; version: string }
  | { kind: "downloading"; percent: number | null }
  | { kind: "error"; during: "check" | "install" };

const AUTO_CHECK_DELAY_MS = 4000;

export function useUpdater(enabled: boolean) {
  const [state, setState] = useState<UpdateState>({ kind: "idle" });
  const pending = useRef<Update | null>(null);

  /** silent: otomatik denetimde ağ hatası vb. kullanıcıya gösterilmez. */
  const runCheck = useCallback(
    async (silent: boolean) => {
      if (!enabled) return;
      if (!silent) setState({ kind: "checking" });
      try {
        const update = await check();
        pending.current = update;
        if (update) setState({ kind: "available", version: update.version });
        else if (!silent) setState({ kind: "latest" });
      } catch (err) {
        console.warn("Güncelleme denetimi başarısız:", err);
        setState(silent ? { kind: "idle" } : { kind: "error", during: "check" });
      }
    },
    [enabled],
  );

  const install = useCallback(async () => {
    const update = pending.current;
    if (!update) return;
    let total = 0;
    let done = 0;
    setState({ kind: "downloading", percent: null });
    try {
      await update.downloadAndInstall((e) => {
        if (e.event === "Started") total = e.data.contentLength ?? 0;
        else if (e.event === "Progress") {
          done += e.data.chunkLength;
          setState({ kind: "downloading", percent: total ? Math.min(100, Math.round((done / total) * 100)) : null });
        }
      });
      // Windows'ta kurulum programı uygulamayı kendisi kapatır; diğer durumlarda yeniden başlat.
      await relaunch();
    } catch (err) {
      console.error("Güncelleme kurulamadı:", err);
      setState({ kind: "error", during: "install" });
    }
  }, []);

  useEffect(() => {
    if (!enabled || import.meta.env.DEV) return;
    const id = setTimeout(() => void runCheck(true), AUTO_CHECK_DELAY_MS);
    return () => clearTimeout(id);
  }, [enabled, runCheck]);

  return { state, checkNow: () => runCheck(false), install };
}
