//! The running session's pictures by address, for QML's `Image` (EmuSen_Qt.md §7.3): the reel's moments
//! (`<serial>/reel/<frame>`), the picture on show (`<serial>/now`) and a state slot's picture (`<serial>/file/<path>`),
//! read from the runtime when asked, on the thread that asks. A `QQuickImageProvider` serves them, C++ glue in
//! `cpp/session_pictures.cpp`, since cxx-qt subclasses no Qt class but a QObject's.

use cxx_qt_lib::{QImage, QImageFormat, QString};
use emusen_crystal_client::shapes::SessionId;
use emusen_crystal_client::Runtime;
use std::sync::{Arc, Mutex, PoisonError};

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qimage.h");
        type QImage = cxx_qt_lib::QImage;
        include!(<QtCore/QObject>);
        type QObject = cxx_qt::QObject;
        include!("session_pictures.h");

        /// `source` served under `name` by the QML engine that holds `object`; false where it is held by none.
        #[cxx_name = "sessionPicturesServe"]
        fn serve(object: &QObject, name: &QString, source: Box<PictureSource>) -> bool;

        /// A picture written as a PNG file, by Qt's image writer; false where it could not be.
        #[cxx_name = "savePicture"]
        fn save_png(image: &QImage, path: &QString) -> bool;
    }

    extern "Rust" {
        type PictureSource;

        #[cxx_name = "pictureOf"]
        fn picture_of(source: &PictureSource, id: &QString) -> QImage;
    }
}

/// The name the pictures are served under: `image://session/...`.
pub const NAME: &str = "session";

/// The session the pictures are read from, none while no game is open; shared with the model that opens it.
pub type Shown = Arc<Mutex<Option<(Runtime, SessionId)>>>;

/// What the provider reads from.
pub struct PictureSource {
    shown: Shown,
}

impl PictureSource {
    pub fn new(shown: Shown) -> Box<PictureSource> {
        Box::new(PictureSource { shown })
    }
}

/// A picture by its address after the serial, as RGBA rows with its width and height.
pub fn picture(runtime: &Runtime, session: SessionId, what: &str) -> Option<(Vec<u8>, i32, i32)> {
    if let Some(frame) = what.strip_prefix("reel/") {
        let packed = runtime.rewind_picture(session, frame.parse().ok()?).ok()??;
        unpack(packed)
    } else if what == "now" {
        now(runtime, session)
    } else {
        None
    }
}

/// A packed picture as the runtime hands one: width and height as two little-endian u32, then RGBA rows.
fn unpack(mut packed: Vec<u8>) -> Option<(Vec<u8>, i32, i32)> {
    let word = |at: usize| packed.get(at..at + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as i32);
    let (width, height) = (word(0)?, word(4)?);
    if width <= 0 || height <= 0 || packed.len() < 8 + (width * height * 4) as usize {
        return None;
    }
    packed.drain(..8);
    packed.truncate((width * height * 4) as usize);
    Some((packed, width, height))
}

/// The picture on show, its repeated rows written out (Mistress's `CurrentPicture`); none where none is read back.
fn now(runtime: &Runtime, session: SessionId) -> Option<(Vec<u8>, i32, i32)> {
    let shown = runtime.session_info(session).ok()?.picture;
    let (width, height, repeat) = (shown.width, shown.height, shown.row_repeat.max(1));
    if width <= 0 || height <= 0 {
        return None;
    }
    let row = width as usize * 4;
    let mut rgba = vec![0u8; row * height as usize];
    if runtime.picture_copy(session, &mut rgba).ok()? < rgba.len() {
        return None;
    }
    if repeat == 1 {
        return Some((rgba, width, height));
    }
    let tall: Vec<u8> = rgba.chunks_exact(row).flat_map(|r| std::iter::repeat_n(r, repeat as usize).flatten().copied()).collect();
    Some((tall, width, height * repeat))
}

/// The picture on show written to `path` as a PNG (Mistress's `WritePicture`); false where none is read back or written.
pub fn save_now(runtime: &Runtime, session: SessionId, path: &str) -> bool {
    let Some((rgba, width, height)) = now(runtime, session) else { return false };
    // SAFETY: the bytes are width × height RGBA rows, which QImage takes over and frees with them.
    let image = unsafe { QImage::from_raw_bytes(rgba, width, height, QImageFormat::Format_RGBA8888) };
    ffi::save_png(&image, &QString::from(path))
}

fn picture_of(source: &PictureSource, id: &QString) -> QImage {
    let id = id.to_string();
    let what = id.split_once('/').map_or("", |(_, what)| what);
    if let Some(path) = what.strip_prefix("file/") {
        let bytes = std::fs::read(path).unwrap_or_default();
        return QImage::from_data(&bytes, None).unwrap_or_default();
    }
    let shown = source.shown.lock().unwrap_or_else(PoisonError::into_inner).clone();
    let Some((runtime, session)) = shown else { return QImage::default() };
    match picture(&runtime, session, what) {
        // SAFETY: the bytes are width × height RGBA rows, which QImage takes over and frees with them.
        Some((rgba, width, height)) => unsafe { QImage::from_raw_bytes(rgba, width, height, QImageFormat::Format_RGBA8888) },
        None => QImage::default(),
    }
}
