//! With `PDFCRAFT_EMBED_MODELS` set to a folder of `cargo xtask models` output, English and Thai
//! are compiled into the program, for a build that ships as one file. Unset, nothing is embedded
//! and the models stay files beside the program, as in every other package.

use std::path::{Path, PathBuf};

const EMBEDDED: [&str; 4] = ["text-detection.onnx", "text-recognition.onnx", "thai-recognition.onnx", "thai-keys.txt"];

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=PDFCRAFT_EMBED_MODELS");
    let mut code = String::from("const EMBEDDED: &[(&str, &[u8])] = &[\n");
    if let Some(dir) = std::env::var_os("PDFCRAFT_EMBED_MODELS") {
        for name in EMBEDDED {
            let path = Path::new(&dir).join(name);
            let path = path.canonicalize().unwrap_or_else(|e| panic!("PDFCRAFT_EMBED_MODELS: {}: {e}", path.display()));
            println!("cargo::rerun-if-changed={}", path.display());
            code.push_str(&format!("    ({name:?}, include_bytes!({:?})),\n", path.display().to_string()));
        }
    }
    code.push_str("];\n");
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("cargo sets OUT_DIR")).join("embedded_models.rs");
    std::fs::write(out, code).expect("writing embedded_models.rs");
}
