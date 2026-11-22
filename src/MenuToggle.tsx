// Hakkında panelindeki "ana sağ tık menüsünde göster" anahtarı (yalnızca Windows 11'de görünür).
import { useEffect, useState } from "react";
import { motion } from "motion/react";
import { invoke } from "@tauri-apps/api/core";
import { t } from "./i18n";

type Status = "unavailable" | "off" | "on";
type MenuError = { code: "cancelled" | "unavailable" | "trust" | "package"; hresult?: number };

function message(err: unknown, turningOn: boolean): string {
  const e = err as Partial<MenuError> | null;
  if (e?.code === "cancelled") return t(turningOn ? "menu.cancelled" : "menu.certKept");
  if (e?.code === "trust") return t(turningOn ? "menu.trustFailed" : "menu.certKept");
  if (e?.code === "package" && e.hresult !== undefined) {
    return t("menu.failed", { code: `0x${(e.hresult >>> 0).toString(16).toUpperCase().padStart(8, "0")}` });
  }
  return t("error.unknown");
}

export default function MenuToggle() {
  const [status, setStatus] = useState<Status>("unavailable");
  const [busy, setBusy] = useState(false);
  const [note, setNote] = useState<string | null>(null);

  useEffect(() => {
    invoke<Status>("menu_status").then(setStatus).catch(() => {});
  }, []);

  if (status === "unavailable") return null;
  const on = status === "on";

  const toggle = async () => {
    setBusy(true);
    setNote(null);
    try {
      setStatus(await invoke<Status>("set_menu", { enabled: !on }));
    } catch (err) {
      setNote(message(err, !on));
      // Kapatırken yalnızca sertifika adımı reddedilmiş olabilir; gerçek durumu oku.
      setStatus(await invoke<Status>("menu_status").catch(() => status));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="about-menu">
      <button className="switch-row" role="switch" aria-checked={on} disabled={busy} onClick={() => void toggle()}>
        <span>{t("menu.label")}</span>
        <span className={on ? "switch on" : "switch"}>
          <motion.span
            className="switch-knob"
            initial={false}
            animate={{ x: on ? 14 : 0 }}
            transition={{ type: "spring", stiffness: 500, damping: 36 }}
          />
        </span>
      </button>
      <span className="about-status" role="status">
        {note ?? t(on ? "menu.hintOn" : "menu.hintOff")}
      </span>
    </div>
  );
}
