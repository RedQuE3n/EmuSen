//! bindgen's two views of the header: whole, for layouts and constants, and with its structs replaced by
//! emusen-native's own, for the function types the generated exports are assigned to.

use std::path::PathBuf;
use std::sync::Mutex;

static MACROS: Mutex<Vec<(String, i64)>> = Mutex::new(Vec::new());

#[derive(Debug)]
struct Macros;

impl bindgen::callbacks::ParseCallbacks for Macros {
    fn int_macro(&self, name: &str, value: i64) -> Option<bindgen::callbacks::IntKind> {
        if name.starts_with("EMUSEN_") {
            MACROS.lock().unwrap().push((name.to_owned(), value));
        }
        None
    }
}

const STRUCTS: [(&str, &str); 5] = [
    ("emusen_machine", "Machine"),
    ("emusen_create_params", "CreateParams"),
    ("emusen_file", "FileEntry"),
    ("emusen_frame_info", "FrameInfo"),
    ("emusen_event", "Event"),
];

fn main() {
    let header = "../emusen-native/include/emusen_core.h";
    println!("cargo:rerun-if-changed={header}");
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    bindgen::Builder::default()
        .header(header)
        .allowlist_file(".*emusen_core\\.h")
        .parse_callbacks(Box::new(Macros))
        .layout_tests(false)
        .generate()
        .expect("bindgen on emusen_core.h")
        .write_to_file(out.join("c.rs"))
        .unwrap();

    let mut sig = bindgen::Builder::default().header(header).allowlist_type("emusen_core_.*_fn").layout_tests(false);
    for (c, rust) in STRUCTS {
        sig = sig.blocklist_type(c).raw_line(format!("pub use emusen_native::core::sys::{rust} as {c};"));
    }
    sig.generate().expect("bindgen on emusen_core.h").write_to_file(out.join("sig.rs")).unwrap();

    let mut list = MACROS.lock().unwrap().clone();
    list.sort();
    list.dedup();
    let body: String = list.iter().map(|(n, v)| format!("    ({n:?}, {v}),\n")).collect();
    std::fs::write(out.join("macros.rs"), format!("/// Every integer macro of the header as bindgen evaluates it.\npub const MACROS: &[(&str, i64)] = &[\n{body}];\n")).unwrap();
}
