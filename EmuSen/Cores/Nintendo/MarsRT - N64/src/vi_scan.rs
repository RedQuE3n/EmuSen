//! The VI's scan-out: its registers and RDRAM in, the picture out. A stub until the VI stage lands. See Mars_Native.md §5.2.

use crate::vi::Vi;

/// C#'s raster and composed frame, which no state holds.
#[derive(Default)]
pub struct Scanout {
    /// The composed RGBA frame, `width * height * 4` bytes.
    pub frame: Vec<u8>,
    pub width: u32,
    pub height: u32,
    /// C#'s `RowRepeat`: how many times each row of `frame` is shown.
    pub row_repeat: u32,
}

/// C#'s `Vi.Scan` then `MarsCore.Compose`: false when the VI shows nothing.
pub fn scan(vi: &Vi, rdram: &[u8], hidden: &[u8], out: &mut Scanout) -> bool {
    let _ = (vi, rdram, hidden, out);
    false
}
