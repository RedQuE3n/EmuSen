//! The kit's body face as bytes, for a component that lays words out itself (EmuSen_Kit.md §7.60); QML reads the same file at `BODY_URL`.

/// Barlow Regular, the kit's body face, the file `fonts/Fonts.qrc` carries.
pub const BODY: &[u8] = include_bytes!("../fonts/Barlow-Regular.ttf");

/// The same face as QML loads it from the kit's resources.
pub const BODY_URL: &str = "qrc:/emusen/fonts/Barlow-Regular.ttf";
