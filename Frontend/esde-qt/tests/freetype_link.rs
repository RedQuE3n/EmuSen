//! The build's FreeType plan (build/freetype.rs, EmuSen_Qt.md §4.12.4): pkg-config's folders when it knows freetype2,
//! ldconfig's word when it does not, and a build that stops naming the library when neither finds it.

#[path = "../build/freetype.rs"]
mod freetype;

#[test]
fn a_missing_freetype_is_named_and_a_found_one_linked() {
    let found = "\tlibfreetype.so.6 (libc6,x86-64) => /usr/lib64/libfreetype.so.6\n";
    assert_eq!(freetype::plan(Ok("-L/opt/ft/lib -lfreetype\n".into()), None), Ok((vec!["/opt/ft/lib".to_owned()], None)));
    let (dirs, warning) = freetype::plan(Err("Package freetype2 was not found".into()), Some(found.into())).unwrap();
    assert!(dirs.is_empty() && warning.is_some_and(|w| w.contains("pkg-config does not know freetype2")));
    let missing = freetype::plan(Err("Package freetype2 was not found".into()), Some("\tlibc.so.6 => /lib64/libc.so.6\n".into()));
    assert!(missing.as_ref().is_err_and(|w| w.contains("libfreetype.so.6") && w.contains("install FreeType")), "{missing:?}");
    let unknown = freetype::plan(Err("it did not run".into()), None).unwrap();
    assert!(unknown.1.is_some_and(|w| w.contains("libfreetype.so.6")), "a build that cannot tell still names the library");
}
