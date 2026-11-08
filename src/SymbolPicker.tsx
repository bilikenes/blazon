// Sembol seçici: üstte kategori ikon şeridi, altta tüm semboller tek dikey listede.
// Kaydırdıkça şeritte aktif kategori değişir; şeritteki kategoriye basınca listede oraya kayar.
// Şeridin başındaki büyüteç şeridi arama kutusuna çevirir.
import { useEffect, useRef, useState } from "react";
import { motion } from "motion/react";
import { CATEGORIES, searchSymbols, symbolById, symbolSvg, type SymbolDef } from "./symbols";
import xmarkRaw from "@gravity-ui/icons/svgs/xmark.svg?raw";
import magnifierRaw from "@gravity-ui/icons/svgs/magnifier.svg?raw";

const spring = { type: "spring", stiffness: 500, damping: 34 } as const;

/** Şeritte her kategoriyi temsil eden ikon (sembol setinden). */
const CATEGORY_ICON: Record<string, string> = {
  general: "star", work: "briefcase", dev: "code", design: "palette", media: "music-note",
  finance: "wallet", learn: "book-open", life: "house", system: "gear",
};

const sized = (raw: string, size: number) =>
  raw.replace(/width="\d+"/, `width="${size}"`).replace(/height="\d+"/, `height="${size}"`);
const categoryIcon = (id: string) => {
  const s = symbolById(CATEGORY_ICON[id]);
  return s ? symbolSvg(s, 15) : "";
};
const Svg = ({ html, className = "svg" }: { html: string; className?: string }) => (
  <span className={className} dangerouslySetInnerHTML={{ __html: html }} />
);

type Props = { value: string | null; onChange: (id: string | null) => void };

export default function SymbolPicker({ value, onChange }: Props) {
  const [active, setActive] = useState(CATEGORIES[0].id);
  const [searchOpen, setSearchOpen] = useState(false);
  const [query, setQuery] = useState("");
  const scroller = useRef<HTMLDivElement>(null);
  const input = useRef<HTMLInputElement>(null);
  const results = query.trim() ? searchSymbols(query) : null;

  useEffect(() => {
    if (searchOpen) input.current?.focus();
  }, [searchOpen]);

  const closeSearch = () => {
    setSearchOpen(false);
    setQuery("");
  };

  // Görünen bölüme göre aktif kategori
  const onScroll = () => {
    const el = scroller.current;
    if (!el || results) return;
    const line = el.getBoundingClientRect().top + 24;
    let current = CATEGORIES[0].id;
    for (const sec of el.querySelectorAll<HTMLElement>("[data-cat]")) {
      if (sec.getBoundingClientRect().top <= line) current = sec.dataset.cat!;
    }
    setActive(current);
  };

  const jumpTo = (id: string) => {
    const el = scroller.current;
    const sec = el?.querySelector<HTMLElement>(`[data-cat="${id}"]`);
    if (!el || !sec) return;
    setActive(id);
    el.scrollTo({
      top: el.scrollTop + sec.getBoundingClientRect().top - el.getBoundingClientRect().top,
      behavior: "smooth",
    });
  };

  const grid = (items: SymbolDef[], withNone = false) => (
    <div className="symbols">
      {withNone && (
        <button className={`symbol ${value === null ? "on" : ""}`} title="Sembol yok" onClick={() => onChange(null)}>
          {value === null && <motion.span layoutId="symbol-bg" className="symbol-bg" transition={spring} />}
          <span className="symbol-none" />
        </button>
      )}
      {items.map((s) => (
        <button
          key={s.id}
          className={`symbol ${value === s.id ? "on" : ""}`}
          title={s.label}
          aria-label={s.label}
          onClick={() => onChange(s.id)}
        >
          {value === s.id && <motion.span layoutId="symbol-bg" className="symbol-bg" transition={spring} />}
          <Svg className="symbol-glyph" html={symbolSvg(s, 18)} />
        </button>
      ))}
    </div>
  );

  return (
    <>
      <div className="strip">
        {searchOpen ? (
          <motion.label
            className="strip-search"
            initial={{ opacity: 0, x: -8 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.16 }}
          >
            <Svg html={sized(magnifierRaw, 14)} />
            <input
              ref={input}
              value={query}
              placeholder="Sembol ara"
              spellCheck={false}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={(e) => e.key === "Escape" && closeSearch()}
            />
            <button className="icon-btn" title="Aramayı kapat" onClick={closeSearch}>
              <Svg html={sized(xmarkRaw, 12)} />
            </button>
          </motion.label>
        ) : (
          <>
            <button className="strip-btn" title="Ara" aria-label="Sembol ara" onClick={() => setSearchOpen(true)}>
              <Svg html={sized(magnifierRaw, 15)} />
            </button>
            <span className="strip-sep" />
            {CATEGORIES.map((c) => (
              <button
                key={c.id}
                className={`strip-btn ${active === c.id ? "on" : ""}`}
                title={c.label}
                aria-label={c.label}
                onClick={() => jumpTo(c.id)}
              >
                <Svg html={categoryIcon(c.id)} />
                {active === c.id && <motion.span layoutId="strip-line" className="strip-line" transition={spring} />}
              </button>
            ))}
          </>
        )}
      </div>

      <div className="symbols-scroll" ref={scroller} onScroll={onScroll}>
        {results ? (
          results.length ? (
            grid(results)
          ) : (
            <p className="empty">“{query}” için sonuç yok</p>
          )
        ) : (
          CATEGORIES.map((c, i) => (
            <section key={c.id} data-cat={c.id}>
              <h4 className="section-title">{c.label}</h4>
              {grid(c.icons, i === 0)}
            </section>
          ))
        )}
      </div>
    </>
  );
}
