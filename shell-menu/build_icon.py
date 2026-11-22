"""Sağ tık menüsü ikonunu (blazon-menu.ico) başlık çubuğu bukalemunundan üretir.

    pip install pillow
    python shell-menu/build_icon.py
"""
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent
SOURCE = ROOT.parent / "design/logo/chameleon/titlebar/chameleon-transparent.png"
SIZES = (16, 20, 24, 32, 40, 48, 64, 256)

art = Image.open(SOURCE).convert("RGBA")
art = art.crop(art.getbbox())
side = max(art.size)
square = Image.new("RGBA", (side, side))
square.paste(art, ((side - art.width) // 2, (side - art.height) // 2))

frames = [square.resize((s, s), Image.Resampling.LANCZOS) for s in SIZES]
frames[-1].save(ROOT / "blazon-menu.ico", sizes=[(s, s) for s in SIZES], append_images=frames[:-1])
