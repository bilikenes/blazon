"""Windows ikonunu (icon.ico) boyuta göre doğru çizimden kurar.

16–32 px: küçük boyut kesimi (blazon-symbol-small.svg) — kalın boşluk, iki damla.
40 px ve üstü: ana işaret (blazon-symbol.svg).

Önce:  python design/logo/build_logo.py
       npx tauri icon design/logo/final/blazon-symbol-1024.png   (diğer platform PNG'leri)
Sonra: python design/logo/build_icons.py
PNG çizimi için logo-design skill'inin render_png.py betiği kullanılır (Chrome gerekir).
"""
import os
import struct
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FINAL = ROOT / "design" / "logo" / "final"
ICONS = ROOT / "src-tauri" / "icons"
RENDER = Path(os.environ.get("LOGO_SKILL_DIR", Path.home() / ".claude/skills/logo-design")) / "scripts/render_png.py"

SMALL = [16, 20, 24, 32]
LARGE = [40, 48, 64, 128, 256]


def render(svg: Path, size: int, out: Path) -> bytes:
    subprocess.run([sys.executable, "-I", str(RENDER), str(svg), "-o", str(out), "--size", str(size)],
                   check=True, stdout=subprocess.DEVNULL)
    return out.read_bytes()


def build_ico(images: list[tuple[int, bytes]]) -> bytes:
    header = struct.pack("<HHH", 0, 1, len(images))
    offset = 6 + 16 * len(images)
    entries, data = b"", b""
    for size, png in images:
        dim = 0 if size >= 256 else size
        entries += struct.pack("<BBBBHHII", dim, dim, 0, 0, 1, 32, len(png), offset + len(data))
        data += png
    return header + entries + data


def main() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        t = Path(tmp)
        imgs = [(s, render(FINAL / "blazon-symbol-small.svg", s, t / f"s{s}.png")) for s in SMALL]
        imgs += [(s, render(FINAL / "blazon-symbol.svg", s, t / f"l{s}.png")) for s in LARGE]
        (ICONS / "icon.ico").write_bytes(build_ico(imgs))
        (ICONS / "32x32.png").write_bytes(dict(imgs)[32])
    print("wrote", ICONS / "icon.ico")


if __name__ == "__main__":
    main()
