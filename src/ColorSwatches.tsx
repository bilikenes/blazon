// Hazır renk kutuları + son kutuda özel renk seçici (açılır panel).
import { useEffect, useRef, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { HexColorInput, HexColorPicker } from "react-colorful";
import { t } from "./i18n";

export type Swatch = {
  id: string; // sabit anahtar (dil değişse de aynı)
  name: string; // görünen ad
  hex: string | null; // null = "Otomatik"
  preview?: string; // kutuda gösterilecek CSS arka planı (varsayılan: hex)
};

type Props = {
  swatches: Swatch[];
  value: string | null;
  onChange: (hex: string | null) => void;
  group: string; // seçim halkası animasyonu için benzersiz ad
  defaultCustom: string;
};

const spring = { type: "spring", stiffness: 500, damping: 32 } as const;

export const colorName = (swatches: Swatch[], value: string | null) =>
  swatches.find((s) => s.hex === value)?.name ?? t("color.customName", { hex: value?.toUpperCase() ?? "" });

export default function ColorSwatches({ swatches, value, onChange, group, defaultCustom }: Props) {
  const isPreset = swatches.some((s) => s.hex === value);
  const [custom, setCustom] = useState(isPreset || !value ? defaultCustom : value);
  const [open, setOpen] = useState(false);
  const wrap = useRef<HTMLDivElement>(null);

  // Dışarı tıklayınca veya Esc ile kapanır
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

  const pickCustom = (hex: string) => {
    setCustom(hex);
    onChange(hex);
  };

  return (
    <div className="swatches-wrap" ref={wrap}>
      <div className="swatches">
        {swatches.map((s) => (
          <button
            key={s.id}
            className="swatch"
            title={s.name}
            aria-label={s.name}
            onClick={() => {
              onChange(s.hex);
              setOpen(false);
            }}
          >
            {value === s.hex && <motion.span layoutId={`${group}-ring`} className="swatch-ring" transition={spring} />}
            <span className="swatch-fill" style={{ background: s.preview ?? s.hex ?? undefined }} />
          </button>
        ))}

        <button
          className={`swatch custom ${!isPreset ? "on" : ""}`}
          title={t("color.custom")}
          aria-label={t("color.custom")}
          aria-expanded={open}
          onClick={() => {
            if (isPreset) onChange(custom);
            setOpen((o) => !o || isPreset);
          }}
        >
          {!isPreset && <motion.span layoutId={`${group}-ring`} className="swatch-ring" transition={spring} />}
          <span className="swatch-fill custom-fill" style={!isPreset ? { background: value ?? undefined } : undefined}>
            <span className="custom-plus">{!isPreset ? "" : "+"}</span>
          </span>
        </button>
      </div>

      <AnimatePresence>
        {open && (
          <motion.div
            className="color-pop"
            initial={{ opacity: 0, y: -6, scale: 0.97 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -4, scale: 0.98 }}
            transition={{ duration: 0.14 }}
          >
            <HexColorPicker color={custom} onChange={pickCustom} />
            <div className="color-pop-row">
              <span className="color-pop-dot" style={{ background: custom }} />
              <span className="color-pop-hash">#</span>
              <HexColorInput className="color-pop-input" color={custom} onChange={pickCustom} />
              <button className="color-pop-done" onClick={() => setOpen(false)}>
                {t("color.done")}
              </button>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
