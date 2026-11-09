// src/icons/catalog.json'daki seçili Gravity UI ikonlarını tek bir TS dosyasına derler.
// Böylece uygulamaya 809 ikonun hepsi değil, yalnızca seçtiklerimiz girer.
// Çalıştır: npm run icons
import { readFileSync, writeFileSync, copyFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const pkg = join(root, "node_modules", "@gravity-ui", "icons");
const catalog = JSON.parse(readFileSync(join(root, "src", "icons", "catalog.json"), "utf8"));

// İngilizce etiket verilmemişse Gravity adından türet: "music-note" -> "Music note"
const humanize = (name) => {
  const words = name.replace(/-(fill|alt)$/, "").split("-");
  const text = words.join(" ");
  return text.charAt(0).toUpperCase() + text.slice(1);
};

const seen = new Set();
const missing = [];
const categories = catalog.categories.map((cat) => ({
  id: cat.id,
  label: cat.label,
  labelEn: cat.label_en ?? cat.label,
  icons: cat.icons.flatMap(([name, label, labelEn]) => {
    if (seen.has(name)) throw new Error(`"${name}" iki kez listelenmiş`);
    seen.add(name);
    let svg;
    try {
      svg = readFileSync(join(pkg, "svgs", `${name}.svg`), "utf8").trim();
    } catch {
      missing.push(name);
      return [];
    }
    const viewBox = svg.match(/viewBox="([^"]+)"/)?.[1] ?? "0 0 16 16";
    // Dış <svg> kabuğunu at; renk her zaman currentColor'dan gelsin.
    const body = svg
      .replace(/^<svg[^>]*>/, "")
      .replace(/<\/svg>$/, "")
      .replace(/(fill|stroke)="(?!none|currentColor)[^"]*"/g, '$1="currentColor"');
    return [{ id: name, label, labelEn: labelEn ?? humanize(name), viewBox, body }];
  }),
}));

if (missing.length) {
  console.error("Bulunamayan ikonlar:", missing.join(", "));
  process.exit(1);
}

const out = `// BU DOSYA OTOMATİK ÜRETİLİR — elle düzenleme. Kaynak: src/icons/catalog.json, komut: npm run icons
// İkonlar: Gravity UI Icons (MIT) — https://github.com/gravity-ui/icons
import type { SymbolCategory } from "./types";

export const CATEGORIES: SymbolCategory[] = ${JSON.stringify(categories, null, 2)};
`;
writeFileSync(join(root, "src", "icons", "catalog.generated.ts"), out);

// Lisans metni uygulamayla birlikte dağıtılır.
mkdirSync(join(root, "public", "licenses"), { recursive: true });
copyFileSync(join(pkg, "LICENSE"), join(root, "public", "licenses", "gravity-ui-icons.txt"));

const total = categories.reduce((n, c) => n + c.icons.length, 0);
console.log(`${categories.length} kategori, ${total} ikon yazıldı.`);
