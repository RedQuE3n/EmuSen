//! A glyph's height as the system's FreeType hints it at a size, the library ES-DE 3.4.1 draws with: the help bar's icon
//! is sized from it (EmuSen_Qt.md §4.12.1). libfreetype.so.6 is loaded at run time; its absence is an error, never a guess.
//! The call: FT_Set_Char_Size at round(px × 64)/64 px and 72 dpi, FT_Load_Char with FT_LOAD_DEFAULT, the slot's metrics.height.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{c_char, c_int, c_long, c_uint, c_ulong, c_void, CString};

/// The library's file name, as the system's loader finds it.
pub const LIBRARY: &str = "libfreetype.so.6";

type Init = unsafe extern "C" fn(*mut *mut c_void) -> c_int;
type NewFace = unsafe extern "C" fn(*mut c_void, *const c_char, c_long, *mut *mut FaceRec) -> c_int;
type NewMemoryFace = unsafe extern "C" fn(*mut c_void, *const u8, c_long, c_long, *mut *mut FaceRec) -> c_int;
type SetCharSize = unsafe extern "C" fn(*mut FaceRec, c_long, c_long, c_uint, c_uint) -> c_int;
type LoadChar = unsafe extern "C" fn(*mut FaceRec, c_ulong, i32) -> c_int;

/// FT_FaceRec up to its glyph slot, as freetype.h lays it out.
#[repr(C)]
struct FaceRec {
    counts: [c_long; 5],
    names: [*const c_char; 2],
    fixed_sizes: c_int,
    available_sizes: *const c_void,
    charmaps_count: c_int,
    charmaps: *const c_void,
    generic: [*const c_void; 2],
    bbox: [c_long; 4],
    metrics: [i16; 8],
    glyph: *const SlotRec,
}

/// FT_GlyphSlotRec up to its metrics.
#[repr(C)]
struct SlotRec {
    links: [*const c_void; 3],
    glyph_index: [c_uint; 2],
    generic: [*const c_void; 2],
    width: c_long,
    height: c_long,
}

struct FreeType {
    _lib: libloading::Library,
    library: *mut c_void,
    new_face: NewFace,
    new_memory_face: NewMemoryFace,
    set_char_size: SetCharSize,
    load_char: LoadChar,
    faces: HashMap<String, Option<*mut FaceRec>>,
}

impl FreeType {
    fn open() -> Result<FreeType, String> {
        FreeType::open_named(LIBRARY)
    }

    fn open_named(name: &str) -> Result<FreeType, String> {
        let failed = |e: libloading::Error| format!("the help bar's icons are sized by FreeType, and {name} could not be loaded: {e}");
        let lib = unsafe { libloading::Library::new(name) }.map_err(failed)?;
        let (init, new_face, new_memory_face, set_char_size, load_char) = unsafe {
            (
                *lib.get::<Init>(b"FT_Init_FreeType\0").map_err(failed)?,
                *lib.get::<NewFace>(b"FT_New_Face\0").map_err(failed)?,
                *lib.get::<NewMemoryFace>(b"FT_New_Memory_Face\0").map_err(failed)?,
                *lib.get::<SetCharSize>(b"FT_Set_Char_Size\0").map_err(failed)?,
                *lib.get::<LoadChar>(b"FT_Load_Char\0").map_err(failed)?,
            )
        };
        let mut library = std::ptr::null_mut();
        if unsafe { init(&mut library) } != 0 {
            return Err(format!("{name} loaded but did not start"));
        }
        Ok(FreeType { _lib: lib, library, new_face, new_memory_face, set_char_size, load_char, faces: HashMap::new() })
    }

    /// The face of a file, or of the kit's body face ("", bytes that live as long as the program), opened once.
    fn face(&mut self, path: &str) -> Option<*mut FaceRec> {
        let (library, new_face, new_memory_face) = (self.library, self.new_face, self.new_memory_face);
        *self.faces.entry(path.to_owned()).or_insert_with(|| {
            let mut face = std::ptr::null_mut();
            let status = if path.is_empty() {
                let body = emusen_kit_qt::fonts::BODY;
                unsafe { new_memory_face(library, body.as_ptr(), body.len() as c_long, 0, &mut face) }
            } else {
                let file = CString::new(path).ok()?;
                unsafe { new_face(library, file.as_ptr(), 0, &mut face) }
            };
            (status == 0).then_some(face)
        })
    }

    fn height(&mut self, path: &str, px: f64, ch: char) -> Option<f64> {
        let face = self.face(path)?;
        let size = (px * 64.0).round() as c_long;
        unsafe {
            if (self.set_char_size)(face, 0, size, 72, 72) != 0 || (self.load_char)(face, c_ulong::from(ch), 0) != 0 {
                return None;
            }
            Some((*(*face).glyph).height as f64 / 64.0)
        }
    }
}

thread_local! {
    static FREETYPE: RefCell<Option<Result<FreeType, String>>> = const { RefCell::new(None) };
    static HEIGHTS: RefCell<HashMap<(String, u64, char), Option<f64>>> = RefCell::new(HashMap::new());
}

fn with<T>(f: impl FnOnce(&mut FreeType) -> T) -> Result<T, String> {
    FREETYPE.with(|ft| {
        let mut ft = ft.borrow_mut();
        match ft.get_or_insert_with(FreeType::open) {
            Ok(ft) => Ok(f(ft)),
            Err(words) => Err(words.clone()),
        }
    })
}

/// Whether the library is there: none, or the words the painter shows when it is not.
pub fn ready() -> Result<(), String> {
    with(|_| ())
}

/// The height in pixels of `ch` in the face at `path` hinted at `px`; none when the face or its glyph is missing, and an
/// error when the library is.
pub fn height(path: &str, px: f64, ch: char) -> Result<Option<f64>, String> {
    let key = (path.to_owned(), px.to_bits(), ch);
    if let Some(h) = HEIGHTS.with(|m| m.borrow().get(&key).copied()) {
        return Ok(h);
    }
    let h = with(|ft| ft.height(path, px, ch))?;
    HEIGHTS.with(|m| m.borrow_mut().insert(key, h));
    Ok(h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_library_is_an_error_in_words() {
        let words = FreeType::open_named("libfreetype-absent.so.0").err().expect("no such library");
        assert!(words.starts_with("the help bar's icons are sized by FreeType, and libfreetype-absent.so.0 could not be loaded"), "{words}");
    }

    #[test]
    fn the_kit_s_body_face_is_measured_from_its_bytes() {
        assert!(height("", 20.0, 'S').unwrap().is_some_and(|h| h > 10.0));
        assert_eq!(height("/nonexistent/face.ttf", 20.0, 'S'), Ok(None));
    }
}
