//! The C layer over emusen-systems: the table as JSON, and the save media's format modules, for the C# runtime's
//! `SystemTable` and `SystemCode` facades (EmuSen_Backend.md §5.4, §12, §13). Nothing is decided here.

use crate::{bytes, fail, given, put, required, status};
use emusen_systems::media::{self, Fault};

/// The table as `emusen_systems::to_json` writes it, by the length-query idiom.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_systems_table(out: *mut u8, len: usize) -> i64 {
    match emusen_systems::table_json() {
        Ok(text) => unsafe { put(text.as_bytes(), out, len) },
        Err(e) => fail(status::PARSE, e.to_string()) as i64,
    }
}

/// Every save medium's module facts as JSON, by the length-query idiom.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_systems_media(out: *mut u8, len: usize) -> i64 {
    unsafe { put(media::modules_json().as_bytes(), out, len) }
}

fn medium(id: &str) -> Result<&'static str, i32> {
    media::module(id).map(|m| m.id).ok_or_else(|| fail(status::ABSENT, format!("no module reads the medium {id}")))
}

fn edited(result: Option<Result<Vec<u8>, Fault>>, out: *mut u8, len: usize) -> i64 {
    match result {
        Some(Ok(image)) => unsafe { put(&image, out, len) },
        Some(Err(Fault::Refused(words))) => fail(status::REFUSED, words) as i64,
        Some(Err(Fault::Outside)) => fail(status::BAD_ARGUMENT, "the directory reaches outside the image") as i64,
        None => fail(status::ABSENT, "no module reads the medium") as i64,
    }
}

/// A medium's listing as JSON, by the length-query idiom.
///
/// # Safety
/// Each pointer must be valid for its length, or null; `out` for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_systems_media_read(id: *const u8, id_len: usize, image: *const u8, image_len: usize, out: *mut u8, len: usize) -> i64 {
    let (id, image) = match unsafe { (required(id, id_len), bytes(image, image_len)) } {
        (Ok(id), Ok(image)) => (id, image),
        (Err(s), _) | (_, Err(s)) => return s as i64,
    };
    let id = match medium(id) {
        Ok(id) => id,
        Err(s) => return s as i64,
    };
    match media::read(id, image) {
        Some(Ok(listing)) => unsafe { put(media::listing_json(&listing).as_bytes(), out, len) },
        Some(Err(_)) => fail(status::BAD_ARGUMENT, "the directory reaches outside the image") as i64,
        None => fail(status::ABSENT, "no module reads the medium") as i64,
    }
}

/// `to` with the save named `save` on `from` copied onto it, by the length-query idiom; REFUSED with the words.
///
/// # Safety
/// Each pointer must be valid for its length, or null; `out` for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_systems_media_copy(
    id: *const u8,
    id_len: usize,
    from: *const u8,
    from_len: usize,
    save: *const u8,
    save_len: usize,
    to: *const u8,
    to_len: usize,
    out: *mut u8,
    len: usize,
) -> i64 {
    let (id, from, save, to) = match unsafe { (required(id, id_len), bytes(from, from_len), given(save, save_len), bytes(to, to_len)) } {
        (Ok(id), Ok(from), Ok(save), Ok(to)) => (id, from, save, to),
        (Err(s), ..) | (_, Err(s), ..) | (_, _, Err(s), _) | (.., Err(s)) => return s as i64,
    };
    match medium(id) {
        Ok(id) => edited(media::copy(id, from, save, to), out, len),
        Err(s) => s as i64,
    }
}

/// `image` with the save named `save` deleted, by the length-query idiom; REFUSED with the words.
///
/// # Safety
/// Each pointer must be valid for its length, or null; `out` for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_systems_media_delete(id: *const u8, id_len: usize, image: *const u8, image_len: usize, save: *const u8, save_len: usize, out: *mut u8, len: usize) -> i64 {
    let (id, image, save) = match unsafe { (required(id, id_len), bytes(image, image_len), given(save, save_len)) } {
        (Ok(id), Ok(image), Ok(save)) => (id, image, save),
        (Err(s), ..) | (_, Err(s), _) | (.., Err(s)) => return s as i64,
    };
    match medium(id) {
        Ok(id) => edited(media::delete(id, image, save), out, len),
        Err(s) => s as i64,
    }
}

/// A formatted blank of the medium, by the length-query idiom.
///
/// # Safety
/// `id` must be valid for `id_len` bytes; `out` for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_systems_media_format(id: *const u8, id_len: usize, out: *mut u8, len: usize) -> i64 {
    let id = match unsafe { required(id, id_len) }.and_then(medium) {
        Ok(id) => id,
        Err(s) => return s as i64,
    };
    edited(media::format(id).map(Ok), out, len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_comes_through_the_c_layer_whole() {
        let n = unsafe { emusen_systems_table(std::ptr::null_mut(), 0) };
        assert!(n > 0);
        let mut small = vec![0u8; n as usize - 1];
        assert_eq!(unsafe { emusen_systems_table(small.as_mut_ptr(), small.len()) }, n);
        assert!(small.iter().all(|&b| b == 0), "a buffer too small is not written");
        let mut out = vec![0u8; n as usize];
        assert_eq!(unsafe { emusen_systems_table(out.as_mut_ptr(), out.len()) }, n);
        assert_eq!(std::str::from_utf8(&out).unwrap(), emusen_systems::table_json().unwrap());
    }

    fn call(f: impl Fn(*mut u8, usize) -> i64) -> Result<Vec<u8>, (i64, String)> {
        let n = f(std::ptr::null_mut(), 0);
        if n < 0 {
            let mut words = vec![0u8; 512];
            let w = unsafe { crate::emusen_platform_last_error(words.as_mut_ptr(), words.len()) };
            return Err((n, String::from_utf8_lossy(&words[..w as usize]).into_owned()));
        }
        let mut out = vec![0u8; n as usize];
        assert_eq!(f(out.as_mut_ptr(), out.len()), n);
        Ok(out)
    }

    #[test]
    fn a_card_is_formatted_read_and_refused_through_the_c_layer() {
        let id = "psx.card";
        let blank = call(|o, l| unsafe { emusen_systems_media_format(id.as_ptr(), id.len(), o, l) }).unwrap();
        assert_eq!(blank.len(), 128 * 1024);
        let listing = call(|o, l| unsafe { emusen_systems_media_read(id.as_ptr(), id.len(), blank.as_ptr(), blank.len(), o, l) }).unwrap();
        assert_eq!(std::str::from_utf8(&listing).unwrap(), r#"{"formatted":true,"saves":[],"free":15,"problem":null}"#);
        let save = "BASLUS-00001";
        let refused = call(|o, l| unsafe { emusen_systems_media_delete(id.as_ptr(), id.len(), blank.as_ptr(), blank.len(), save.as_ptr(), save.len(), o, l) });
        assert_eq!(refused, Err((status::REFUSED as i64, "no save named BASLUS-00001 is on the card".into())));
        let nameless = call(|o, l| unsafe { emusen_systems_media_delete(id.as_ptr(), id.len(), blank.as_ptr(), blank.len(), std::ptr::null(), 0, o, l) });
        assert_eq!(nameless, Err((status::REFUSED as i64, "no save named  is on the card".into())), "an empty name may come as a null");
        let empty = call(|o, l| unsafe { emusen_systems_media_read(id.as_ptr(), id.len(), std::ptr::null(), 0, o, l) }).unwrap();
        assert!(std::str::from_utf8(&empty).unwrap().contains("this file is 0"));
        let unknown = "snes.sram";
        assert_eq!(call(|o, l| unsafe { emusen_systems_media_format(unknown.as_ptr(), unknown.len(), o, l) }).unwrap_err().0, status::ABSENT as i64);
        assert!(call(|o, l| unsafe { emusen_systems_media(o, l) }).unwrap().starts_with(br#"[{"id":"psx.card","length":131072,"capacity":15,"unit":"block"}"#));
    }
}
