#!/usr/bin/env python3
"""Check that this fork's changes survived a merge from upstream. Needs no build.

    python3 packaging/custom-check.py [folder]

A clean merge can still undo them: upstream can add a new promotion, or change the code they rely
on, with no conflict. Run this after every `git merge main`. It exits 1 and says what came back or
went missing. CUSTOM.md says how to put each one right.
"""
import hashlib
import re
import sys
from pathlib import Path

ROOT = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else Path(__file__).resolve().parent.parent

HOME = "crates/ui-egui/src/home.rs"
CHROME = "crates/ui-egui/src/chrome.rs"
LIB = "crates/ui-egui/src/lib.rs"
APP = "crates/ui-egui/src/app.rs"
CRAFT = "crates/fonts/src/craft.rs"
UI_FONTS = "crates/ui-egui/src/platform/system_fonts.rs"
THEME = "crates/ui-egui/src/theme.rs"
EDIT = "crates/edit/src/text_regions/styled.rs"
VIEWER = "crates/render/src/system_font.rs"
ICON = "assets/app-icon/pdfcraft.svg"
WIDGETS = "crates/ui-egui/src/widgets.rs"
DIALOGS = "crates/ui-egui/src/dialogs.rs"
COMMANDS = "crates/engine/src/commands.rs"
LINKS = "crates/engine/src/links.rs"
README = "README.md"
ATTR = "ATTRIBUTION.toml"
CREDITS = "packaging/macos/Credits.rtf"
WORKFLOW = ".github/workflows/thai-build.yml"
NOTES = "packaging/thai/release-notes.md"
OCR = "crates/ocr/src/lib.rs"
OCR_LAYER = "crates/engine/src/ocr/layer.rs"
OCR_UI = "crates/ui-egui/src/workflows/ocr_ui.rs"
OCR_PP = "crates/ocr/src/pp.rs"
OCR_ENGINE = "crates/engine/src/ocr.rs"
DESKTOP = "apps/pdfcraft/src/main.rs"
UPDATES = "crates/ui-egui/src/platform/updates.rs"

# Promotions: none of these may be on the home screen or in the title bar.
PROMOTION = re.compile(r"discord|community|sponsor|donat|join (the|our)|newsletter", re.IGNORECASE)

ABSENT = [
    (LIB, "community_card_closed", "the community card's state is back"),
    (APP, "community_card_closed", "the community card's state is back"),
    (WIDGETS, "artcraft_logo", "the ArtCraft wordmark is drawn again"),
    (WIDGETS, "artcraft_mark", "the ArtCraft mark is drawn again"),
    (DIALOGS, "artcraft_", "the About dialog draws an ArtCraft mark again"),
    (COMMANDS, "help.discord", "the Help menu has a Discord entry again"),
    (COMMANDS, "help.website", "the Help menu has an ArtCraft website entry again"),
    (LINKS, "discord.gg", "the links table points to Discord again"),
    (ATTR, 'path = "docs/brand/', "ATTRIBUTION.toml lists the ArtCraft logos again"),
    (ATTR, 'path = "docs/images/pdfcraft-tools.png"', "ATTRIBUTION.toml lists the screenshot that shows the ArtCraft mark again"),
    (README, "docs/brand", "the README shows the ArtCraft logos again; keep this edition's own README"),
    (README, "discord.gg/artcraft", "the README promotes the community again; keep this edition's own README"),
    (CRAFT, "PathBuf::from(\"/Applications/Microsoft Word.app", "a Microsoft font is a candidate for writing into PDFs again"),
    (UI_FONTS, "Thonburi.ttc", "Thonburi is listed for the interface; its marks show dotted circles without shaping"),
    (CREDITS, "discord", "the macOS About panel links to Discord again"),
    (CREDITS, "getartcraft.com", "the macOS About panel links to the ArtCraft site again"),
    (WORKFLOW, "xlarge", "the Thai build uses a paid macOS runner; keep the free macos-latest"),
    (NOTES, "discord", "the release notes promote the community again"),
    (NOTES, "getartcraft", "the release notes link to the ArtCraft site again"),
    (DESKTOP, "set_startup_update_check(true)", "the app asks upstream for updates at start again; a newer upstream build would replace this edition"),
    (UPDATES, "RELEASES_PAGE: &str = \"https://github.com/storytold", "Check for updates points to upstream's releases again"),
]

PRESENT = [
    (WORKFLOW, "CRAFT_I18N_REQUIRED=1", "the Thai build no longer brings the translations; its interface would be English only"),
    (WORKFLOW, "CRAFT_FONTS_REQUIRED=1", "the Thai build no longer brings craft-fonts (Noto Sans Thai)"),
    (LIB, 'repo: "settanun-prime/pdfcraft"', "the updater can download upstream's releases again"),
    (CRAFT, "validate_document_input(&bytes)", "an installed Thai font no longer has to declare an open licence before it is written into a PDF"),
    (CRAFT, ".or_else(installed_thai_font)", "writing Thai no longer falls back to an installed font"),
    (CRAFT, "fn standalone_face", "a face can't be taken out of a .ttc any more"),
    (CRAFT, '"Sarabun.ttc"', "Sarabun is gone from the Thai document fonts"),
    (UI_FONTS, "fn thai_fallback", "the interface has no installed Thai font"),
    (UI_FONTS, "fn thai_face", "the Thai interface face is no longer picked by style, so Sukhumvit Set draws in Thin"),
    (UI_FONTS, 'macos_candidates(&["SukhumvitSet.ttc"', "Sukhumvit Set no longer comes first for the interface"),
    (THEME, "thai_fallback()", "the installed Thai font is not on the interface"),
    (VIEWER, "names_thai_family", "PDFs that don't embed a Thai font no longer use an installed one"),
    (VIEWER, '&["Supplemental/SukhumvitSet.ttc", "Supplemental/Sathu.ttf"', "Sukhumvit Set no longer comes first in the viewer; tone marks disappear without it"),
    (ICON, 'id="thai-logo"', "the app icon is not the Thai logo (the upstream lion is back?)"),
    (CREDITS, "unofficial Thai edition", "the macOS About panel no longer says this is an unofficial edition"),
    (WORKFLOW, "runs-on: macos-latest", "the Thai build workflow is gone or no longer uses the free macOS runner"),
    (WORKFLOW, "--draft", "the Thai build would publish a release without a draft stage"),
    (NOTES, "unofficial Thai edition", "the release notes no longer say this is an unofficial edition"),
    (OCR, '("th", "Thai and English")', "Thai is gone from the OCR languages"),
    (OCR_PP, "THAI_MIN_SCORE", "Thai OCR no longer filters out artwork read as letters"),
    (OCR_ENGINE, "pdfcraft_ocr::pp::is_pp_language(&settings.language)", "Thai no longer reads with PP-OCR; upstream's ocrs reads only Latin letters"),
    (OCR_LAYER, "fn invisible_font", "the OCR text layer is no longer the Type 0 font Apple's PDF reader can see"),
    (OCR_UI, 'available_for("th")', "the OCR dialog no longer opens on Thai"),
    (OCR_UI, "settings.read_pictures = true", "Edit no longer reads the picture on a page that also has text (a certificate)"),
    (ATTR, 'file = "thai-recognition.onnx"', "the Thai OCR model is no longer fetched"),
]


def read(name):
    path = ROOT / name
    return path.read_text(encoding="utf-8") if path.is_file() else None


def promotions(name):
    """Promotion words in code the app draws. Comments don't draw anything."""
    text = read(name)
    if text is None:
        return [f"{name} is missing"]
    found = []
    for number, line in enumerate(text.splitlines(), 1):
        code = line.split("//", 1)[0]
        if PROMOTION.search(code):
            found.append(f"{name}:{number} looks like a promotion: {code.strip()[:80]}")
    return found


GONE = ["docs/images/pdfcraft-tools.png"]
# upstream's licence check requires the brand licence text, so that one file stays; the logos do not.
BRAND_KEPT = {"LICENSE-brand.txt"}


def brand_files():
    """The ArtCraft logos and the screenshot that shows them must not be in this edition."""
    found = [f"{name} is back; remove it, this edition carries no ArtCraft mark" for name in GONE if (ROOT / name).exists()]
    folder = ROOT / "docs/brand"
    if folder.is_dir():
        found += [f"docs/brand/{f.name} is back; remove it, this edition carries no ArtCraft mark" for f in sorted(folder.iterdir()) if f.name not in BRAND_KEPT]
    return found


def icon_hashes():
    """The stamped icons have to be the ones ATTRIBUTION.toml records."""
    manifest = read("ATTRIBUTION.toml")
    if manifest is None:
        return ["ATTRIBUTION.toml is missing"]
    found = []
    for name in ("assets/app-icon/pdfcraft-1024.png", "assets/app-icon/pdfcraft.icns", "assets/app-icon/pdfcraft.svg"):
        path = ROOT / name
        match = re.search(r'path = "' + re.escape(name) + r'"\n(?:(?!\[\[asset\]\]).*\n)*?sha256 = "([0-9a-f]{64})"', manifest)
        if not path.is_file() or match is None:
            found.append(f"{name} is missing, or has no sha256 in ATTRIBUTION.toml")
        elif hashlib.sha256(path.read_bytes()).hexdigest() != match.group(1):
            found.append(f"{name} doesn't match its sha256 in ATTRIBUTION.toml")
    return found


def main():
    problems = promotions(HOME) + promotions(CHROME)
    for name, needle, message in ABSENT:
        text = read(name)
        if text is None or needle in text:
            problems.append(f"{name}: {message}")
    for name, needle, message in PRESENT:
        text = read(name)
        if text is None or needle not in text:
            problems.append(f"{name}: {message}")
    problems += icon_hashes() + brand_files()
    if problems:
        print(f"custom-check: {len(problems)} problem(s), see CUSTOM.md")
        for problem in problems:
            print(f"  - {problem}")
        return 1
    print(f"custom-check: ok ({len(PRESENT) + len(ABSENT) + len(GONE) + 5} checks)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
