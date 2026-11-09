"""Blazon logo master dosyalarını üretir (design/logo/final).

Kullanım:  python design/logo/build_logo.py
Geometri burada tanımlı; SVG'leri elle düzenlemek yerine bu dosyayı değiştirip yeniden çalıştırın.
"""
from pathlib import Path

OUT = Path(__file__).parent / "final"
OUT.mkdir(exist_ok=True)

PLUM = "#9b6bdf"    # arka kapak
CORAL = "#ee7a64"   # ön kapak + damlalar
INK = "#23262f"     # yazı (açık zemin)
PAPER = "#ffffff"   # yazı (koyu zemin)


def svg(body: str, title: str, w: int = 256, h: int = 256) -> str:
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}">'
            f"<title>{title}</title>{body}</svg>\n")


def write(name: str, content: str) -> None:
    (OUT / name).write_text(content, encoding="utf-8")


def tab(x0, y_top, x_sh, y_body, r):
    """Klasör sekmesi: yuvarlak sol üst köşe, yumuşak omuz, 45° iniş."""
    d = y_body - y_top - 4
    return (f"M{x0} {y_top + r}A{r} {r} 0 0 1 {x0 + r} {y_top}H{x_sh - 6}"
            f"Q{x_sh} {y_top} {x_sh + 4} {y_top + 4}L{x_sh + 4 + d} {y_body}")


def symbol_paths(dy: float = -10):
    """Ana işaret (48 px ve üstü). dy: optik ortalama için dikey kaydırma."""
    Y = lambda v: v + dy
    back = (tab(32, Y(48), 92, Y(70), 18)
            + f"H206A18 18 0 0 1 224 {Y(88)}V{Y(94)}H32Z")
    bot, fr = Y(176), 8
    drips = [(178, 20, Y(204)), (116, 24, Y(226)), (66, 18, Y(198))]
    d = (f"M32 {Y(116)}A16 16 0 0 1 48 {Y(100)}H208A16 16 0 0 1 224 {Y(116)}"
         f"V{Y(160)}A16 16 0 0 1 208 {bot}")
    for cx, wd, by in drips:
        r = wd / 2
        xr, xl = cx + r, cx - r
        d += (f"H{xr + fr}A{fr} {fr} 0 0 0 {xr} {bot + fr}V{by - r}A{r} {r} 0 0 1 {xl} {by - r}"
              f"V{bot + fr}A{fr} {fr} 0 0 0 {xl - fr} {bot}")
    d += f"H48A16 16 0 0 1 32 {Y(160)}Z"
    return back, d


def small_paths():
    """16–32 px kesimi: 16'lık ızgara, kalın boşluk, iki damla."""
    back = tab(24, 36, 96, 64, 20) + "H212A20 20 0 0 1 232 84V96H24Z"
    # ön kapak y 112..184, damlalar: geniş orta + sağ kısa
    d = "M24 132A20 20 0 0 1 44 112H212A20 20 0 0 1 232 132V164A20 20 0 0 1 212 184"
    for cx, wd, by in [(184, 32, 208), (104, 40, 232)]:
        r = wd / 2
        d += (f"H{cx + r}V{by - r}A{r} {r} 0 0 1 {cx - r} {by - r}V184")
    d += "H44A20 20 0 0 1 24 164Z"
    return back, d


# ---------- Yazı: "Blazon" (yuvarlak uçlu geometrik harfler, kontur) ----------
S = 24  # kontur kalınlığı


def wordmark_body(color: str, x: float = 0, y: float = 0, scale: float = 1) -> str:
    # Harf merkez çizgileri: üst 52 (B) / 44 (l), x-yüksekliği 108, taban 188, kase yarıçapı 40.
    parts = [
        "M12 52H52A30 30 0 0 1 52 112H12M12 112H58A38 38 0 0 1 58 188H12Z",   # B
        "M142 44V188",                                                         # l
        "M184 148a40 40 0 1 0 80 0a40 40 0 1 0 -80 0M264 108V188",             # a
        "M308 108H372L308 188H372",                                            # z
        "M414 148a40 40 0 1 0 80 0a40 40 0 1 0 -80 0",                         # o
        "M538 188V108M538 140A32 32 0 0 1 602 140V188",                        # n
    ]
    # Aralık: düz–düz ~22, düz–yuvarlak ~18 birim (kontur kenarından ölçülür).
    return (f'<g transform="translate({x} {y}) scale({scale})" fill="none" stroke="{color}" '
            f'stroke-width="{S}" stroke-linecap="round" stroke-linejoin="round">'
            + "".join(f'<path d="{p}"/>' for p in parts) + "</g>")


WORD_W, WORD_TOP, WORD_BOTTOM = 616, 32, 200  # konturla birlikte sınırlar


def symbol_body(colors=(PLUM, CORAL), small=False) -> str:
    back, front = small_paths() if small else symbol_paths()
    return f'<path fill="{colors[0]}" d="{back}"/><path fill="{colors[1]}" d="{front}"/>'


def main() -> None:
    T = "Blazon"
    # Sembol
    write("blazon-symbol.svg", svg(symbol_body(), T))
    write("blazon-symbol-small.svg", svg(symbol_body(small=True), T))
    write("blazon-symbol-black.svg", svg(symbol_body(("#000", "#000")), T))
    write("blazon-symbol-white.svg", svg(symbol_body((PAPER, PAPER)), T))

    # Yazı
    h = WORD_BOTTOM - WORD_TOP
    for name, c in [("blazon-wordmark.svg", INK), ("blazon-wordmark-white.svg", PAPER)]:
        write(name, svg(wordmark_body(c, 0, -WORD_TOP), T, WORD_W, h))

    # Yatay yerleşim: yükseklik 256, yazı x-yüksekliği sembolün ön kapağına hizalı
    k = 0.62
    wx, wy = 256 + 20, 128 - (WORD_TOP + h / 2) * k - 6
    lw = int(wx + WORD_W * k + 8)
    for name, c in [("blazon-horizontal.svg", INK), ("blazon-horizontal-white.svg", PAPER)]:
        write(name, svg(symbol_body() + wordmark_body(c, wx, wy, k), T, lw, 256))

    # Dikey yerleşim
    k2 = 0.5
    sw = int(WORD_W * k2) + 40
    sx = (sw - 256) / 2
    for name, c in [("blazon-stacked.svg", INK), ("blazon-stacked-white.svg", PAPER)]:
        body = f'<g transform="translate({sx} 0)">{symbol_body()}</g>' + \
               wordmark_body(c, 20, 236 - WORD_TOP * k2, k2)
        write(name, svg(body, T, sw, int(236 + h * k2 + 16)))
    print("wrote", OUT)


if __name__ == "__main__":
    main()
