// Başlık çubuğundaki logoya basınca açılan "Hakkında" paneli + güncelleme düğmesi.
import { useEffect, useRef, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";
import { t } from "./i18n";
import type { UpdateState } from "./updater";

const REPO = "https://github.com/bilikenes/blazon";

type Props = {
  logo: string;
  inApp: boolean;
  update: UpdateState;
  onCheck: () => void;
  onInstall: () => void;
};

/** Güncelleme durumunu tek satırlık metne çevirir. */
export function updateLabel(s: UpdateState): string | null {
  switch (s.kind) {
    case "checking":
      return t("update.checking");
    case "latest":
      return t("update.latest");
    case "available":
      return t("update.available", { version: s.version });
    case "downloading":
      return s.percent === null ? t("update.downloading") : t("update.downloadingPct", { pct: String(s.percent) });
    case "error":
      return s.during === "check" ? t("update.checkFailed") : t("update.installFailed");
    default:
      return null;
  }
}

export default function About({ logo, inApp, update, onCheck, onInstall }: Props) {
  const [open, setOpen] = useState(false);
  const [version, setVersion] = useState<string>("");
  const wrap = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (inApp) getVersion().then(setVersion).catch(() => {});
  }, [inApp]);

  useEffect(() => {
    if (!open) return;
    const onDown = (e: PointerEvent) => {
      if (!wrap.current?.contains(e.target as Node)) setOpen(false);
    };
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && setOpen(false);
    document.addEventListener("pointerdown", onDown);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("pointerdown", onDown);
      document.removeEventListener("keydown", onKey);
    };
  }, [open]);

  const busy = update.kind === "checking" || update.kind === "downloading";
  const label = updateLabel(update);

  return (
    <div className="about" ref={wrap}>
      <button className="brand" title={t("about.title")} aria-expanded={open} onClick={() => setOpen((o) => !o)}>
        <span className="brand-mark" dangerouslySetInnerHTML={{ __html: logo }} />
        Blazon
      </button>

      {/* Güncelleme varsa başlık çubuğunda görünür bir hap */}
      <AnimatePresence>
        {(update.kind === "available" || update.kind === "downloading") && (
          <motion.button
            className="update-pill"
            initial={{ opacity: 0, x: -6 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0 }}
            disabled={update.kind === "downloading"}
            onClick={onInstall}
          >
            <span className="update-dot" />
            {update.kind === "available" ? t("update.pill", { version: update.version }) : label}
          </motion.button>
        )}
      </AnimatePresence>

      <AnimatePresence>
        {open && (
          <motion.div
            className="about-pop"
            initial={{ opacity: 0, y: -6, scale: 0.98 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -4, scale: 0.98 }}
            transition={{ duration: 0.14 }}
          >
            <div className="about-head">
              <span className="about-logo" dangerouslySetInnerHTML={{ __html: logo.replace(/width="16"/, 'width="40"').replace(/height="16"/, 'height="40"') }} />
              <div>
                <strong>Blazon</strong>
                {version && <small>{t("about.version", { version })}</small>}
              </div>
            </div>
            <p className="about-text">{t("about.tagline")}</p>

            {inApp && (
              <div className="about-update">
                {update.kind === "available" ? (
                  <button className="primary small" onClick={onInstall}>
                    {t("update.installNow", { version: update.version })}
                  </button>
                ) : (
                  <button className="secondary small" disabled={busy} onClick={onCheck}>
                    {t("update.check")}
                  </button>
                )}
                {label && <span className={`about-status ${update.kind}`}>{label}</span>}
              </div>
            )}

            <div className="about-links">
              <button className="link" onClick={() => void openUrl(REPO)}>GitHub</button>
              <span className="dot">·</span>
              <button className="link" onClick={() => void openUrl(`${REPO}/issues/new`)}>{t("about.report")}</button>
              <span className="dot">·</span>
              <button className="link" onClick={() => void openUrl(`${REPO}/blob/main/LICENSE`)}>{t("about.license")}</button>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
