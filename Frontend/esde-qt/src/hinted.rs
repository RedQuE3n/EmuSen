//! A glyph's height as the system's FreeType renders it at a size, the library ES-DE 3.4.1 draws with: the help bar's row
//! and icon are sized from it (EmuSen_Qt.md §4.12.4). libfreetype.so.6 is linked by build.rs, the library Qt loads.
//! The call is ES-DE's (es-core Font.cpp, FontFace and Font::getGlyph): FT_Set_Char_Size at trunc(px × 64) in f32, the
//! default 72 dpi, FT_Load_Char with FT_LOAD_RENDER, and the rendered bitmap's rows.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{c_char, c_int, c_long, c_uint, c_ulong, c_void, CString};

/// FT_LOAD_RENDER (freetype.h): the glyph is rendered to a bitmap as it loads.
const LOAD_RENDER: i32 = 1 << 2;

unsafe extern "C" {
    fn FT_Init_FreeType(library: *mut *mut c_void) -> c_int;
    fn FT_New_Face(library: *mut c_void, path: *const c_char, index: c_long, face: *mut *mut FaceRec) -> c_int;
    fn FT_New_Memory_Face(library: *mut c_void, data: *const u8, size: c_long, index: c_long, face: *mut *mut FaceRec) -> c_int;
    fn FT_Set_Char_Size(face: *mut FaceRec, width: c_long, height: c_long, h_dpi: c_uint, v_dpi: c_uint) -> c_int;
    fn FT_Load_Char(face: *mut FaceRec, code: c_ulong, flags: i32) -> c_int;
}

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

/// FT_GlyphSlotRec up to its bitmap's rows, as freetype.h lays it out.
#[repr(C)]
struct SlotRec {
    links: [*const c_void; 3],
    glyph_index: c_uint,
    generic: [*const c_void; 2],
    metrics: [c_long; 8],
    linear_advances: [c_long; 2],
    advance: [c_long; 2],
    format: c_uint,
    bitmap: Bitmap,
}

/// FT_Bitmap up to its buffer; its pointer aligns it after the slot's format.
#[repr(C)]
struct Bitmap {
    rows: c_uint,
    width: c_uint,
    pitch: c_int,
    buffer: *const u8,
}

/// The height FreeType is given for a size, in 26.6 points at 72 dpi: px × 64 in f32, truncated by the integer cast, as
/// ES-DE 3.4.1's FontFace does (EmuSen_Qt.md §4.12.4).
pub fn char_size(px: f32) -> c_long {
    (px * 64.0) as c_long
}

/// The painter's problem when FreeType does not start.
pub const NOT_STARTED: &str = "the help bar's icons are sized by FreeType, and FreeType did not start";

struct FreeType {
    library: *mut c_void,
    faces: HashMap<String, Option<*mut FaceRec>>,
}

impl FreeType {
    fn open() -> Result<FreeType, String> {
        let mut library = std::ptr::null_mut();
        if unsafe { FT_Init_FreeType(&mut library) } != 0 {
            return Err(NOT_STARTED.into());
        }
        Ok(FreeType { library, faces: HashMap::new() })
    }

    /// The face of a file, or of the kit's body face ("", bytes that live as long as the program), opened once.
    fn face(&mut self, path: &str) -> Option<*mut FaceRec> {
        let library = self.library;
        *self.faces.entry(path.to_owned()).or_insert_with(|| {
            let mut face = std::ptr::null_mut();
            let status = if path.is_empty() {
                let body = emusen_kit_qt::fonts::BODY;
                unsafe { FT_New_Memory_Face(library, body.as_ptr(), body.len() as c_long, 0, &mut face) }
            } else {
                let file = CString::new(path).ok()?;
                unsafe { FT_New_Face(library, file.as_ptr(), 0, &mut face) }
            };
            (status == 0).then_some(face)
        })
    }

    fn rows(&mut self, path: &str, px: f32, ch: char) -> Option<u32> {
        let face = self.face(path)?;
        unsafe {
            if FT_Set_Char_Size(face, 0, char_size(px), 0, 0) != 0 || FT_Load_Char(face, c_ulong::from(ch), LOAD_RENDER) != 0 {
                return None;
            }
            Some((*(*face).glyph).bitmap.rows)
        }
    }
}

thread_local! {
    static FREETYPE: RefCell<Option<Result<FreeType, String>>> = const { RefCell::new(None) };
    static ROWS: RefCell<HashMap<(String, u32, char), Option<u32>>> = RefCell::new(HashMap::new());
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

/// Whether the library started: none, or the words the painter shows when it did not.
pub fn ready() -> Result<(), String> {
    with(|_| ())
}

/// The rows of `ch`'s bitmap in the face at `path` rendered at `px`, ES-DE's letter height; none when the face or its
/// glyph is missing, and an error when the library did not start.
pub fn rows(path: &str, px: f32, ch: char) -> Result<Option<u32>, String> {
    let key = (path.to_owned(), px.to_bits(), ch);
    if let Some(r) = ROWS.with(|m| m.borrow().get(&key).copied()) {
        return Ok(r);
    }
    let r = with(|ft| ft.rows(path, px, ch))?;
    ROWS.with(|m| m.borrow_mut().insert(key, r));
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Art Book Next at 1280 × 800: 0.04583333 × 800 in f32 is 36.666664 px, 2346.67 sixty-fourths, truncated to 2346.
    #[test]
    fn the_size_in_sixty_fourths_is_truncated_from_the_f32_product() {
        let px = 0.045_833_33_f32 * 800.0;
        assert_eq!(px, 36.666_664);
        assert_eq!(char_size(px), 2346);
        assert_eq!(char_size(54.999_996), 3519);
        assert_eq!(char_size(20.0), 1280);
    }

    #[test]
    fn a_library_that_did_not_start_is_an_error_that_says_so() {
        FREETYPE.with(|ft| *ft.borrow_mut() = Some(Err(NOT_STARTED.to_owned())));
        assert_eq!(ready(), Err(NOT_STARTED.to_owned()));
        assert_eq!(rows("", 20.0, 'S'), Err(NOT_STARTED.to_owned()));
    }

    #[test]
    fn the_kit_s_body_face_is_measured_from_its_bytes() {
        assert!(rows("", 20.0, 'S').unwrap().is_some_and(|h| h > 10));
        assert_eq!(rows("/nonexistent/face.ttf", 20.0, 'S'), Ok(None));
    }
}
