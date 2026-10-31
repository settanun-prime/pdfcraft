//! Fonts from the optional craft-fonts build input (<https://github.com/storytold/craft-fonts>).
//!
//! `build.rs` embeds every font in craft-fonts' manifest when PdfCraft is built with
//! `CRAFT_FONTS_DIR=<checkout>`; otherwise [`CRAFT_FONTS`] is empty and everything here returns
//! nothing. Callers must work either way. Thai alone can also come from a face installed on the
//! machine ([`installed_thai_font`]).

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// A font from the optional craft-fonts build input (empty unless built with `CRAFT_FONTS_DIR`).
pub struct CraftFont {
    pub family: &'static str,
    pub style: &'static str,
    /// ISO 15924 scripts the font is for, e.g. `"Jpan"`.
    pub scripts: &'static [&'static str],
    pub bytes: &'static [u8],
}

include!(concat!(env!("OUT_DIR"), "/craft_fonts.rs"));

impl CraftFont {
    /// Whether the font is meant for `script` (ISO 15924, e.g. `"Jpan"`).
    pub fn covers(&self, script: &str) -> bool {
        self.scripts.contains(&script)
    }

    /// A unique name for registering the face with a font system ("BIZ UDPGothic Bold").
    pub fn name(&self) -> String {
        format!("{} {}", self.family, self.style)
    }
}

/// The `Jpan` craft-fonts faces in the order the interface prefers them: BIZ UDPGothic (a UI
/// face) first, then the rest in manifest order. Empty without craft-fonts.
pub fn ui_japanese_fonts() -> Vec<&'static CraftFont> {
    let mut fonts: Vec<&CraftFont> = CRAFT_FONTS.iter().filter(|f| f.covers("Jpan")).collect();
    // Stable: manifest order within each group.
    fonts.sort_by_key(|f| f.family != "BIZ UDPGothic");
    fonts
}

/// The `Hans` craft-fonts faces for Simplified Chinese interface text, in manifest order.
/// Empty when built without craft-fonts (Chinese text then shows the font system's
/// replacement glyph, the same degraded mode as Japanese without craft-fonts).
pub fn ui_chinese_fonts() -> Vec<&'static CraftFont> {
    CRAFT_FONTS.iter().filter(|f| f.covers("Hans")).collect()
}

/// The `Arab` craft-fonts faces for Arabic-script interface text (file names, document titles),
/// in manifest order. Empty when built without craft-fonts or when it has no Arabic face.
pub fn ui_arabic_fonts() -> Vec<&'static CraftFont> {
    arabic(CRAFT_FONTS.iter())
}

/// The face for Arabic text written into PDFs: the first `Arab` face. `None` when built without
/// craft-fonts or when its revision has no Arabic face.
pub fn document_arabic_font() -> Option<&'static CraftFont> {
    ui_arabic_fonts().into_iter().next()
}

fn arabic<'a>(faces: impl IntoIterator<Item = &'a CraftFont>) -> Vec<&'a CraftFont> {
    faces.into_iter().filter(|f| f.covers("Arab")).collect()
}

/// The `Telu` craft-fonts faces for Telugu interface text (the Telugu catalog, file names,
/// document titles), in manifest order. Empty when built without craft-fonts or when it has no
/// Telugu face.
pub fn ui_telugu_fonts() -> Vec<&'static CraftFont> {
    CRAFT_FONTS.iter().filter(|f| f.covers("Telu")).collect()
}

/// Thai faces shared by the interface and PDF text writer.
pub fn ui_thai_fonts() -> Vec<&'static CraftFont> {
    CRAFT_FONTS.iter().filter(|f| f.covers("Thai")).collect()
}

/// The face for Thai text written into PDFs: the first `Thai` face, else [`installed_thai_font`].
pub fn document_thai_font() -> Option<&'static CraftFont> {
    ui_thai_fonts().into_iter().next().or_else(installed_thai_font)
}

/// Thai characters a document face must have: letters, a vowel, a tone mark and a digit.
const THAI_PROBE: &str = "กฮำ่๑";

/// The largest installed face read, which is also what [`crate::embedding::validate`] accepts.
const MAX_INSTALLED_BYTES: u64 = 16 << 20;
/// Faces looked at in an installed collection (`.ttc`).
const MAX_COLLECTION_FACES: u32 = 32;
/// Downloaded font assets looked at per kind on macOS.
const MAX_FONT_ASSETS: usize = 512;

/// An openly licensed Thai face already installed on this machine, read once. Text written with it
/// carries the face's outlines into the PDF, unlike every other installed face, so the face has to
/// declare its licence (`embedding::validate_document_input`: OFL, Apache, MIT…). Embedding bits
/// alone don't qualify, so Tahoma and the fonts that come with Word are never used for this.
/// `PDFCRAFT_SYSTEM_FONTS=0` turns it off.
pub fn installed_thai_font() -> Option<&'static CraftFont> {
    static FACE: OnceLock<Option<CraftFont>> = OnceLock::new();
    FACE.get_or_init(|| {
        if cfg!(target_arch = "wasm32") || std::env::var_os("PDFCRAFT_SYSTEM_FONTS").is_some_and(|v| v == "0") {
            return None;
        }
        installed_thai_candidates().iter().find_map(|path| read_thai_face(path))
    })
    .as_ref()
}

/// The face at `path` when it is a regular file that shows Thai, may be embedded and declares an
/// open licence, the Regular one of a collection. Its bytes live as long as the process: this runs
/// once, and [`CraftFont`] holds static data.
fn read_thai_face(path: &Path) -> Option<CraftFont> {
    let meta = std::fs::metadata(path).ok()?;
    if !meta.is_file() || meta.len() > MAX_INSTALLED_BYTES {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    let bytes = if bytes.starts_with(b"ttcf") { standalone_face(&bytes, regular_face(&bytes)?)? } else { bytes };
    if !crate::embedding::covers(&bytes, THAI_PROBE) {
        return None;
    }
    // Permission to embed is not a licence: the face has to declare an open one.
    crate::embedding::validate_document_input(&bytes).ok()?;
    let (family, _) = crate::embedding::names(&bytes).ok()?;
    let family: &'static str = Box::leak(family.into_boxed_str());
    let bytes: &'static [u8] = Box::leak(bytes.into_boxed_slice());
    Some(CraftFont { family, style: "Regular", scripts: &["Thai"], bytes })
}

/// The Regular face of a collection. Its full name says so: Sarabun's Medium and Thin faces call
/// themselves Regular too, each in a family of its own.
fn regular_face(bytes: &[u8]) -> Option<u32> {
    use skrifa::MetadataProvider as _;
    use skrifa::string::StringId;
    (0..MAX_COLLECTION_FACES).find(|&index| {
        skrifa::FontRef::from_index(bytes, index).is_ok_and(|font| {
            font.localized_strings(StringId::FULL_NAME).any(|name| name.to_string().ends_with(" Regular")) && font.charmap().map('\u{0E01}').is_some()
        })
    })
}

/// Face `index` of a collection as a font of its own, for shaping and embedding, which take one
/// font. Each table is copied whole, so the face draws exactly as it does in the collection.
fn standalone_face(bytes: &[u8], index: u32) -> Option<Vec<u8>> {
    const HEADER: usize = 12;
    const RECORD: usize = 16;
    let u32_at = |at: usize| Some(u32::from_be_bytes(bytes.get(at..at.checked_add(4)?)?.try_into().ok()?));
    let directory = u32_at(usize::try_from(index).ok()?.checked_mul(4)?.checked_add(HEADER)?)?;
    let directory = usize::try_from(directory).ok()?;
    let head = bytes.get(directory..directory.checked_add(HEADER)?)?;
    let tables = usize::from(u16::from_be_bytes(head.get(4..6)?.try_into().ok()?));
    let mut font = head.to_vec();
    let mut body = Vec::new();
    let body_start = tables.checked_mul(RECORD)?.checked_add(HEADER)?;
    for table in 0..tables {
        let at = table.checked_mul(RECORD)?.checked_add(directory)?.checked_add(HEADER)?;
        let record = bytes.get(at..at.checked_add(RECORD)?)?;
        let start = usize::try_from(u32_at(at.checked_add(8)?)?).ok()?;
        let length = usize::try_from(u32_at(at.checked_add(12)?)?).ok()?;
        let data = bytes.get(start..start.checked_add(length)?)?;
        let offset = u32::try_from(body_start.checked_add(body.len())?).ok()?;
        font.extend_from_slice(record.get(..8)?);
        font.extend_from_slice(&offset.to_be_bytes());
        font.extend_from_slice(record.get(12..)?);
        body.extend_from_slice(data);
        // Tables start on four-byte boundaries.
        body.resize(body.len().checked_next_multiple_of(4)?, 0);
        if u64::try_from(body.len()).ok()? > MAX_INSTALLED_BYTES {
            return None;
        }
    }
    font.extend_from_slice(&body);
    Some(font)
}

/// The `AssetData` folders of the fonts macOS downloads, whose ids change with each asset version.
fn macos_font_assets() -> Vec<PathBuf> {
    let Ok(kinds) = std::fs::read_dir("/System/Library/AssetsV2") else { return Vec::new() };
    kinds
        .flatten()
        .filter(|kind| kind.file_name().to_string_lossy().starts_with("com_apple_MobileAsset_Font"))
        .filter_map(|kind| std::fs::read_dir(kind.path()).ok())
        .flat_map(|assets| assets.flatten().take(MAX_FONT_ASSETS))
        .map(|asset| asset.path().join("AssetData"))
        .collect()
}

/// Where an openly licensed Thai document face may be, best first: one the user installed, then
/// Sarabun, which macOS downloads as a collection. Each still has to declare its licence when read.
/// The fonts that ship with the system or with Word are left out on purpose: they are not open, and
/// all of them have the shaping tables Thonburi lacks.
fn installed_thai_candidates() -> Vec<PathBuf> {
    const DOCUMENT_FACES: &[&str] = &["Sarabun-Regular.ttf", "THSarabunNew.ttf", "NotoSansThai-Regular.ttf"];
    if cfg!(windows) {
        let root = std::env::var_os("WINDIR").or_else(|| std::env::var_os("SystemRoot"));
        let mut dirs = vec![root.map_or_else(|| PathBuf::from(r"C:\Windows"), PathBuf::from).join("Fonts")];
        dirs.extend(std::env::var_os("LOCALAPPDATA").map(|local| PathBuf::from(local).join("Microsoft/Windows/Fonts")));
        dirs.iter().flat_map(|dir| DOCUMENT_FACES.iter().map(move |f| dir.join(f))).collect()
    } else if cfg!(target_os = "macos") {
        let mut dirs = vec![PathBuf::from("/Library/Fonts")];
        dirs.extend(std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Library/Fonts")));
        let mut paths: Vec<PathBuf> = dirs.iter().flat_map(|dir| DOCUMENT_FACES.iter().map(move |f| dir.join(f))).collect();
        paths.extend(macos_font_assets().into_iter().map(|dir| dir.join("Sarabun.ttc")));
        paths
    } else {
        let mut dirs = vec![
            PathBuf::from("/usr/share/fonts/truetype/noto"),
            PathBuf::from("/usr/share/fonts/noto"),
            PathBuf::from("/usr/share/fonts/google-noto"),
            PathBuf::from("/usr/local/share/fonts"),
        ];
        dirs.extend(std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share/fonts")));
        dirs.iter().flat_map(|dir| DOCUMENT_FACES.iter().map(move |f| dir.join(f))).collect()
    }
}

/// The `Deva` craft-fonts faces for Devanagari interface text (the Hindi catalog, file names,
/// document titles), in manifest order. Empty when built without craft-fonts or when its revision
/// has no Devanagari face; a desktop build then falls back to one font already installed on the
/// machine (`SYSTEM_DEVANAGARI_FALLBACK`), and the web build shows replacement boxes.
pub fn ui_devanagari_fonts() -> Vec<&'static CraftFont> {
    CRAFT_FONTS.iter().filter(|f| f.covers("Deva")).collect()
}

/// The face for Devanagari text written into PDFs: the first `Deva` face. `None` when built
/// without craft-fonts or when its revision has no Devanagari face.
pub fn document_devanagari_font() -> Option<&'static CraftFont> {
    ui_devanagari_fonts().into_iter().next()
}

/// The `Hebr` craft-fonts faces for Hebrew interface text (file names, document titles, text
/// being edited), in manifest order. Empty when built without craft-fonts or when it has no
/// Hebrew face.
pub fn ui_hebrew_fonts() -> Vec<&'static CraftFont> {
    CRAFT_FONTS.iter().filter(|f| f.covers("Hebr")).collect()
}

/// The face for Hebrew text written into PDFs: the `Hebr` face of the asked weight, else its
/// Regular, else any `Hebr` face. `None` without craft-fonts or without a Hebrew face.
pub fn document_hebrew_font(bold: bool) -> Option<&'static CraftFont> {
    hebrew_face(CRAFT_FONTS, bold)
}

fn hebrew_face(faces: &[CraftFont], bold: bool) -> Option<&CraftFont> {
    let hebr = || faces.iter().filter(|f| f.covers("Hebr"));
    let style = if bold { "Bold" } else { "Regular" };
    hebr().find(|f| f.style == style).or_else(|| hebr().find(|f| f.style == "Regular")).or_else(|| hebr().next())
}

/// Interface CJK faces in fallback order for the UI language: Simplified Chinese first when
/// `prefer_hans`, otherwise Japanese first (the historical default).
///
/// The order matters beyond glyph shapes. egui renders each character with the FIRST face that
/// has its glyph, so with Japanese first a Simplified-only character (e.g. U+6B22 欢, absent
/// from Japanese faces) lands in a different face than its neighbours; the mixed vertical
/// metrics then sink it below the line. Preferring the UI language's face keeps one line in
/// one face with one baseline.
pub fn ui_cjk_fonts(prefer_hans: bool) -> Vec<&'static CraftFont> {
    order_cjk(CRAFT_FONTS.iter(), prefer_hans)
}

fn order_cjk<'a>(faces: impl IntoIterator<Item = &'a CraftFont>, prefer_hans: bool) -> Vec<&'a CraftFont> {
    let mut out: Vec<&'a CraftFont> = faces.into_iter().filter(|f| f.covers("Hans") || f.covers("Jpan")).collect();
    let first = if prefer_hans { "Hans" } else { "Jpan" };
    // The preferred script's faces first (a face covering both counts as preferred); within each
    // group the Japanese UI face (BIZ UDPGothic, not a serif) leads, otherwise manifest order
    // (the sort is stable).
    out.sort_by_key(|f| (!f.covers(first), f.family != "BIZ UDPGothic"));
    out
}

/// The face for Japanese text written into PDFs (serif document text): Shippori Mincho, then
/// BIZ UDMincho, then any other regular `Jpan` face. `None` without craft-fonts.
pub fn document_japanese_font() -> Option<&'static CraftFont> {
    document_face(CRAFT_FONTS, true, false)
}

/// A real Japanese document face matching serif/sans and weight where available.
/// Sans text prefers BIZ UDPGothic Bold or Regular. Missing weights fall back to Regular;
/// serif text keeps the document Mincho preference. No synthetic weight or slant is applied.
/// `None` without craft-fonts. The small web input currently has only Gothic Regular.
pub fn document_japanese_font_for_style(serif: bool, bold: bool) -> Option<&'static CraftFont> {
    document_face(CRAFT_FONTS, serif, bold)
}

/// Every `Jpan` face that can stand in for the document text, the best match for the style first
/// (the face [`document_japanese_font_for_style`] returns), then the others. Not every face has
/// every glyph (their coverage of e.g. Cyrillic differs), so a caller can move on to the next one.
/// Empty without craft-fonts.
pub fn document_japanese_fonts_for_style(serif: bool, bold: bool) -> Vec<&'static CraftFont> {
    document_faces(CRAFT_FONTS, serif, bold)
}

/// The scripts the bundled Inter faces are for.
static LATIN_SCRIPTS: &[&str] = &["Latn", "Cyrl", "Grek"];

/// The bundled Latin document faces (`assets/fonts`, OFL): every build has them, unlike
/// craft-fonts. The Japanese faces carry only part of the Cyrillic (`привет` yes, Uzbek `қизил`
/// and Kazakh `әже` no), so letters like қ, ғ, ҳ, ә, ө, ү, ұ and һ have no face at all without
/// these.
static INTER_REGULAR_FACE: CraftFont =
    CraftFont { family: "Inter", style: "Regular", scripts: LATIN_SCRIPTS, bytes: include_bytes!("../../../assets/fonts/Inter-Regular.ttf") };
static INTER_SEMIBOLD_FACE: CraftFont =
    CraftFont { family: "Inter", style: "SemiBold", scripts: LATIN_SCRIPTS, bytes: include_bytes!("../../../assets/fonts/Inter-SemiBold.ttf") };

/// The face for Latin, Greek and Cyrillic text written into PDFs: the bundled Inter Regular, or
/// SemiBold for bold text. Present in every build, crafted ones included.
pub fn document_latin_fonts(bold: bool) -> Vec<&'static CraftFont> {
    vec![if bold { &INTER_SEMIBOLD_FACE } else { &INTER_REGULAR_FACE }]
}

fn document_face(faces: &[CraftFont], serif: bool, bold: bool) -> Option<&CraftFont> {
    document_faces(faces, serif, bold).into_iter().next()
}

fn document_faces(faces: &[CraftFont], serif: bool, bold: bool) -> Vec<&CraftFont> {
    let jpan = || faces.iter().filter(|f| f.covers("Jpan"));
    let mut preferred = Vec::new();
    if !serif {
        let style = if bold { "Bold" } else { "Regular" };
        preferred.extend(jpan().find(|f| f.family == "BIZ UDPGothic" && f.style == style));
        preferred.extend(jpan().find(|f| f.family == "BIZ UDPGothic" && f.style == "Regular"));
    }
    for family in ["Shippori Mincho", "BIZ UDMincho"] {
        preferred.extend(jpan().find(|f| f.family == family && f.style == "Regular"));
    }
    let mut out: Vec<&CraftFont> = Vec::new();
    for face in preferred.into_iter().chain(jpan().filter(|f| f.style == "Regular")).chain(jpan()) {
        if !out.iter().any(|f| std::ptr::eq(*f, face)) {
            out.push(face);
        }
    }
    out
}

const fn str_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

const fn find(family: &str, style: &str) -> Option<&'static [u8]> {
    let mut i = 0;
    while i < CRAFT_FONTS.len() {
        let f = &CRAFT_FONTS[i];
        if str_eq(f.family, family) && str_eq(f.style, style) {
            return Some(f.bytes);
        }
        i += 1;
    }
    None
}

/// Shippori Mincho Regular from craft-fonts, the preferred face for Japanese document text.
/// `None` when PdfCraft was built without craft-fonts.
pub static SHIPPORI_MINCHO: Option<&[u8]> = find("Shippori Mincho", "Regular");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cjk_fallback_order_follows_the_ui_language() {
        // Synthetic faces: order_cjk must not depend on the real build input.
        static BYTES: &[u8] = b"fake";
        static LATN: &[&str] = &["Latn"];
        static JPAN_LATN: &[&str] = &["Jpan", "Latn"];
        static HANS_LATN: &[&str] = &["Hans", "Latn"];
        let biz_bold = CraftFont { family: "BIZ UDPGothic", style: "Bold", scripts: JPAN_LATN, bytes: BYTES };
        let biz = CraftFont { family: "BIZ UDPGothic", style: "Regular", scripts: JPAN_LATN, bytes: BYTES };
        let hans = CraftFont { family: "FakeHans", style: "Regular", scripts: HANS_LATN, bytes: BYTES };
        let inter = CraftFont { family: "Inter", style: "Regular", scripts: LATN, bytes: BYTES };
        let faces = [biz_bold, biz, hans, inter];
        // Chinese mode: the Hans group first (manifest order), then Japanese.
        let zh: Vec<&str> = order_cjk([&faces[0], &faces[2], &faces[1]], true).iter().map(|f| f.family).collect();
        assert_eq!(zh, ["FakeHans", "BIZ UDPGothic", "BIZ UDPGothic"]);
        // Japanese mode keeps the historical default: the Japanese UI face first.
        let ja: Vec<&str> = order_cjk([&faces[0], &faces[2], &faces[1]], false).iter().map(|f| f.family).collect();
        assert_eq!(ja, ["BIZ UDPGothic", "BIZ UDPGothic", "FakeHans"]);
        // Latin-only faces are never interface CJK fallbacks.
        assert!(order_cjk(&faces, true).iter().all(|f| f.covers("Hans") || f.covers("Jpan")));
        // A serif listed first in the manifest never leads the Japanese group, in either mode.
        let mincho = CraftFont { family: "Shippori Mincho", style: "Regular", scripts: JPAN_LATN, bytes: BYTES };
        let zh: Vec<&str> = order_cjk([&mincho, &faces[2], &faces[1]], true).iter().map(|f| f.family).collect();
        assert_eq!(zh, ["FakeHans", "BIZ UDPGothic", "Shippori Mincho"]);
        let ja: Vec<&str> = order_cjk([&mincho, &faces[2], &faces[1]], false).iter().map(|f| f.family).collect();
        assert_eq!(ja, ["BIZ UDPGothic", "Shippori Mincho", "FakeHans"]);
    }

    #[test]
    fn document_faces_match_style_without_inventing_missing_weights() {
        let faces = [
            CraftFont { family: "Shippori Mincho", style: "Regular", scripts: &["Jpan"], bytes: b"serif" },
            CraftFont { family: "BIZ UDPGothic", style: "Regular", scripts: &["Jpan"], bytes: b"sans" },
            CraftFont { family: "BIZ UDPGothic", style: "Bold", scripts: &["Jpan"], bytes: b"bold" },
            CraftFont { family: "BIZ UDPGothic", style: "Bold", scripts: &["Latn"], bytes: b"not-japanese" },
        ];
        assert_eq!(document_face(&faces, false, false).unwrap().bytes, b"sans");
        assert_eq!(document_face(&faces, false, true).unwrap().bytes, b"bold");
        assert_eq!(document_face(&faces, true, false).unwrap().bytes, b"serif");
        assert_eq!(document_face(&faces, true, true).unwrap().bytes, b"serif");
        assert_eq!(document_face(&faces[..2], false, true).unwrap().bytes, b"sans");
        assert_eq!(document_face(&faces[..1], false, true).unwrap().bytes, b"serif");
        // Every Japanese face is a candidate, the style's match first and none twice.
        let order: Vec<&[u8]> = document_faces(&faces, false, true).iter().map(|f| f.bytes).collect();
        assert_eq!(order, [b"bold".as_slice(), b"sans", b"serif"]);
        let order: Vec<&[u8]> = document_faces(&faces, true, false).iter().map(|f| f.bytes).collect();
        assert_eq!(order, [b"serif".as_slice(), b"sans", b"bold"]);
        assert!(document_face(&faces[3..], false, true).is_none());
        assert!(document_face(&[], false, true).is_none());
    }

    #[test]
    fn arabic_faces_are_picked_by_script_in_manifest_order() {
        // Synthetic faces: the filter must not depend on the real build input.
        static BYTES: &[u8] = b"fake";
        static ARAB_LATN: &[&str] = &["Arab", "Latn"];
        static JPAN: &[&str] = &["Jpan"];
        let naskh = CraftFont { family: "FakeNaskh", style: "Regular", scripts: ARAB_LATN, bytes: BYTES };
        let biz = CraftFont { family: "BIZ UDPGothic", style: "Regular", scripts: JPAN, bytes: BYTES };
        let kufi = CraftFont { family: "FakeKufi", style: "Regular", scripts: ARAB_LATN, bytes: BYTES };
        let faces = [naskh, biz, kufi];
        let ar: Vec<&str> = arabic(&faces).iter().map(|f| f.family).collect();
        assert_eq!(ar, ["FakeNaskh", "FakeKufi"]);
        // An Arabic face is never a CJK fallback, and the other way round.
        assert!(order_cjk(&faces, false).iter().all(|f| f.family == "BIZ UDPGothic"));
        assert_eq!(ui_arabic_fonts().len(), CRAFT_FONTS.iter().filter(|f| f.covers("Arab")).count());
    }

    #[test]
    fn hebrew_document_faces_follow_the_weight() {
        static HEBR_LATN: &[&str] = &["Hebr", "Latn"];
        let faces = [
            CraftFont { family: "BIZ UDPGothic", style: "Bold", scripts: &["Jpan"], bytes: b"not-hebrew" },
            CraftFont { family: "FakeHebrew", style: "Regular", scripts: HEBR_LATN, bytes: b"regular" },
            CraftFont { family: "FakeHebrew", style: "Bold", scripts: HEBR_LATN, bytes: b"bold" },
        ];
        assert_eq!(hebrew_face(&faces, false).map(|f| f.bytes), Some(&b"regular"[..]));
        assert_eq!(hebrew_face(&faces, true).map(|f| f.bytes), Some(&b"bold"[..]));
        // No bold face: Regular, never a face of another script.
        assert_eq!(hebrew_face(&faces[..2], true).map(|f| f.bytes), Some(&b"regular"[..]));
        assert!(hebrew_face(&faces[..1], false).is_none());
        assert_eq!(ui_hebrew_fonts().len(), CRAFT_FONTS.iter().filter(|f| f.covers("Hebr")).count());
        assert_eq!(document_hebrew_font(false).is_some(), CRAFT_FONTS.iter().any(|f| f.covers("Hebr")));
    }

    /// `build.rs` embeds the Japanese CJK faces whole and any other CJK face only up to
    /// `MAX_OTHER_CJK_BYTES` (the explicitly pinned native-only LXGW UI fallback has a separate
    /// 16 MiB cap), so a checkout with a large Chinese or Korean face (craft-fonts has the
    /// 16.4 MB Noto Sans CJK SC) leaves it out. The four Japanese faces of the pinned craft-fonts
    /// are 24.1 MB.
    #[test]
    fn cjk_faces_follow_the_size_rule() {
        use crate::cjk_scripts::{MAX_NATIVE_UI_CJK_BYTES, MAX_OTHER_CJK_BYTES, NATIVE_UI_CJK_FAMILIES, WHOLE_CJK_FAMILIES, is_cjk_script};
        // A tag can carry a region after the code: craft-fonts tags Noto Sans HK UI `Hant-HK`.
        assert!(["Hant-HK", "Hans", "Jpan", "Hang"].iter().all(|s| is_cjk_script(s)));
        assert!(!["Latn", "Arab", "Hebr"].iter().any(|s| is_cjk_script(s)));
        let cjk = CRAFT_FONTS.iter().filter(|f| f.scripts.iter().any(|s| is_cjk_script(s)));
        let (whole, other): (Vec<_>, Vec<_>) = cjk.partition(|f| WHOLE_CJK_FAMILIES.contains(&f.family));
        for face in &other {
            let len = face.bytes.len();
            let native_ui = NATIVE_UI_CJK_FAMILIES.contains(&face.family);
            assert!(!cfg!(target_arch = "wasm32") || !native_ui, "native UI face in web payload");
            let cap = if native_ui { MAX_NATIVE_UI_CJK_BYTES } else { MAX_OTHER_CJK_BYTES };
            assert!(u64::try_from(len).is_ok_and(|n| n <= cap), "{} is embedded at {len} bytes", face.name());
        }
        let bytes: usize = whole.iter().map(|f| f.bytes.len()).sum();
        assert!(bytes < 25_000_000, "{bytes} bytes of Japanese faces");
    }

    /// The bundled Inter faces carry the Latin, Greek and Cyrillic letters the craft-fonts faces
    /// lack: Uzbek қ ғ ҳ, Kazakh ә ө ү ұ һ, Ukrainian ї є ґ, Greek. This one runs with or without
    /// CRAFT_FONTS_DIR (#2593).
    #[test]
    fn latin_faces_carry_the_letters_the_japanese_faces_lack() {
        for face in document_latin_fonts(false).into_iter().chain(document_latin_fonts(true)) {
            for ch in ['қ', 'ғ', 'ҳ', 'ә', 'ө', 'ү', 'ұ', 'һ', 'ї', 'є', 'ґ', 'ё', 'α', 'β', 'γ', '≥', '≈'] {
                assert!(crate::japanese_glyph_from(face, ch).is_ok(), "{} has no {ch}", face.name());
            }
        }
        assert_eq!(document_latin_fonts(false)[0].style, "Regular");
        assert_eq!(document_latin_fonts(true)[0].style, "SemiBold");
    }

    #[test]
    fn craft_fonts_are_optional_and_consistent() {
        // Holds with or without CRAFT_FONTS_DIR.
        assert_eq!(SHIPPORI_MINCHO.is_some(), CRAFT_FONTS.iter().any(|f| f.family == "Shippori Mincho" && f.style == "Regular"));
        assert_eq!(document_japanese_font().is_some(), CRAFT_FONTS.iter().any(|f| f.covers("Jpan")));
        let ui = ui_japanese_fonts();
        assert_eq!(ui.len(), CRAFT_FONTS.iter().filter(|f| f.covers("Jpan")).count());
        let ui_zh = ui_chinese_fonts();
        assert_eq!(ui_zh.len(), CRAFT_FONTS.iter().filter(|f| f.covers("Hans")).count());
        if CRAFT_FONTS.is_empty() {
            eprintln!("built without craft-fonts (CRAFT_FONTS_DIR unset): no Japanese faces, as expected");
            assert!(SHIPPORI_MINCHO.is_none() && ui.is_empty() && ui_zh.is_empty());
            return;
        }
        assert!(CRAFT_FONTS.iter().all(|f| !f.bytes.is_empty()));
        if CRAFT_FONTS.iter().any(|f| f.family == "BIZ UDPGothic") {
            assert_eq!(ui.first().map(|f| f.family), Some("BIZ UDPGothic"));
        }
        assert_eq!(document_japanese_font().map(|f| f.family), Some("Shippori Mincho"));
    }

    #[test]
    fn only_a_readable_face_with_thai_is_an_installed_thai_font() {
        let inter = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/fonts/Inter-Regular.ttf"));
        assert!(read_thai_face(inter).is_none(), "Inter has no Thai");
        assert!(read_thai_face(Path::new("definitely/not/here.ttf")).is_none());
        // A directory is not a font.
        assert!(read_thai_face(&std::env::temp_dir()).is_none());
    }

    #[test]
    fn installed_thai_candidates_are_absolute_font_files() {
        let list = installed_thai_candidates();
        assert!(!list.is_empty());
        assert!(list.iter().all(|p| p.is_absolute() || cfg!(windows)), "{list:?}");
        assert!(list.iter().all(|p| p.extension().is_some_and(|e| e == "ttf" || e == "ttc")), "{list:?}");
    }

    /// `font` as a one-face collection: a header and the table offsets moved past it.
    fn one_face_collection(font: &[u8]) -> Vec<u8> {
        const SHIFT: u32 = 16;
        let tables = usize::from(u16::from_be_bytes([font[4], font[5]]));
        let mut face = font.to_vec();
        for table in 0..tables {
            let at = 12 + table * 16 + 8;
            let offset = u32::from_be_bytes(face[at..at + 4].try_into().unwrap());
            face[at..at + 4].copy_from_slice(&(offset + SHIFT).to_be_bytes());
        }
        let mut collection = b"ttcf\x00\x01\x00\x00\x00\x00\x00\x01".to_vec();
        collection.extend_from_slice(&SHIFT.to_be_bytes());
        collection.extend_from_slice(&face);
        collection
    }

    #[test]
    fn a_face_comes_out_of_a_collection_whole() {
        use skrifa::MetadataProvider as _;
        let inter = crate::INTER_REGULAR;
        let collection = one_face_collection(inter);
        assert!(skrifa::FontRef::new(&collection).is_err(), "a collection is not one font");
        let face = standalone_face(&collection, 0).expect("the only face");
        let font = skrifa::FontRef::new(&face).expect("a font of its own");
        let source = skrifa::FontRef::new(inter).unwrap();
        assert_eq!(font.charmap().map('A'), source.charmap().map('A'));
        assert_eq!(crate::embedding::names(&face).map(|(family, _)| family), crate::embedding::names(inter).map(|(family, _)| family));
    }

    #[test]
    fn a_broken_collection_gives_no_face() {
        let collection = one_face_collection(crate::INTER_REGULAR);
        assert!(standalone_face(&collection, 1).is_none(), "no second face");
        assert!(standalone_face(&collection[..64], 0).is_none(), "tables cut off");
        assert!(standalone_face(b"ttcf", 0).is_none());
        assert!(standalone_face(b"", 0).is_none());
        assert!(regular_face(b"ttcf").is_none());
    }

    /// An installed face goes into saved PDFs, so it has to declare an open licence. Its embedding
    /// bits are not enough: they say what the font allows, not on what terms it was licensed.
    #[test]
    fn an_installed_thai_font_declares_an_open_licence() {
        if let Some(face) = installed_thai_font() {
            assert!(crate::embedding::validate_document_input(face.bytes).is_ok(), "{}", face.family);
            assert!(crate::embedding::covers(face.bytes, "น้ำ กิ่ ผู้ใช้ ๑๒๓ Hello"));
            assert!(face.covers("Thai"));
        }
    }

    /// Tahoma and the Thai fonts that come with Word allow embedding by their bits, but belong to
    /// Microsoft and are licensed for their own products. They may draw on screen, never go into a PDF.
    #[cfg(target_os = "macos")]
    #[test]
    fn a_mac_never_writes_thai_in_a_proprietary_font() {
        for path in ["/System/Library/Fonts/Supplemental/Tahoma.ttf", "/Applications/Microsoft Word.app/Contents/Resources/DFonts/thsarabun.ttf"] {
            if Path::new(path).is_file() {
                assert!(read_thai_face(Path::new(path)).is_none(), "{path}");
            }
        }
    }

    /// macOS downloads Sarabun as a collection. Its Regular face has to come out, not a lighter
    /// one that also calls itself Regular.
    #[cfg(target_os = "macos")]
    #[test]
    fn a_mac_with_sarabun_writes_thai_in_its_regular_face() {
        let Some(path) = macos_font_assets().into_iter().map(|dir| dir.join("Sarabun.ttc")).find(|path| path.is_file()) else {
            eprintln!("Sarabun has not been downloaded here: skipped");
            return;
        };
        let face = read_thai_face(&path).expect("the collection's Regular face");
        assert_eq!(face.family, "Sarabun");
        assert!(crate::embedding::validate(face.bytes).is_ok());
        assert!(crate::embedding::covers(face.bytes, "น้ำ กิ่ ผู้ใช้ ๑๒๓ Hello"));
    }
}
