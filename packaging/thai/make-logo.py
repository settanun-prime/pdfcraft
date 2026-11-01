#!/usr/bin/env python3
"""Draw the pdfคับ app icon and write every size of it.

    python3 packaging/thai/make-logo.py            draw from logo-glyphs.json, write assets/app-icon/*
    python3 packaging/thai/make-logo.py --trace    first re-trace the words from Kodchasan Bold (needs the font)
    python3 packaging/thai/make-logo.py --preview FILE.png   only write a 1024 px preview, touch nothing else

The icon is a red page with a folded corner and the word pdfคับ in a pale pink, with pdf large and
tilted away from the fold, and คับ small. Everything is plain polygons, so the SVG and every PNG come from the same shapes. The outlines of the words are traced
from Kodchasan Bold (SIL Open Font License 1.1) and stored in logo-glyphs.json, so drawing needs
neither the font nor any tool besides Pillow and, on a Mac, iconutil.

This script also updates the sha256 values in ATTRIBUTION.toml. Run `cargo xtask assets --write` after it.
"""
import glob
import hashlib
import json
import math
import pathlib
import re
import struct
import subprocess
import sys
import tempfile

from PIL import Image, ImageDraw, ImageFont

ROOT = pathlib.Path(__file__).resolve().parents[2]
ICONS = ROOT / "assets/app-icon"
GLYPHS = pathlib.Path(__file__).with_name("logo-glyphs.json")
ID = "ai.storyteller.pdfcraft"
RED, INK, FOLD = "#c8102e", "#ffe3de", "#8e0a20"
WORDS = ("pdf", "คับ")


# ---- tracing the letters from the font ----------------------------------------------------------
def contours(img, level=128):
    """Closed outlines of the shape in an 8-bit mask (marching squares, edges placed by interpolation)."""
    w, h = img.size
    data = img.tobytes()
    rows = [data[y * w:(y + 1) * w] for y in range(h)]
    adj = {}

    def link(a, b):
        adj.setdefault(a, []).append(b)
        adj.setdefault(b, []).append(a)

    for y in range(h - 1):
        r0, r1 = rows[y], rows[y + 1]
        if not any(r0) and not any(r1):
            continue
        for x in range(w - 1):
            a, b, c, d = r0[x] >= level, r0[x + 1] >= level, r1[x + 1] >= level, r1[x] >= level
            case = (a << 3) | (b << 2) | (c << 1) | d
            if case in (0, 15):
                continue
            top, right, bottom, left = (0, x, y), (1, x + 1, y), (0, x, y + 1), (1, x, y)
            centre = (r0[x] + r0[x + 1] + r1[x + 1] + r1[x]) / 4 >= level
            segs = {
                1: [(left, bottom)], 2: [(bottom, right)], 3: [(left, right)], 4: [(top, right)],
                6: [(top, bottom)], 7: [(top, left)], 8: [(top, left)], 9: [(top, bottom)],
                11: [(top, right)], 12: [(left, right)], 13: [(bottom, right)], 14: [(left, bottom)],
                5: [(top, left), (bottom, right)] if centre else [(top, right), (left, bottom)],
                10: [(top, right), (left, bottom)] if centre else [(top, left), (bottom, right)],
            }[case]
            for s, e in segs:
                link(s, e)

    def point(edge):
        kind, x, y = edge
        if kind == 0:
            v0, v1 = rows[y][x], rows[y][x + 1]
            return (x + (level - v0) / (v1 - v0), float(y))
        v0, v1 = rows[y][x], rows[y + 1][x]
        return (float(x), y + (level - v0) / (v1 - v0))

    seen, loops = set(), []
    for start in adj:
        if start in seen:
            continue
        loop, prev, cur = [], None, start
        while cur not in seen:
            seen.add(cur)
            loop.append(point(cur))
            nxt = [n for n in adj[cur] if n != prev]
            if not nxt:
                break
            prev, cur = cur, nxt[0]
        loops.append(loop)
    return loops


def simplify(points, tolerance):
    """Douglas-Peucker on a closed loop."""
    if len(points) < 4:
        return points

    def dist(p, a, b):
        (px, py), (ax, ay), (bx, by) = p, a, b
        dx, dy = bx - ax, by - ay
        n = dx * dx + dy * dy
        if n == 0:
            return math.hypot(px - ax, py - ay)
        t = max(0, min(1, ((px - ax) * dx + (py - ay) * dy) / n))
        return math.hypot(px - (ax + t * dx), py - (ay + t * dy))

    def chain(pts):
        keep = [False] * len(pts)
        keep[0] = keep[-1] = True
        stack = [(0, len(pts) - 1)]
        while stack:
            i, j = stack.pop()
            far, at = 0.0, -1
            for k in range(i + 1, j):
                d = dist(pts[k], pts[i], pts[j])
                if d > far:
                    far, at = d, k
            if far > tolerance and at > 0:
                keep[at] = True
                stack += [(i, at), (at, j)]
        return [p for p, k in zip(pts, keep) if k]

    far = max(range(len(points)), key=lambda i: math.hypot(points[i][0] - points[0][0], points[i][1] - points[0][1]))
    first = chain(points[:far + 1])
    second = chain(points[far:] + [points[0]])
    return first[:-1] + second[:-1]


def inside(p, poly):
    x, y, c = p[0], p[1], False
    for (x1, y1), (x2, y2) in zip(poly, poly[1:] + poly[:1]):
        if (y1 > y) != (y2 > y) and x < (x2 - x1) * (y - y1) / (y2 - y1) + x1:
            c = not c
    return c


def trace():
    ttc = glob.glob("/System/Library/AssetsV2/com_apple_MobileAsset_Font*/*/AssetData/Kodchasan.ttc")
    if not ttc:
        sys.exit("make-logo: Kodchasan.ttc not found; the outlines in logo-glyphs.json are used as they are")
    em, pad, out = 1000, 400, {}  # the pad keeps Thai marks, which hang left of their origin, in the image
    font = ImageFont.truetype(ttc[0], em, index=6)
    assert font.getname() == ("Kodchasan", "Bold"), font.getname()
    ascent, descent = font.getmetrics()
    for word in WORDS:
        img = Image.new("L", (int(font.getlength(word)) + 2 * pad, ascent + descent + 2 * pad), 0)
        ImageDraw.Draw(img).text((pad, pad + ascent), word, font=font, fill=255, anchor="ls")
        loops = [simplify([(x - pad, y - pad - ascent) for x, y in lp], 0.35) for lp in contours(img)]
        out[word] = [[[round(x, 1), round(y, 1)] for x, y in lp] for lp in loops if len(lp) > 3]
    GLYPHS.write_text(json.dumps(out, ensure_ascii=False, separators=(",", ":")) + "\n", encoding="utf-8")
    print("traced", " ".join(f"{w}:{sum(len(l) for l in lp)}pts" for w, lp in out.items()))


# ---- the drawing ---------------------------------------------------------------------------------
def page(cut=100, r=112):
    """A rounded square with its top right corner cut off for the fold."""
    pts = [(512 - cut, 0), (512, cut)]
    for cx, cy, a0 in ((512 - r, 512 - r, 0), (r, 512 - r, 90), (r, r, 180)):
        pts += [(cx + r * math.cos(math.radians(a)), cy + r * math.sin(math.radians(a))) for a in range(a0, a0 + 91, 5)]
    return pts


def bounds(loops):
    xs, ys = [p[0] for lp in loops for p in lp], [p[1] for lp in loops for p in lp]
    return min(xs), min(ys), max(xs), max(ys)


def at_left(loops, x, top, width):
    """Scale to `width` with the top left of the ink at (x, top)."""
    x0, y0, x1, _ = bounds(loops)
    k = width / (x1 - x0)
    return [[(x + (px - x0) * k, top + (py - y0) * k) for px, py in lp] for lp in loops]


def centred(loops, cx, cy, width):
    """Scale to `width` with the ink centred on (cx, cy)."""
    x0, y0, x1, y1 = bounds(loops)
    k = width / (x1 - x0)
    return [[(cx + (px - (x0 + x1) / 2) * k, cy + (py - (y0 + y1) / 2) * k) for px, py in lp] for lp in loops]


def tilted(loops, degrees):
    """Turned clockwise around the centre of the ink."""
    x0, y0, x1, y1 = bounds(loops)
    cx, cy, a = (x0 + x1) / 2, (y0 + y1) / 2, math.radians(degrees)
    c, s = math.cos(a), math.sin(a)
    return [[(cx + (x - cx) * c - (y - cy) * s, cy + (x - cx) * s + (y - cy) * c) for x, y in lp] for lp in loops]


def layers(glyphs):
    """(colour, loops) from bottom to top, in a 512 px square. pdf tilts down to the right, away from the fold."""
    return [
        (RED, [page()]),
        (FOLD, [[(412, 0), (512, 100), (440, 100), (412, 72)]]),
        (INK, tilted(at_left(glyphs["pdf"], 40, 96, 404), 6)),
        (INK, centred(glyphs["คับ"], 330, 412, 232)),
    ]


def by_depth(loops):
    """(nesting depth, loop), outermost first: even depth fills, odd depth is a hole."""
    depth = [(sum(1 for j, other in enumerate(loops) if j != i and inside(lp[0], other)), lp) for i, lp in enumerate(loops)]
    return sorted(depth, key=lambda item: item[0])


def svg(glyphs):
    def d(loops):
        return "".join("M" + "L".join(f"{x:.1f} {y:.1f}" for x, y in lp) + "Z" for lp in loops)

    paths = "".join(f'<path fill="{colour}" fill-rule="evenodd" d="{d(loops)}"/>' for colour, loops in layers(glyphs))
    return ('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" role="img"><title>pdfคับ</title>'
            f'<g id="thai-logo">{paths}</g></svg>\n')


def rgb(colour):
    return tuple(int(colour[i:i + 2], 16) for i in (1, 3, 5))


def render(glyphs, size, mac):
    """One PNG. `mac`: Apple's grid, an 824 px body centred on a transparent 1024 px canvas."""
    ss = 4 if size <= 512 else 2
    n = size * ss
    k = n / 1024 * (824 / 512) if mac else n / 512
    off = n * 100 / 1024 if mac else 0
    img = Image.new("RGBA", (n, n), rgb(RED) + (0,))  # transparent but red, so the edge blends to red

    for colour, loops in layers(glyphs):
        mask = Image.new("L", (n, n), 0)
        draw = ImageDraw.Draw(mask)
        for depth, lp in by_depth(loops):
            draw.polygon([(off + x * k, off + y * k) for x, y in lp], fill=0 if depth % 2 else 255)
        img.paste(colour, (0, 0), mask)
    return img.resize((size, size), Image.BOX)


def ico(path, pngs):
    blobs = [p.read_bytes() for p in pngs]
    head = struct.pack("<HHH", 0, 1, len(blobs))
    entries, data, offset = b"", b"", 6 + 16 * len(blobs)
    for blob in blobs:
        w, h = struct.unpack(">II", blob[16:24])
        entries += struct.pack("<BBBBHHII", w % 256, h % 256, 0, 0, 1, 32, len(blob), offset)
        data += blob
        offset += len(blob)
    path.write_bytes(head + entries + data)


def update_hashes(files):
    """Write each file's sha256 into its ATTRIBUTION.toml entry."""
    toml = ROOT / "ATTRIBUTION.toml"
    text = toml.read_text(encoding="utf-8")
    for f in files:
        rel = f.relative_to(ROOT).as_posix()
        digest = hashlib.sha256(f.read_bytes()).hexdigest()
        pattern = re.compile(r'(path = "' + re.escape(rel) + r'"\n(?:(?!\[\[asset\]\]).*\n)*?sha256 = ")[0-9a-f]{64}(")')
        text, count = pattern.subn(lambda m: m.group(1) + digest + m.group(2), text)
        if count != 1:
            sys.exit(f"make-logo: {rel} has no unique entry with a sha256 in ATTRIBUTION.toml")
    toml.write_text(text, encoding="utf-8")


def main():
    args = sys.argv[1:]
    if "--trace" in args or not GLYPHS.exists():
        trace()
    glyphs = json.loads(GLYPHS.read_text(encoding="utf-8"))
    if "--preview" in args:
        render(glyphs, 1024, True).save(args[args.index("--preview") + 1])
        return
    written = []

    def put(path, data):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data) if isinstance(data, bytes) else path.write_text(data, encoding="utf-8")
        written.append(path)

    vector = svg(glyphs)
    put(ICONS / "pdfcraft.svg", vector)
    put(ICONS / "pdfcraft-small.svg", vector)
    put(ICONS / "hicolor/scalable/apps" / f"{ID}.svg", vector)
    with tempfile.TemporaryDirectory() as tmp:
        tmp = pathlib.Path(tmp)
        render(glyphs, 1024, True).save(ICONS / "pdfcraft-1024.png")
        written.append(ICONS / "pdfcraft-1024.png")
        for s in (16, 24, 32, 48, 64, 128, 256, 512):
            path = ICONS / f"hicolor/{s}x{s}/apps/{ID}.png"
            path.parent.mkdir(parents=True, exist_ok=True)
            render(glyphs, s, False).save(path)
            written.append(path)
        pngs = []
        for s in (16, 20, 24, 32, 40, 48, 64, 128, 256):
            render(glyphs, s, False).save(tmp / f"ico-{s}.png")
            pngs.append(tmp / f"ico-{s}.png")
        ico(ICONS / "pdfcraft.ico", pngs)
        written.append(ICONS / "pdfcraft.ico")
        iconset = tmp / "pdfcraft.iconset"
        iconset.mkdir()
        for s in (16, 32, 128, 256, 512):
            render(glyphs, s, True).save(iconset / f"icon_{s}x{s}.png")
            render(glyphs, s * 2, True).save(iconset / f"icon_{s}x{s}@2x.png")
        try:
            subprocess.run(["iconutil", "-c", "icns", "-o", str(ICONS / "pdfcraft.icns"), str(iconset)], check=True)
            written.append(ICONS / "pdfcraft.icns")
        except (FileNotFoundError, subprocess.CalledProcessError):
            print("make-logo: iconutil is missing (macOS only); pdfcraft.icns was not rewritten")
    update_hashes(written)
    print(f"make-logo: wrote {len(written)} files; sha256 updated in ATTRIBUTION.toml")


if __name__ == "__main__":
    main()
