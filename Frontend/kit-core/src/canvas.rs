//! The big screen's canvas: the design height every screen is laid out in and scaled from to the window's height
//! (EmuSen_Kit.md §7.112, EmuSen_Qt.md §1.4). Its one owner; the kit's `CanvasUnits` gives it to QML and the night sky measures in it.

/// The height of the canvas in design units, a constant of the design itself.
pub const DESIGN_HEIGHT: f64 = 800.0;
