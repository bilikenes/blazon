import { useEffect, useMemo, useState } from "react";
import {
  AnimatePresence,
  motion,
  useAnimate,
  useMotionValue,
  useSpring,
  useTransform,
} from "motion/react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";
import { folderPng, folderSvg, shade } from "./folderArt";
import ColorSwatches, { colorName, type Swatch } from "./ColorSwatches";
import { symbolById } from "./symbols";
import SymbolPicker from "./SymbolPicker";
import "./App.css";

// Son kutu her iki palette de "Özel renk" seçicidir.
const FOLDER_COLORS: Swatch[] = [
  { name: "Gök", hex: "#5b8def" },
  { name: "Çivit", hex: "#6e6ae8" },
  { name: "Mor", hex: "#9b6bdf" },
  { name: "Gül", hex: "#e46a93" },
  { name: "Mercan", hex: "#ee7a64" },
  { name: "Kehribar", hex: "#f0b44c" },
  { name: "Adaçayı", hex: "#7fb58a" },
  { name: "Deniz", hex: "#4fb3a9" },
  { name: "Kum", hex: "#c9b394" },
  { name: "Arduvaz", hex: "#7d8798" },
  { name: "Grafit", hex: "#4a4f5c" },
];

/** Sembol paleti; ilk kutu "Otomatik" = klasör renginin koyu tonuyla kabartma. */
const symbolColors = (folderHex: string): Swatch[] => [
  {
    name: "Otomatik",
    hex: null,
    preview: `linear-gradient(135deg, ${shade(folderHex, 0.1)} 50%, ${shade(folderHex, -0.38)} 50%)`,
  },
  { name: "Beyaz", hex: "#ffffff" },
  { name: "Gece", hex: "#23262f" },
  { name: "Gök", hex: "#5b8def" },
  { name: "Çivit", hex: "#6e6ae8" },
  { name: "Mor", hex: "#9b6bdf" },
  { name: "Gül", hex: "#e46a93" },
  { name: "Mercan", hex: "#ee7a64" },
  { name: "Kehribar", hex: "#f0b44c" },
  { name: "Adaçayı", hex: "#7fb58a" },
  { name: "Deniz", hex: "#4fb3a9" },
];

const spring = { type: "spring", stiffness: 500, damping: 32 } as const;
const soft = { type: "spring", stiffness: 260, damping: 26 } as const;
const IN_APP = isTauri();

type Toast = { id: number; kind: "ok" | "err"; text: string };

const baseName = (p: string) => p.split(/[\\/]/).filter(Boolean).pop() ?? p;

export default function App() {
  const [folder, setFolder] = useState<string | null>(null);
  const [color, setColor] = useState<string>(FOLDER_COLORS[0].hex!);
  const [symbol, setSymbol] = useState<string | null>("star");
  const [symbolColor, setSymbolColor] = useState<string | null>(null);
  const [colorTarget, setColorTarget] = useState<"folder" | "symbol">("folder");
  const symbolPalette = useMemo(() => symbolColors(color), [color]);
  const [dragging, setDragging] = useState(false);
  const [busy, setBusy] = useState(false);
  const [toast, setToast] = useState<Toast | null>(null);

  const style = { color, symbol, symbolColor };
  const svg = useMemo(() => folderSvg(style), [color, symbol, symbolColor]);
  const miniSvg = useMemo(() => folderSvg(style, 40), [color, symbol, symbolColor]);

  // Özel renk sürüklenirken sürekli tetiklenmesin diye yalnızca hazır seçimlerde animasyon
  const isPresetKey = (palette: Swatch[], v: string | null) => (palette.some((s) => s.hex === v) ? v : "custom");
  const settleKey = `${isPresetKey(FOLDER_COLORS, color)}|${symbol}|${isPresetKey(symbolPalette, symbolColor)}`;

  // Önizleme: değişince küçük bir "oturma" animasyonu
  const [previewRef, animate] = useAnimate();
  useEffect(() => {
    if (previewRef.current) animate(previewRef.current, { scale: [0.96, 1] }, soft);
  }, [settleKey]);

  // Fareyle hafif 3B eğim
  const mx = useMotionValue(0);
  const my = useMotionValue(0);
  const rotateY = useSpring(useTransform(mx, [-1, 1], [-10, 10]), soft);
  const rotateX = useSpring(useTransform(my, [-1, 1], [8, -8]), soft);

  const notify = (kind: Toast["kind"], text: string) => {
    const id = Date.now();
    setToast({ id, kind, text });
    setTimeout(() => setToast((t) => (t?.id === id ? null : t)), 2800);
  };

  useEffect(() => {
    if (!IN_APP) {
      document.documentElement.classList.add("no-mica");
      return;
    }
    const unlisten = getCurrentWebview().onDragDropEvent((e) => {
      const p = e.payload;
      if (p.type === "enter" || p.type === "over") setDragging(true);
      else if (p.type === "leave") setDragging(false);
      else if (p.type === "drop") {
        setDragging(false);
        if (p.paths[0]) setFolder(p.paths[0]);
      }
    });
    return () => void unlisten.then((f) => f());
  }, []);

  const pickFolder = async () => {
    if (!IN_APP) return notify("err", "Klasör seçimi yalnızca uygulamada çalışır");
    const picked = await open({ directory: true, multiple: false });
    if (typeof picked === "string") setFolder(picked);
  };

  const run = async (action: "apply" | "reset") => {
    if (!folder) return;
    setBusy(true);
    try {
      if (action === "apply") {
        const png = await folderPng(style);
        await invoke("apply_folder_icon", { folder, png: Array.from(png) });
        notify("ok", `${baseName(folder)} güncellendi`);
      } else {
        await invoke("reset_folder_icon", { folder });
        notify("ok", `${baseName(folder)} varsayılana döndü`);
      }
    } catch (err) {
      notify("err", String(err));
    } finally {
      setBusy(false);
    }
  };

  const win = IN_APP ? getCurrentWindow() : null;
  const symbolLabel = symbolById(symbol)?.label ?? "Yok";

  return (
    <div className="window">
      <header className="titlebar" data-tauri-drag-region>
        <div className="brand" data-tauri-drag-region>
          <span className="brand-mark" dangerouslySetInnerHTML={{ __html: folderSvg({ color: "#8a93a6", symbol: null }, 16) }} />
          Folder Studio
        </div>
        <div className="controls">
          <button aria-label="Küçült" onClick={() => win?.minimize()}>{""}</button>
          <button aria-label="Büyüt" onClick={() => win?.toggleMaximize()}>{""}</button>
          <button aria-label="Kapat" className="close" onClick={() => win?.close()}>{""}</button>
        </div>
      </header>

      <main className="body">
        <section
          className={`stage ${dragging ? "is-dragging" : ""}`}
          onMouseMove={(e) => {
            const r = e.currentTarget.getBoundingClientRect();
            mx.set(((e.clientX - r.left) / r.width) * 2 - 1);
            my.set(((e.clientY - r.top) / r.height) * 2 - 1);
          }}
          onMouseLeave={() => {
            mx.set(0);
            my.set(0);
          }}
        >
          <div className="canvas">
            <motion.div
              className="preview"
              style={{ rotateX, rotateY }}
              animate={{ scale: dragging ? 1.06 : 1, y: dragging ? -6 : 0 }}
              transition={soft}
            >
              <div ref={previewRef} dangerouslySetInnerHTML={{ __html: svg }} />
            </motion.div>
            <motion.div
              className="floor"
              animate={{ scaleX: dragging ? 0.85 : 1, opacity: dragging ? 0.5 : 1 }}
              transition={soft}
            />
          </div>

          <AnimatePresence mode="wait" initial={false}>
            {folder ? (
              <motion.div
                key="chosen"
                className="target"
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -8 }}
                transition={soft}
              >
                <span className="target-icon" dangerouslySetInnerHTML={{ __html: miniSvg }} />
                <span className="target-text">
                  <strong>{baseName(folder)}</strong>
                  <small title={folder}>{folder}</small>
                </span>
                <button className="link" onClick={pickFolder}>Değiştir</button>
              </motion.div>
            ) : (
              <motion.div
                key="empty"
                className="hint"
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -8 }}
                transition={soft}
              >
                {dragging ? "Bırak" : "Bir klasörü pencereye sürükle"}
                <span className="dot">·</span>
                <button className="link" onClick={pickFolder}>Klasör seç</button>
              </motion.div>
            )}
          </AnimatePresence>
        </section>

        <aside className="inspector">
          <div className="group">
            <div className="label">
              <span>
                Renk{" "}
                <em>
                  {colorTarget === "folder"
                    ? colorName(FOLDER_COLORS, color)
                    : colorName(symbolPalette, symbolColor)}
                </em>
              </span>
              <div className="segmented" role="tablist">
                {(["folder", "symbol"] as const).map((t) => (
                  <button
                    key={t}
                    role="tab"
                    aria-selected={colorTarget === t}
                    className={colorTarget === t ? "on" : ""}
                    onClick={() => setColorTarget(t)}
                  >
                    {colorTarget === t && <motion.span layoutId="seg-bg" className="seg-bg" transition={spring} />}
                    <span>{t === "folder" ? "Klasör" : "Sembol"}</span>
                  </button>
                ))}
              </div>
            </div>
            {colorTarget === "folder" ? (
              <ColorSwatches
                key="folder"
                group="folder"
                swatches={FOLDER_COLORS}
                value={color}
                onChange={(hex) => hex && setColor(hex)}
                defaultCustom="#3fbf8f"
              />
            ) : (
              <ColorSwatches
                key="symbol"
                group="symbol"
                swatches={symbolPalette}
                value={symbolColor}
                onChange={setSymbolColor}
                defaultCustom="#ffd166"
              />
            )}
          </div>

          <div className="group grow">
            <div className="label">
              Sembol <em>{symbolLabel}</em>
            </div>

            <SymbolPicker value={symbol} onChange={setSymbol} />
          </div>

          <div className="footer">
            <motion.button
              className="primary"
              disabled={!folder || busy}
              onClick={() => run("apply")}
              whileTap={{ scale: 0.98 }}
              transition={spring}
            >
              {busy ? <span className="spinner" /> : "Uygula"}
            </motion.button>
            <button className="secondary" disabled={!folder || busy} onClick={() => run("reset")}>
              Varsayılana döndür
            </button>
          </div>
        </aside>
      </main>

      <AnimatePresence>
        {toast && (
          <motion.div
            key={toast.id}
            className={`toast ${toast.kind}`}
            initial={{ opacity: 0, y: 16, scale: 0.96 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: 8, scale: 0.98 }}
            transition={spring}
          >
            <span className="toast-dot" />
            {toast.text}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
