//! Unicode search text has no visible glyphs. A composite font with one blank TrueType glyph
//! supplies advance widths and a ToUnicode map, so no display font is needed. It is embedded
//! rather than left out because PDF readers do not agree on anything less: Apple's PDFKit found
//! no text at all in the empty Type 3 font upstream writes.

use std::collections::BTreeMap;

use pdfcraft_cos::{Dict, Document, Object, PdfString, Stream};
use pdfcraft_edit::EditError;
use pdfcraft_ocr::PlacedWord;

/// Refuse oversized text or invalid geometry before cloning words or allocating fonts.
/// These limits apply to the complete input, including all editable export pages.
pub(crate) fn validate_words<'a>(words: impl IntoIterator<Item = &'a PlacedWord>) -> Result<usize, EditError> {
    let mut count = 0usize;
    let mut length = 0usize;
    for w in words {
        count += 1;
        length = length.checked_add(w.text.len()).ok_or_else(|| EditError::Invalid("OCR text layer is too large".into()))?;
        if count > 65536 || length > 4 * 1024 * 1024 {
            return Err(EditError::Invalid("OCR text layer is too large".into()));
        }
        if !w.origin.iter().chain(&w.across).chain(&w.up).all(|v| v.is_finite())
            || !w.across[0].hypot(w.across[1]).is_finite()
            || !w.up[0].hypot(w.up[1]).is_finite()
            || (0..2).any(|i| !(w.origin[i] + 0.2 * w.up[i]).is_finite() || !(w.origin[i] + w.across[i] + 0.2 * w.up[i]).is_finite())
        {
            return Err(EditError::Invalid("OCR text coordinates must be finite".into()));
        }
    }
    Ok(count)
}

pub(crate) fn add_text(doc: &mut Document, page: usize, words: &[PlacedWord]) -> Result<(), EditError> {
    validate_words(words)?;
    // Keep the existing Latin layout, including Helvetica's proportional advance widths.
    if words.iter().all(|w| {
        w.text
            .chars()
            .all(|c| matches!(c, '\u{20}'..='\u{7e}' | '\u{a0}'..='\u{ff}' | '€' | '‚' | '„' | '…' | '‘' | '’' | '“' | '”' | '•' | '–' | '—' | '™'))
    }) {
        pdfcraft_edit::stamp(doc, page, "OCR", pdfcraft_ocr::text_layer(words, &pdfcraft_fonts::TextPlan::default()))?;
        return Ok(());
    }
    let p = pdfcraft_model::pages(doc).get(page).cloned().ok_or_else(|| EditError::Invalid("OCR page does not exist".into()))?;
    let mut res = p.dict.get(b"Resources").map(|r| doc.resolve(r)).and_then(|r| r.as_dict().cloned()).unwrap_or_default();
    let mut fonts = res.get(b"Font").map(|f| doc.resolve(f)).and_then(|f| f.as_dict().cloned()).unwrap_or_default();
    let chars: Vec<char> = words.iter().flat_map(|w| w.text.chars()).collect::<std::collections::BTreeSet<_>>().into_iter().collect();
    if chars.len() >= 0xFFFF {
        return Err(EditError::Invalid("OCR text layer has too many distinct characters".into()));
    }
    // CID 0 is .notdef, so the first character is CID 1.
    let codes: BTreeMap<char, u16> = chars.iter().enumerate().map(|(i, c)| (*c, i as u16 + 1)).collect();
    let mut name = String::from("PCOcr");
    while fonts.contains(name.as_bytes()) {
        name.push('x');
    }
    let font = invisible_font(doc, &chars);
    fonts.set(name.as_bytes().to_vec(), Object::Ref(font));
    res.set(b"Font".to_vec(), Object::Dict(fonts));
    doc.update_dict(p.obj, |d| d.set(b"Resources".to_vec(), Object::Dict(res)))?;
    // `3 Tr` and the font are graphics state. `ET` does not reset them, and a page with no earlier
    // content is not wrapped, so without `q … Q` text added afterwards draws invisibly (#3029).
    let mut content = format!("/OCR BMC\nq\nBT\n3 Tr\n/{name} 1 Tf\n");
    for w in words {
        let hex: String = w.text.chars().filter_map(|c| codes.get(&c)).map(|code| format!("{code:04X}")).collect();
        let count = hex.len() / 4;
        if count == 0 || w.up[0].hypot(w.up[1]) <= 0.0 || w.across[0].hypot(w.across[1]) <= 0.0 {
            continue;
        }
        // Every character is one em wide: the matrix stretches the run across the word's box. The
        // font's em box is -200..800, so the baseline sits 0.2 em up and the box spans the word.
        let n = count as f64;
        content.push_str(&format!(
            "{} {} {} {} {} {} Tm <{hex}> Tj\n",
            w.across[0] / n,
            w.across[1] / n,
            w.up[0],
            w.up[1],
            w.origin[0] + 0.2 * w.up[0],
            w.origin[1] + 0.2 * w.up[1]
        ));
    }
    content.push_str("ET\nQ\nEMC\n");
    pdfcraft_edit::stamp(doc, page, "OCR", content.into_bytes()).map(|_| ())
}

/// A Type 0 font whose CID `i + 1` is `chars[i]`, every one drawn with the same blank glyph.
fn invisible_font(doc: &mut Document, chars: &[char]) -> pdfcraft_cos::ObjRef {
    let mut cmap = String::from(
        "/CIDInit /ProcSet findresource begin\n12 dict begin\nbegincmap\n/CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n/CMapName /Adobe-Identity-UCS def\n/CMapType 2 def\n1 begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n",
    );
    for (batch, part) in chars.chunks(100).enumerate() {
        cmap.push_str(&format!("{} beginbfchar\n", part.len()));
        for (i, ch) in part.iter().enumerate() {
            let mut units = [0u16; 2];
            let hex: String = ch.encode_utf16(&mut units).iter().map(|u| format!("{u:04X}")).collect();
            cmap.push_str(&format!("<{:04X}> <{hex}>\n", batch * 100 + i + 1));
        }
        cmap.push_str("endbfchar\n");
    }
    cmap.push_str("endcmap\nCMapName currentdict /CMap defineresource pop\nend\nend\n");
    let to_unicode = doc.add(Object::Stream(Stream::flate(Dict::new(), cmap.as_bytes())));

    let program = blank_truetype();
    let mut file = Dict::new();
    file.set(b"Length1".to_vec(), Object::Int(program.len() as i64));
    let file = doc.add(Object::Stream(Stream::flate(file, &program)));

    let mut descriptor = Dict::new();
    descriptor.set(b"Type".to_vec(), Object::name("FontDescriptor"));
    descriptor.set(b"FontName".to_vec(), Object::name("PdfCraftInvisible"));
    descriptor.set(b"Flags".to_vec(), Object::Int(4));
    descriptor.set(b"FontBBox".to_vec(), Object::Array(vec![0.into(), (-200).into(), 1000.into(), 800.into()]));
    descriptor.set(b"ItalicAngle".to_vec(), Object::Int(0));
    descriptor.set(b"Ascent".to_vec(), Object::Int(800));
    descriptor.set(b"Descent".to_vec(), Object::Int(-200));
    descriptor.set(b"CapHeight".to_vec(), Object::Int(800));
    descriptor.set(b"StemV".to_vec(), Object::Int(0));
    descriptor.set(b"FontFile2".to_vec(), Object::Ref(file));
    let descriptor = doc.add(Object::Dict(descriptor));

    // Every CID but .notdef draws glyph 1, the blank one.
    let mut gids = vec![0, 0];
    gids.extend(std::iter::repeat_n([0u8, 1], chars.len()).flatten());
    let gids = doc.add(Object::Stream(Stream::flate(Dict::new(), &gids)));

    let mut system = Dict::new();
    system.set(b"Registry".to_vec(), Object::String(PdfString::literal("Adobe")));
    system.set(b"Ordering".to_vec(), Object::String(PdfString::literal("Identity")));
    system.set(b"Supplement".to_vec(), Object::Int(0));
    let mut descendant = Dict::new();
    descendant.set(b"Type".to_vec(), Object::name("Font"));
    descendant.set(b"Subtype".to_vec(), Object::name("CIDFontType2"));
    descendant.set(b"BaseFont".to_vec(), Object::name("PdfCraftInvisible"));
    descendant.set(b"CIDSystemInfo".to_vec(), Object::Dict(system));
    descendant.set(b"FontDescriptor".to_vec(), Object::Ref(descriptor));
    descendant.set(b"DW".to_vec(), Object::Int(1000));
    descendant.set(b"CIDToGIDMap".to_vec(), Object::Ref(gids));
    let descendant = doc.add(Object::Dict(descendant));

    let mut font = Dict::new();
    font.set(b"Type".to_vec(), Object::name("Font"));
    font.set(b"Subtype".to_vec(), Object::name("Type0"));
    font.set(b"BaseFont".to_vec(), Object::name("PdfCraftInvisible"));
    font.set(b"Encoding".to_vec(), Object::name("Identity-H"));
    font.set(b"DescendantFonts".to_vec(), Object::Array(vec![Object::Ref(descendant)]));
    font.set(b"ToUnicode".to_vec(), Object::Ref(to_unicode));
    doc.add(Object::Dict(font))
}

/// A TrueType font of two empty glyphs, one em wide, with the em box -200..800.
fn blank_truetype() -> Vec<u8> {
    let be16 = |v: &[i32]| v.iter().flat_map(|n| (*n as i16).to_be_bytes()).collect::<Vec<u8>>();
    let mut head = vec![0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0x5F, 0x0F, 0x3C, 0xF5, 0, 0, 0x03, 0xE8];
    head.extend([0; 16]); // created and modified
    head.extend(be16(&[0, -200, 1000, 800, 0, 8, 2, 0, 0]));
    let mut hhea = vec![0, 1, 0, 0];
    hhea.extend(be16(&[800, -200, 0, 1000, 0, 0, 1000, 1, 0, 0, 0, 0, 0, 0, 0, 1]));
    let hmtx = be16(&[1000, 0, 0]);
    let loca = be16(&[0, 0, 0]);
    let mut maxp = vec![0, 1, 0, 0];
    maxp.extend(be16(&[2, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0]));
    let glyf = vec![0; 4];
    let tables: [(&[u8; 4], &[u8]); 6] = [(b"glyf", &glyf), (b"head", &head), (b"hhea", &hhea), (b"hmtx", &hmtx), (b"loca", &loca), (b"maxp", &maxp)];

    let sum =
        |bytes: &[u8]| bytes.chunks(4).fold(0u32, |s, c| s.wrapping_add(c.iter().fold(0u32, |w, b| (w << 8) | u32::from(*b)) << (8 * (4 - c.len()))));
    let mut out = vec![0, 1, 0, 0, 0, 6, 0, 64, 0, 2, 0, 32];
    let mut offset = 12 + 16 * tables.len();
    let mut body = Vec::new();
    for (tag, data) in tables {
        out.extend_from_slice(tag);
        out.extend(sum(data).to_be_bytes());
        out.extend((offset as u32).to_be_bytes());
        out.extend((data.len() as u32).to_be_bytes());
        body.extend_from_slice(data);
        body.resize(body.len().next_multiple_of(4), 0);
        offset = 12 + 16 * tables.len() + body.len();
    }
    out.extend(body);
    // head.checkSumAdjustment: the whole file then sums to the format's magic number.
    let adjust = 0xB1B0_AFBAu32.wrapping_sub(sum(&out));
    let head_at = 12 + 16 * tables.len() + glyf.len();
    out[head_at + 8..head_at + 12].copy_from_slice(&adjust.to_be_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The program is a well-formed sfnt: six sorted tables whose offsets and checksums add up,
    /// which is what a strict reader checks before it trusts the font.
    #[test]
    fn the_blank_font_is_a_well_formed_sfnt() {
        let f = blank_truetype();
        assert_eq!(&f[..4], &[0, 1, 0, 0]);
        let count = u16::from_be_bytes([f[4], f[5]]) as usize;
        assert_eq!(count, 6);
        let sum = |bytes: &[u8]| {
            bytes.chunks(4).fold(0u32, |s, c| s.wrapping_add(c.iter().fold(0u32, |w, b| (w << 8) | u32::from(*b)) << (8 * (4 - c.len()))))
        };
        assert_eq!(sum(&f), 0xB1B0_AFBA, "head.checkSumAdjustment");
        let mut last = [0u8; 4];
        for t in f[12..12 + 16 * count].chunks(16) {
            let tag: [u8; 4] = t[..4].try_into().unwrap();
            assert!(tag > last, "tables are sorted by tag");
            last = tag;
            let offset = u32::from_be_bytes(t[8..12].try_into().unwrap()) as usize;
            let length = u32::from_be_bytes(t[12..16].try_into().unwrap()) as usize;
            assert!(offset.is_multiple_of(4) && offset + length <= f.len(), "{tag:?}");
        }
        let head = 12 + 16 * count + 4;
        assert_eq!(u16::from_be_bytes([f[head + 18], f[head + 19]]), 1000, "units per em");
    }
}
