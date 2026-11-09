use cxx_qt_build::CxxQtBuilder;

// theme-audit has no glue of its own: the kit's singletons are found through the theme host's (emusen-theme-qt).
fn main() {
    CxxQtBuilder::new().qt_module("Qml").build();
}
