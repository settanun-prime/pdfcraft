//! The PP-OCR engine: Thai, Chinese and accented Latin. ocrs's detector finds the lines, and a
//! PP-OCR recognition model with its character dictionary reads them ([`crate::ppocr`]). The
//! models are fetched by `cargo xtask models` (see ATTRIBUTION.toml) and found where the ocrs
//! models are, or compiled into the program (`PDFCRAFT_EMBED_MODELS`, see build.rs).

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rten_imageproc::BoundingRect;

use super::{DETECTION_MODEL, Line, Models, OcrError, OcrImage, RecognizeOptions, Recognizer, Word};
use crate::ppocr::PpOcr;

pub const CHINESE_MODEL: &str = "chinese-recognition.onnx";
pub const CHINESE_DICTIONARY: &str = "chinese-keys.txt";
pub const LATIN_MODEL: &str = "latin-recognition.onnx";
pub const LATIN_DICTIONARY: &str = "latin-keys.txt";
pub const THAI_MODEL: &str = "thai-recognition.onnx";
pub const THAI_DICTIONARY: &str = "thai-keys.txt";
/// Thai reads best with a margin around the line for its marks, and artwork on a certificate reads
/// as scattered letters that score far below real text (measured on a scanned certificate: text
/// 0.82 to 0.97, decorations mostly under 0.6).
const THAI_PADDING: f32 = 0.15;
const THAI_MIN_SCORE: f32 = 0.6;

/// The PP-OCR languages: (code, model file, dictionary file, output classes = CTC blank +
/// one per dictionary line + the appended space). The class count is pinned per model, so a
/// wrong or truncated dictionary fails at load instead of decoding garbage.
const PP_OCR: &[(&str, &str, &str, usize)] = &[
    ("th", THAI_MODEL, THAI_DICTIONARY, 526),
    ("zh", CHINESE_MODEL, CHINESE_DICTIONARY, 6625),
    ("la", LATIN_MODEL, LATIN_DICTIONARY, 504),
    ("fr", LATIN_MODEL, LATIN_DICTIONARY, 504),
    ("de", LATIN_MODEL, LATIN_DICTIONARY, 504),
    ("es", LATIN_MODEL, LATIN_DICTIONARY, 504),
    ("pt", LATIN_MODEL, LATIN_DICTIONARY, 504),
    ("it", LATIN_MODEL, LATIN_DICTIONARY, 504),
    ("nl", LATIN_MODEL, LATIN_DICTIONARY, 504),
    ("da", LATIN_MODEL, LATIN_DICTIONARY, 504),
    ("no", LATIN_MODEL, LATIN_DICTIONARY, 504),
    ("sv", LATIN_MODEL, LATIN_DICTIONARY, 504),
    ("is", LATIN_MODEL, LATIN_DICTIONARY, 504),
    ("fi", LATIN_MODEL, LATIN_DICTIONARY, 504),
    ("cs", LATIN_MODEL, LATIN_DICTIONARY, 504),
    ("pl", LATIN_MODEL, LATIN_DICTIONARY, 504),
];

/// The languages PP-OCR reads, after English (ocrs): what the language menus list.
pub const PP_LANGUAGES: &[(&str, &str)] = &[
    ("th", "Thai and English"),
    ("zh", "Chinese (Simplified) and English"),
    ("la", "Latin (accented)"),
    ("fr", "French"),
    ("de", "German"),
    ("es", "Spanish"),
    ("pt", "Portuguese"),
    ("it", "Italian"),
    ("nl", "Dutch"),
    ("da", "Danish"),
    ("no", "Norwegian"),
    ("sv", "Swedish"),
    ("is", "Icelandic"),
    ("fi", "Finnish"),
    ("cs", "Czech"),
    ("pl", "Polish"),
];

/// Whether PP-OCR reads `language`.
pub fn is_pp_language(language: &str) -> bool {
    PP_OCR.iter().any(|l| l.0 == language)
}

/// Letters a document language prefers when the recogniser is unsure. Empty means no preference.
pub fn preferred_letters(language: &str) -> &'static str {
    match language {
        "fr" => "àâæçéèêëîïôœùûüÿÀÂÆÇÉÈÊËÎÏÔŒÙÛÜŸ",
        // ß is lowercase-only in most German text, but the 2017 reform gave it the capital ẞ.
        "de" => "äöüßÄÖÜẞ",
        "es" => "áéíñóúü¿¡ÁÉÍÑÓÚÜ",
        "pt" => "áàâãçéêíóôõúÁÀÂÃÇÉÊÍÓÔÕÚ",
        "it" => "àèéìíîòóùÀÈÉÌÍÎÒÓÙ",
        "nl" => "áéíóúëïöüÁÉÍÓÚËÏÖÜ",
        "da" | "no" => "åæøÅÆØ",
        "sv" | "fi" => "åäöÅÄÖ",
        "is" => "áðéíóúýþæöÁÐÉÍÓÚÝÞÆÖ",
        "cs" => "áčďéěíňóřšťúůýžÁČĎÉĚÍŇÓŘŠŤÚŮÝŽ",
        "pl" => "ąćęłńóśźżĄĆĘŁŃÓŚŹŻ",
        _ => "",
    }
}

/// Where one PP-OCR language's files are: ocrs's detector, the recognition model and its
/// dictionary beside it.
#[derive(Clone, Debug, PartialEq)]
pub struct PpModels {
    pub detection: PathBuf,
    pub recognition: PathBuf,
    pub dictionary: PathBuf,
    pub language: &'static str,
}

impl PpModels {
    /// The files for `language` in `dir`, when all three are there.
    pub fn in_dir(dir: &Path, language: &str) -> Option<PpModels> {
        let &(language, model, dictionary, _) = PP_OCR.iter().find(|(code, ..)| *code == language)?;
        let detection = Models::in_dir(dir)
            .map(|m| m.detection)
            .or_else(|| [DETECTION_MODEL, "text-detection.rten"].into_iter().map(|name| dir.join(name)).find(|path| path.is_file()))?;
        let m = PpModels { detection, recognition: dir.join(model), dictionary: dir.join(dictionary), language };
        (m.recognition.is_file() && m.dictionary.is_file()).then_some(m)
    }

    /// Look where [`Models::search_dirs`] looks, then in the models compiled into the program.
    pub fn find(language: &str) -> Option<PpModels> {
        let mut dirs = Models::search_dirs();
        dirs.extend(embedded_dir());
        dirs.iter().find_map(|d| Self::in_dir(d, language))
    }
}

include!(concat!(env!("OUT_DIR"), "/embedded_models.rs"));

/// The folder holding the models compiled into this program (`PDFCRAFT_EMBED_MODELS`, the one-file
/// Windows build), written out once per version because the recognisers load from files.
pub fn embedded_dir() -> Option<PathBuf> {
    static DIR: std::sync::OnceLock<Option<PathBuf>> = std::sync::OnceLock::new();
    DIR.get_or_init(|| {
        if EMBEDDED.is_empty() {
            return None;
        }
        let base = std::env::var_os("LOCALAPPDATA").map_or_else(std::env::temp_dir, PathBuf::from);
        let dir = base.join("PdfCraft").join("models").join(env!("CARGO_PKG_VERSION"));
        std::fs::create_dir_all(&dir).ok()?;
        for (name, bytes) in EMBEDDED {
            let path = dir.join(name);
            if !std::fs::metadata(&path).is_ok_and(|m| m.len() == bytes.len() as u64) {
                let part = dir.join(format!("{name}.{}.part", std::process::id()));
                std::fs::write(&part, bytes).ok()?;
                std::fs::rename(&part, &path).ok()?;
            }
        }
        Some(dir)
    })
    .clone()
}

/// The longest piece of a line the PP-OCR recognisers read at once, as width over height. Longer
/// lines are read in pieces split at the gaps between their words.
const PIECE_ASPECT: f32 = 40.0;

/// `words` (boxes `[left, top, right, bottom]`, left to right along one line) gathered into
/// pieces of at most [`PIECE_ASPECT`]: each piece is the box around the words it takes. A word
/// longer than that on its own is a piece of its own. Non-finite boxes are left out.
fn line_pieces(words: &[[f32; 4]]) -> Vec<[f32; 4]> {
    let union = |a: [f32; 4], b: [f32; 4]| [a[0].min(b[0]), a[1].min(b[1]), a[2].max(b[2]), a[3].max(b[3])];
    let aspect = |r: [f32; 4]| (r[2] - r[0]) / (r[3] - r[1]).max(1.0);
    let mut pieces = Vec::new();
    let mut current: Option<[f32; 4]> = None;
    for w in words.iter().copied().filter(|w| w.iter().all(|v| v.is_finite())) {
        current = Some(match current {
            Some(c) if aspect(union(c, w)) > PIECE_ASPECT => {
                pieces.push(c);
                w
            }
            Some(c) => union(c, w),
            None => w,
        });
    }
    pieces.extend(current);
    pieces
}

/// One loaded language: ocrs's detector and the PP-OCR reader.
struct Loaded {
    detector: ocrs::OcrEngine,
    reader: PpOcr,
    preferred: &'static str,
}

impl Loaded {
    fn load(models: &PpModels) -> Result<Loaded, OcrError> {
        let &(_, _, _, classes) = PP_OCR.iter().find(|l| l.0 == models.language).ok_or(OcrError::NoModels)?;
        let reader = PpOcr::load(&models.recognition, &models.dictionary, classes)?;
        let reader = if models.language == "th" { reader.tuned(THAI_PADDING, THAI_MIN_SCORE) } else { reader };
        let detection =
            rten::Model::load_file(&models.detection).map_err(|e| OcrError::Load(models.detection.display().to_string(), e.to_string()))?;
        let params = ocrs::OcrEngineParams { detection_model: Some(detection), recognition_model: None, ..Default::default() };
        let detector = ocrs::OcrEngine::new(params).map_err(|e| OcrError::Load("ocr engine".into(), e.to_string()))?;
        Ok(Loaded { detector, reader, preferred: preferred_letters(models.language) })
    }

    fn recognize(&self, image: &OcrImage) -> Result<Vec<Line>, OcrError> {
        let (width, height) = (image.width(), image.height());
        let rgb: Vec<u8> = image.rgba().as_chunks::<4>().0.iter().flat_map(|p| [p[0], p[1], p[2]]).collect();
        let err = |e: &dyn std::fmt::Display| OcrError::Recognize(e.to_string());
        let source = ocrs::ImageSource::from_bytes(&rgb, (width, height)).map_err(|e| err(&e))?;
        let input = self.detector.prepare_input(source).map_err(|e| err(&e))?;
        let found = self.detector.detect_words(&input).map_err(|e| err(&e))?;
        let lines = self.detector.find_text_lines(&input, &found);
        if lines.len() > 4096 {
            return Err(OcrError::Recognize("page has more than 4096 text lines".into()));
        }
        let mut result = Vec::new();
        for line in lines {
            let words: Vec<[f32; 4]> = line.iter().map(|r| r.bounding_rect()).map(|r| [r.left(), r.top(), r.right(), r.bottom()]).collect();
            let mut read = Vec::new();
            for rect in line_pieces(&words) {
                let (text, score) = self.reader.read(&rgb, width, height, rect, self.preferred)?;
                if !text.trim().is_empty() {
                    read.push(Word::new(text, rect, "ppocr").with_confidence(score * 100.0));
                }
            }
            if !read.is_empty() {
                result.push(Line { words: read });
            }
        }
        Ok(result)
    }
}

/// The PP-OCR engine. It loads a language's models the first time that language is read and
/// keeps them.
#[derive(Default)]
pub struct PpOcrRecognizer {
    loaded: Mutex<Vec<(&'static str, std::sync::Arc<Loaded>)>>,
}

impl PpOcrRecognizer {
    pub fn new() -> PpOcrRecognizer {
        PpOcrRecognizer::default()
    }

    /// Whether `language`'s models are installed.
    pub fn available_for(language: &str) -> bool {
        PpModels::find(language).is_some()
    }

    fn loaded(&self, language: &str) -> Result<std::sync::Arc<Loaded>, OcrError> {
        let mut cache = self.loaded.lock().map_err(|_| OcrError::Recognize("the PP-OCR cache is poisoned".into()))?;
        if let Some((_, l)) = cache.iter().find(|(code, _)| *code == language) {
            return Ok(l.clone());
        }
        let models = PpModels::find(language).ok_or(OcrError::NoModels)?;
        let l = std::sync::Arc::new(Loaded::load(&models)?);
        cache.push((models.language, l.clone()));
        Ok(l)
    }
}

impl Recognizer for PpOcrRecognizer {
    fn id(&self) -> &'static str {
        "ppocr"
    }

    fn available(&self) -> bool {
        PP_OCR.iter().any(|l| Self::available_for(l.0))
    }

    fn languages(&self) -> Vec<(String, String)> {
        PP_LANGUAGES.iter().filter(|l| Self::available_for(l.0)).map(|(c, n)| ((*c).to_string(), (*n).to_string())).collect()
    }

    fn recognize(&self, image: &OcrImage, options: &RecognizeOptions) -> Result<Vec<Line>, OcrError> {
        let language = if is_pp_language(&options.language) { options.language.as_str() } else { "th" };
        self.loaded(language)?.recognize(image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_long_line_is_read_in_pieces_at_word_gaps() {
        let words: Vec<[f32; 4]> = (0..30).map(|i| [i as f32 * 100.0, 0.0, i as f32 * 100.0 + 90.0, 20.0]).collect();
        let pieces = line_pieces(&words);
        assert!(pieces.len() > 1);
        assert!(pieces.iter().all(|p| (p[2] - p[0]) / (p[3] - p[1]) <= PIECE_ASPECT));
        assert_eq!(pieces.first().map(|p| p[0]), Some(0.0));
        assert_eq!(pieces.last().map(|p| p[2]), Some(2990.0));
        assert!(line_pieces(&[[f32::NAN, 0.0, 1.0, 1.0]]).is_empty());
    }

    #[test]
    fn every_pp_language_is_listed_once_with_a_model() {
        for (code, _) in PP_LANGUAGES {
            assert!(is_pp_language(code), "{code}");
        }
        assert_eq!(PP_LANGUAGES.len(), PP_OCR.len());
        assert!(!is_pp_language("en"), "English stays with ocrs");
    }

    /// Thai with its own models: the words of a rendered line come back. Skipped without them.
    #[test]
    fn thai_files_are_found_together() {
        let dir = tempfile::tempdir().unwrap();
        for f in [DETECTION_MODEL, THAI_MODEL] {
            std::fs::write(dir.path().join(f), b"model").unwrap();
        }
        assert_eq!(PpModels::in_dir(dir.path(), "th"), None, "no dictionary");
        std::fs::write(dir.path().join(THAI_DICTIONARY), b"k").unwrap();
        let m = PpModels::in_dir(dir.path(), "th").unwrap();
        assert_eq!((m.recognition.file_name().unwrap(), m.language), (std::ffi::OsStr::new(THAI_MODEL), "th"));
        assert_eq!(PpModels::in_dir(dir.path(), "en"), None, "English is not PP-OCR");
    }
}
