// Klasör ikonunun tek kaynağı: hem önizleme hem de klasöre uygulanan PNG buradan çıkar.
import { symbolById } from "./symbols";

export type FolderStyle = {
  color: string; // #rrggbb
  symbol: string | null; // symbols.ts içindeki id
  symbolColor?: string | null; // null/undefined = otomatik (klasör renginde kabartma)
};

const clamp = (n: number) => Math.max(0, Math.min(255, Math.round(n)));

/** amount > 0 beyaza, < 0 siyaha doğru karıştırır. */
export function shade(hex: string, amount: number): string {
  const n = parseInt(hex.slice(1), 16);
  const mix = (c: number) => (amount >= 0 ? c + (255 - c) * amount : c * (1 + amount));
  const r = clamp(mix((n >> 16) & 255));
  const g = clamp(mix((n >> 8) & 255));
  const b = clamp(mix(n & 255));
  return `#${((r << 16) | (g << 8) | b).toString(16).padStart(6, "0")}`;
}

let uid = 0;

export function folderSvg({ color, symbol, symbolColor }: FolderStyle, size = 256): string {
  // Aynı sayfadaki SVG'lerin degrade/filtre id'leri çakışmasın diye benzersiz önek
  const k = `fs${++uid}`;
  const back = shade(color, -0.2);
  const backDeep = shade(color, -0.32);
  const top = shade(color, 0.16);
  const bottom = shade(color, -0.07);
  const engrave = shade(color, -0.38);
  const def = symbolById(symbol);

  // Sembol ön panele "kazınmış" gibi durur: altta ince açık ton, üstte koyu ton.
  // İkon iç içe <svg> olarak yerleşir; viewBox'u ne olursa olsun 84px kareye ölçeklenir.
  const glyphAt = (dy: number, tint: string, opacity: number) =>
    def
      ? `<svg x="86" y="${118 + dy}" width="84" height="84" viewBox="${def.viewBox}" fill="none" color="${tint}" opacity="${opacity}">${def.body}</svg>`
      : "";
  // Otomatik: klasör renginin koyu tonuyla kabartma. Seçilmiş renk: o renkte, hafif gölgeli.
  const glyph = !def
    ? ""
    : symbolColor
      ? `<g filter="url(#${k}gs)">${glyphAt(0, symbolColor, 1)}</g>`
      : glyphAt(1.6, "#ffffff", 0.38) + glyphAt(0, engrave, 0.9);

  return `<svg xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}" viewBox="0 0 256 256">
  <defs>
    <linearGradient id="${k}b" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="${back}"/>
      <stop offset="1" stop-color="${backDeep}"/>
    </linearGradient>
    <linearGradient id="${k}f" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="${top}"/>
      <stop offset=".55" stop-color="${color}"/>
      <stop offset="1" stop-color="${bottom}"/>
    </linearGradient>
    <linearGradient id="${k}p" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#ffffff"/>
      <stop offset="1" stop-color="#e9ecf2"/>
    </linearGradient>
    <filter id="${k}s" x="-15%" y="-15%" width="130%" height="140%">
      <feDropShadow dx="0" dy="2" stdDeviation="2" flood-color="#000" flood-opacity=".18"/>
      <feDropShadow dx="0" dy="8" stdDeviation="8" flood-color="#000" flood-opacity=".14"/>
    </filter>
    <filter id="${k}gs" x="-20%" y="-20%" width="140%" height="150%">
      <feDropShadow dx="0" dy="2" stdDeviation="2" flood-color="#000" flood-opacity=".22"/>
    </filter>
    <filter id="${k}ps" x="-10%" y="-30%" width="120%" height="160%">
      <feDropShadow dx="0" dy="-1" stdDeviation="1.5" flood-color="#000" flood-opacity=".12"/>
    </filter>
  </defs>
  <g filter="url(#${k}s)">
    <path fill="url(#${k}b)" d="M22 64a20 20 0 0 1 20-20h46.7a20 20 0 0 1 13.4 5.2L114 60h100a20 20 0 0 1 20 20v118a20 20 0 0 1-20 20H42a20 20 0 0 1-20-20z"/>
    <rect x="34" y="72" width="188" height="60" rx="8" fill="url(#${k}p)" filter="url(#${k}ps)"/>
    <rect x="16" y="90" width="224" height="130" rx="20" fill="url(#${k}f)"/>
    <rect x="16.75" y="90.75" width="222.5" height="128.5" rx="19.25" fill="none" stroke="#fff" stroke-opacity=".28" stroke-width="1.5"/>
  </g>
  ${glyph}
</svg>`;
}

/** SVG'yi kare PNG'ye çevirir (Rust tarafı bunu çok boyutlu .ico yapar). */
export async function folderPng(style: FolderStyle, size = 256): Promise<Uint8Array> {
  const svg = folderSvg(style, size);
  const img = new Image();
  img.src = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`;
  await img.decode();

  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = size;
  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error("Canvas kullanılamıyor");
  ctx.drawImage(img, 0, 0, size, size);

  const blob = await new Promise<Blob>((resolve, reject) =>
    canvas.toBlob((b) => (b ? resolve(b) : reject(new Error("PNG üretilemedi"))), "image/png"),
  );
  return new Uint8Array(await blob.arrayBuffer());
}
