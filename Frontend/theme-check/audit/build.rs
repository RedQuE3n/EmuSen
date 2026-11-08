use cxx_qt_build::CxxQtBuilder;

// theme-audit's one piece of glue: a kit singleton found in the engine an object lives in (cpp/audit_glue.cpp).
fn main() {
    CxxQtBuilder::new().files(["src/glue.rs"]).include_dir("cpp").cpp_files(["cpp/audit_glue.cpp"]).qt_module("Qml").build();
}
