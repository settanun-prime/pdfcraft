#!/usr/bin/env python3
"""Stamp a TH badge on the stock app icons, for the Thai build of this fork.

Run it from anywhere on unstamped icons (needs Pillow; iconutil on macOS for the .icns):

    python3 packaging/thai-badge.py

It stamps the master vectors and every rendered size, so it needs no resvg, then updates the
sha256 values in ATTRIBUTION.toml. Run `cargo xtask assets --write` afterwards. After taking new
icons from upstream, run it again. The badge is drawn from rectangles, so it uses no font.
"""
import hashlib
import re
import shutil
import struct
import subprocess
import sys
import tempfile
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
DIR = ROOT / "assets" / "app-icon"
APP_ID = "ai.storyteller.pdfcraft"
MARK = 'id="thai-badge"'

INK, PAPER, RED = "#0b0b0c", "#efe9dc", "#b70228"
# In the master's 512 unit tile: inside the tile's rounded corner, over the lion's mane.
BOX = (268, 356, 478, 476)
RADIUS = 36
LETTERS = [
    # T
    (298, 380, 362, 400),
    (320, 400, 340, 452),
    # H
    (382, 380, 402, 452),
    (428, 380, 448, 452),
    (402, 406, 428, 426),
]
# Below this many pixels the letters would be under 5 px tall, so the badge grows from its corner.
SMALL = 40
SMALL_GROWTH = 1.45

SVG_BADGE = (
    f'<g {MARK}>'
    f'<rect x="268" y="356" width="210" height="120" rx="36" fill="{RED}" stroke="{INK}" stroke-width="18"/>'
    f'<rect x="268" y="356" width="210" height="120" rx="36" fill="{RED}" stroke="{PAPER}" stroke-width="8"/>'
    f'<path fill="{PAPER}" d="M298 380h64v20h-22v52h-20v-52h-22zM382 380h20v26h26v-26h20v72h-20v-26h-26v26h-20z"/>'
    "</g>"
)


def stamp(image, unit, origin=(0, 0), growth=1.0):
    """Lay the badge over `image`: `unit` pixels per tile unit, the tile at `origin` pixels."""
    over = 4
    layer = Image.new("RGBA", (image.width * over, image.height * over), (0, 0, 0, 0))
    draw = ImageDraw.Draw(layer)
    anchor_x, anchor_y = BOX[2], BOX[3]

    def point(x, y):
        grown = (anchor_x + (x - anchor_x) * growth, anchor_y + (y - anchor_y) * growth)
        return ((origin[0] + grown[0] * unit) * over, (origin[1] + grown[1] * unit) * over)

    def rect(box, grow=0.0):
        left, top = point(box[0] - grow, box[1] - grow)
        right, bottom = point(box[2] + grow, box[3] + grow)
        return [left, top, right, bottom]

    scale = unit * over * growth
    draw.rounded_rectangle(rect(BOX, 9), radius=(RADIUS + 9) * scale, fill=INK)
    draw.rounded_rectangle(rect(BOX, 4), radius=(RADIUS + 4) * scale, fill=PAPER)
    draw.rounded_rectangle(rect(BOX, -4), radius=(RADIUS - 4) * scale, fill=RED)
    for letter in LETTERS:
        draw.rectangle(rect(letter), fill=PAPER)
    layer = layer.resize(image.size, Image.Resampling.LANCZOS)
    return Image.alpha_composite(image.convert("RGBA"), layer)


def hicolor(size):
    return DIR / "hicolor" / f"{size}x{size}" / "apps" / f"{APP_ID}.png"


def write_ico(path, images):
    """PNG-compressed entries, as packaging/icons.sh writes them."""
    blobs = []
    for image in images:
        with tempfile.NamedTemporaryFile(suffix=".png") as file:
            image.save(file.name)
            blobs.append(Path(file.name).read_bytes())
    head = struct.pack("<HHH", 0, 1, len(blobs))
    entries, data, offset = b"", b"", 6 + 16 * len(blobs)
    for blob in blobs:
        width, height = struct.unpack(">II", blob[16:24])
        entries += struct.pack("<BBBBHHII", width % 256, height % 256, 0, 0, 1, 32, len(blob), offset)
        data += blob
        offset += len(blob)
    path.write_bytes(head + entries + data)


def write_icns(grid):
    if shutil.which("iconutil") is None:
        print("warning: iconutil not found (macOS only); pdfcraft.icns not regenerated", file=sys.stderr)
        return False
    with tempfile.TemporaryDirectory() as tmp:
        iconset = Path(tmp) / "pdfcraft.iconset"
        iconset.mkdir()
        for size in (16, 32, 128, 256, 512):
            for name, pixels in ((f"icon_{size}x{size}.png", size), (f"icon_{size}x{size}@2x.png", size * 2)):
                grid.resize((pixels, pixels), Image.Resampling.LANCZOS).save(iconset / name)
        subprocess.run(["iconutil", "-c", "icns", "-o", str(DIR / "pdfcraft.icns"), str(iconset)], check=True)
    return True


def update_attribution(paths):
    file = ROOT / "ATTRIBUTION.toml"
    text = file.read_text(encoding="utf-8")
    for path in paths:
        name = path.relative_to(ROOT).as_posix()
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        pattern = re.compile(r'(path = "' + re.escape(name) + r'"\n(?:(?!\[\[asset\]\]).*\n)*?sha256 = ")[0-9a-f]{64}(")')
        text, count = pattern.subn(rf"\g<1>{digest}\g<2>", text)
        if count != 1:
            sys.exit(f"error: {name} has no sha256 line in ATTRIBUTION.toml")
    file.write_text(text, encoding="utf-8")


def main():
    master = DIR / "pdfcraft.svg"
    svg = master.read_text(encoding="utf-8")
    if MARK in svg:
        sys.exit("the icons already have the badge; take the stock icons first to stamp them again")

    changed = []
    for path in (master, DIR / "pdfcraft-small.svg"):
        text = path.read_text(encoding="utf-8").rstrip()
        if not text.endswith("</g></svg>"):
            sys.exit(f"error: {path.name} doesn't end the way this script expects")
        path.write_text(text[: -len("</g></svg>")] + SVG_BADGE + "</g></svg>\n", encoding="utf-8")
        changed.append(path)
    scalable = DIR / "hicolor" / "scalable" / "apps" / f"{APP_ID}.svg"
    shutil.copyfile(master, scalable)
    changed.append(scalable)

    tile = None
    for size in (16, 24, 32, 48, 64, 128, 256, 512):
        path = hicolor(size)
        growth = SMALL_GROWTH if size < SMALL else 1.0
        image = stamp(Image.open(path).convert("RGBA"), size / 512, growth=growth)
        image.save(path)
        changed.append(path)
        tile = image

    # Apple's grid: the 824 px body centred on the 1024 canvas.
    grid_path = DIR / "pdfcraft-1024.png"
    grid = stamp(Image.open(grid_path).convert("RGBA"), 824 / 512, origin=(100, 100))
    grid.save(grid_path)
    changed.append(grid_path)

    ico_sizes = (16, 20, 24, 32, 40, 48, 64, 128, 256)
    ico = []
    for size in ico_sizes:
        ico.append(Image.open(hicolor(size)).convert("RGBA") if hicolor(size).exists() else tile.resize((size, size), Image.Resampling.LANCZOS))
    ico_path = DIR / "pdfcraft.ico"
    write_ico(ico_path, ico)
    changed.append(ico_path)

    if write_icns(grid):
        changed.append(DIR / "pdfcraft.icns")

    update_attribution(changed)
    print(f"stamped {len(changed)} files; now run: cargo xtask assets --write")


if __name__ == "__main__":
    main()
