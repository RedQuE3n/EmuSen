//! The ABI's facts, one per baseline line (§5.4): functions, structs and fields from libclang's view of the
//! header for a target triple, constants from bindgen's, exports from emusen-native's table, descriptor fields
//! from the committed schemas.

use std::collections::BTreeMap;
use std::ffi::{CStr, CString, c_void};
use std::path::Path;

use clang_sys::*;
use emusen_native::core::{schema, sys};
use emusen_native::json::Value;

/// The triples the project builds; the facts must be the same for each.
pub const TRIPLES: [&str; 4] = ["x86_64-unknown-linux-gnu", "x86_64-pc-windows-msvc", "aarch64-apple-darwin", "x86_64-apple-darwin"];

/// `key value`: the key names the item, the value is what may not change.
pub type Facts = BTreeMap<String, String>;

/// What libclang read: the facts, and the header's own inconsistencies (a function type unlike its function).
pub struct Header {
    pub facts: Facts,
    pub functions: Vec<String>,
    pub fields: Vec<String>,
    pub enum_constants: Vec<String>,
    pub problems: Vec<String>,
}

fn text(s: CXString) -> String {
    unsafe {
        let p = clang_getCString(s);
        let t = if p.is_null() { String::new() } else { CStr::from_ptr(p).to_string_lossy().into_owned() };
        clang_disposeString(s);
        t
    }
}

fn spelling(t: CXType) -> String {
    text(unsafe { clang_getTypeSpelling(t) }).replace(" *", "*")
}

fn children(c: CXCursor) -> Vec<CXCursor> {
    extern "C" fn visit(c: CXCursor, _parent: CXCursor, data: CXClientData) -> CXChildVisitResult {
        unsafe { (*(data as *mut Vec<CXCursor>)).push(c) };
        CXChildVisit_Continue
    }
    let mut out: Vec<CXCursor> = Vec::new();
    unsafe { clang_visitChildren(c, visit, &mut out as *mut Vec<CXCursor> as *mut c_void) };
    out
}

/// The header parsed for `triple`, freestanding so that no target's system headers are needed.
pub fn header(path: &Path, triple: &str) -> Result<Header, String> {
    let mut args: Vec<String> = ["-x", "c", "-std=c99", "-ffreestanding", "-target", triple].iter().map(|a| a.to_string()).collect();
    // The compiler's own headers, as bindgen finds them; a loaded libclang does not always know where they are.
    let clang = clang_sys::support::Clang::find(None, &[]).ok_or("no clang binary to find the compiler's headers with")?;
    for dir in clang.c_search_paths.unwrap_or_default() {
        if dir.join("stddef.h").exists() {
            args.push("-isystem".to_owned());
            args.push(dir.to_string_lossy().into_owned());
        }
    }
    let args: Vec<CString> = args.into_iter().map(|a| CString::new(a).unwrap()).collect();
    let argv: Vec<*const i8> = args.iter().map(|a| a.as_ptr()).collect();
    let file = CString::new(path.to_str().ok_or("a path that is not UTF-8")?).unwrap();
    let mut h = Header { facts: Facts::new(), functions: Vec::new(), fields: Vec::new(), enum_constants: Vec::new(), problems: Vec::new() };
    unsafe {
        let index = clang_createIndex(0, 0);
        let tu = clang_parseTranslationUnit(index, file.as_ptr(), argv.as_ptr(), argv.len() as i32, std::ptr::null_mut(), 0, CXTranslationUnit_None);
        if tu.is_null() {
            return Err(format!("libclang could not parse the header for {triple}"));
        }
        for i in 0..clang_getNumDiagnostics(tu) {
            let d = clang_getDiagnostic(tu, i);
            if clang_getDiagnosticSeverity(d) >= CXDiagnostic_Error {
                h.problems.push(format!("{triple}: {}", text(clang_formatDiagnostic(d, clang_defaultDiagnosticDisplayOptions()))));
            }
            clang_disposeDiagnostic(d);
        }
        let mut function_types: BTreeMap<String, String> = BTreeMap::new();
        let mut typedefs: Vec<(String, String)> = Vec::new();
        for c in children(clang_getTranslationUnitCursor(tu)) {
            if clang_Location_isFromMainFile(clang_getCursorLocation(c)) == 0 {
                continue;
            }
            let name = text(clang_getCursorSpelling(c));
            match clang_getCursorKind(c) {
                CXCursor_FunctionDecl => {
                    let ty = clang_getCursorType(c);
                    let params: Vec<String> = (0..clang_Cursor_getNumArguments(c).max(0) as u32).map(|i| spelling(clang_getCursorType(clang_Cursor_getArgument(c, i)))).collect();
                    let value = format!("({}) -> {}", params.join(", "), spelling(clang_getResultType(ty)));
                    function_types.insert(name.clone(), spelling(clang_getCanonicalType(ty)));
                    h.facts.insert(format!("fn {name}"), value);
                    h.functions.push(name);
                }
                CXCursor_TypedefDecl if name.ends_with("_fn") => {
                    let pointee = clang_getPointeeType(clang_getCanonicalType(clang_getTypedefDeclUnderlyingType(c)));
                    typedefs.push((name, spelling(pointee)));
                }
                CXCursor_StructDecl if clang_isCursorDefinition(c) != 0 => {
                    let ty = clang_getCursorType(c);
                    h.facts.insert(format!("struct {name}"), format!("size {} align {}", clang_Type_getSizeOf(ty), clang_Type_getAlignOf(ty)));
                    for f in children(c).into_iter().filter(|f| clang_getCursorKind(*f) == CXCursor_FieldDecl) {
                        let field = text(clang_getCursorSpelling(f));
                        let offset = clang_Cursor_getOffsetOfField(f);
                        h.facts.insert(format!("field {name}.{field}"), format!("offset {} type {}", offset / 8, spelling(clang_getCursorType(f))));
                        h.fields.push(format!("{name}.{field}"));
                    }
                }
                CXCursor_EnumDecl => {
                    for k in children(c).into_iter().filter(|k| clang_getCursorKind(*k) == CXCursor_EnumConstantDecl) {
                        let constant = text(clang_getCursorSpelling(k));
                        h.facts.insert(format!("enum {name}.{constant}"), clang_getEnumConstantDeclValue(k).to_string());
                        h.enum_constants.push(format!("{name}_{constant}"));
                    }
                }
                _ => {}
            }
        }
        for f in &h.functions {
            let want = format!("{f}_fn");
            match typedefs.iter().find(|(n, _)| *n == want) {
                Some((_, t)) if Some(t) == function_types.get(f) => {}
                Some((_, t)) => h.problems.push(format!("{want} is {t}, but {f} is {}", function_types[f])),
                None => h.problems.push(format!("{f} has no function type {want}")),
            }
        }
        clang_disposeTranslationUnit(tu);
        clang_disposeIndex(index);
    }
    Ok(h)
}

/// The macros as bindgen evaluated them: a negative one is a status, the rest constants in hex.
pub fn constants(macros: &[(&str, i64)]) -> Facts {
    macros
        .iter()
        .map(|&(name, v)| if v < 0 { (format!("status {name}"), v.to_string()) } else { (format!("const {name}"), format!("{v:#x}")) })
        .collect()
}

/// Each export, required or claimed by its bit.
pub fn exports() -> Facts {
    sys::EXPORTS
        .iter()
        .map(|&(name, bit)| {
            let value = match sys::CAPABILITY_NAMES.iter().find(|(b, _)| *b == bit) {
                None => "required".to_owned(),
                Some((_, cap)) => format!("optional EMUSEN_CAP_{cap}"),
            };
            (format!("export {name}"), value)
        })
        .collect()
}

/// The documents and their schemas, under the names the baseline gives them.
pub const DOCUMENTS: [(&str, &str); 6] = [
    ("info", schema::INFO),
    ("machine_info", schema::MACHINE_INFO),
    ("settings", schema::SETTINGS),
    ("firmware", schema::FIRMWARE),
    ("setting_notes", schema::SETTING_NOTES),
    ("disassembly", schema::DISASSEMBLY),
];

fn type_of(root: &Value, s: &Value) -> String {
    let s = schema::resolve(root, s);
    let base = match s.get("type") {
        Some(Value::Str(t)) => t.clone(),
        Some(Value::Array(ts)) => ts.iter().filter_map(Value::as_str).collect::<Vec<_>>().join("|"),
        _ => "any".to_owned(),
    };
    match (base.as_str(), s.get("items")) {
        ("array", Some(items)) => format!("array<{}>", type_of(root, items)),
        _ => base,
    }
}

fn walk(root: &Value, s: &Value, path: &str, out: &mut Facts) {
    let s = schema::resolve(root, s);
    if let Some(Value::Object(props)) = s.get("properties") {
        let required: Vec<&str> = s.get("required").and_then(Value::as_array).unwrap_or(&[]).iter().filter_map(Value::as_str).collect();
        for (name, child) in props {
            let p = format!("{path}.{name}");
            let need = if required.contains(&name.as_str()) { "required" } else { "optional" };
            out.insert(format!("json {p}"), format!("{} {need}", type_of(root, child)));
            walk(root, child, &p, out);
        }
    }
    if let Some(extra) = s.get("additionalProperties") {
        let p = format!("{path}{{}}");
        out.insert(format!("json {p}"), format!("{} optional", type_of(root, extra)));
        walk(root, extra, &p, out);
    }
    if let Some(items) = s.get("items") {
        walk(root, items, &format!("{path}[]"), out);
    }
}

/// Every descriptor field the schemas define.
pub fn descriptors() -> Result<Facts, String> {
    let mut out = Facts::new();
    for (name, text) in DOCUMENTS {
        let root = emusen_native::json::parse(text.as_bytes()).map_err(|e| format!("{name}'s schema is not JSON at byte {}", e.at))?;
        out.insert(format!("json {name}"), type_of(&root, &root));
        walk(&root, &root, name, &mut out);
    }
    Ok(out)
}
