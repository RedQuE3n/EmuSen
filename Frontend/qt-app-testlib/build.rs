use cxx_qt_build::CxxQtBuilder;

// Links Qt and exported cxx-qt initializers from dependencies for the testlib unit tests.
fn main() {
    CxxQtBuilder::new().qt_module("Qml").build();
}
