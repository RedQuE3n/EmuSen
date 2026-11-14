// EmuSen_Kit.md §7.111: a help bar's own picture and a page of a PDF on the canvas are decoded at the pixels they cover,
// read at two window heights in one run, the window resized between them; the heights are test inputs.

use emusen_qt_test::Harness;

const SCENE: &str = "qrc:/qt/qml/EmuSen/Scenes/qml/DrawnPixelsScene.qml";

/// One page of `w` by `h` points filled grey, its cross-reference offsets counted as the objects are written.
fn one_page(w: u32, h: u32) -> Vec<u8> {
    let content = format!("0.5 g 0 0 {w} {h} re f");
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_owned(),
        format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {w} {h}] /Contents 4 0 R >>"),
        format!("<< /Length {} >>\nstream\n{content}\nendstream", content.len()),
    ];
    let mut out = b"%PDF-1.4\n".to_vec();
    let mut at = Vec::new();
    for (n, body) in objects.iter().enumerate() {
        at.push(out.len());
        out.extend(format!("{} 0 obj\n{body}\nendobj\n", n + 1).bytes());
    }
    let xref = out.len();
    out.extend(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).bytes());
    at.iter().for_each(|o| out.extend(format!("{o:010} 00000 n \n").bytes()));
    out.extend(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objects.len() + 1).bytes());
    out
}

#[test]
fn kit_7_111_a_help_picture_and_a_pdf_page_are_decoded_at_the_pixels_they_cover() {
    let dir = std::env::temp_dir().join(format!("kit-drawn-pixels-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let pdf = dir.join("manual.pdf");
    std::fs::write(&pdf, one_page(600, 800)).unwrap();

    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 1280, 800);
    assert!(scene.set_property("scene", "manual", &format!("file://{}", pdf.display())));
    // Two window sizes, each a test input: the canvas's own height, then half again.
    for (width, height) in [(1280, 800), (1920, 1200)] {
        scene.resize(width, height);
        scene.wait(50);
        let tree = scene.tree();
        let icon = tree.find_named("icon-Menu").map(|i| tree.nodes[i as usize].clone()).expect("the theme's picture in the tree");
        let page = tree.nodes.iter().find(|n| n.cls.contains("PdfPageImage")).cloned().expect("the page's picture in the tree");
        for (what, node) in [("the help picture", &icon), ("the page", &page)] {
            let decoded = (scene.item_number(&node.id, "implicitWidth"), scene.item_number(&node.id, "implicitHeight"));
            // The window's ratio is one device pixel a window pixel under the harness, so a scene pixel is a device pixel.
            let drawn = (node.w.ceil(), node.h.ceil());
            assert!(
                (decoded.0 - drawn.0).abs() <= 1.0 && (decoded.1 - drawn.1).abs() <= 1.0,
                "{what} at {width} x {height}: decoded {decoded:?}, drawn over {drawn:?} pixels"
            );
        }
    }
    scene.assert_quiet();
    std::fs::remove_dir_all(&dir).ok();
}
