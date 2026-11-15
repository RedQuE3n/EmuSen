// EmuSen_Kit.md §7.110: a page turn of the kit's PdfPage is decoded off the calling thread: in the same turn of the event
// loop the page's picture is still loading, and the turned page is drawn once it is ready.

use cxx_qt_lib::QImage;
use emusen_qt_test::{Harness, Scene};

const SCENE: &str = "qrc:/qt/qml/EmuSen/Scenes/qml/PdfTurnScene.qml";
// Test inputs: a red first page, and a blue second page carrying enough marks that its decode is not instant.
const MARKS: usize = 20000;

/// Two pages of `w` by `h` points: the first filled red, the second blue with `marks` small squares over it.
fn two_pages(w: u32, h: u32, marks: usize) -> Vec<u8> {
    let first = format!("1 0 0 rg 0 0 {w} {h} re f");
    let mut second = format!("0 0 1 rg 0 0 {w} {h} re f\n");
    for i in 0..marks {
        let (x, y) = ((i * 37) % w as usize, (i * 53) % h as usize);
        second.push_str(&format!("0 0 {} rg {x} {y} 9 9 re f\n", 0.5 + (i % 50) as f64 / 100.0));
    }
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Kids [3 0 R 5 0 R] /Count 2 >>".to_owned(),
        format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {w} {h}] /Contents 4 0 R >>"),
        format!("<< /Length {} >>\nstream\n{first}\nendstream", first.len() + 1),
        format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {w} {h}] /Contents 6 0 R >>"),
        format!("<< /Length {} >>\nstream\n{second}endstream", second.len()),
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

// The page's picture's status, as Qt's Image numbers it (1 Ready, 2 Loading); NaN before the picture is made.
fn status(scene: &mut Scene<'_>) -> f64 {
    let tree = scene.tree();
    let id = tree.find_named("manualPage").and_then(|p| tree.under(p).into_iter().map(|i| tree.nodes[i as usize].id.clone()).find(|id| tree.nodes.iter().any(|n| &n.id == id && n.picture)));
    id.map_or(f64::NAN, |id| scene.item_number(&id, "status"))
}

// The window's centre pixel, read back from a grab through Qt's image.
fn centre(scene: &mut Scene<'_>, dir: &std::path::Path) -> [u8; 3] {
    let path = dir.join("grab.png");
    assert!(scene.grab(&path.to_string_lossy()));
    let image = QImage::from_data(&std::fs::read(&path).unwrap(), None).expect("the grab reads");
    let c = image.pixel_color(image.width() / 2, image.height() / 2);
    [c.red() as u8, c.green() as u8, c.blue() as u8]
}

fn until(scene: &mut Scene<'_>, what: &str, ok: impl Fn(&mut Scene<'_>) -> bool) {
    let mut waited = 0;
    while !ok(scene) {
        assert!(waited < 20000, "{what} within 20 s");
        scene.wait(5);
        waited += 5;
    }
}

#[test]
fn kit_7_110_a_page_turn_is_decoded_off_the_calling_thread_and_drawn_when_ready() {
    let dir = std::env::temp_dir().join(format!("kit-pdf-turn-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let pdf = dir.join("manual.pdf");
    std::fs::write(&pdf, two_pages(640, 480, MARKS)).unwrap();
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 640, 480);
    assert!(scene.set_property("scene", "manual", &format!("file://{}", pdf.display())));
    until(&mut scene, "the first page drawn", |s| status(s) == 1.0);
    until(&mut scene, "the first page red", |s| centre(s, &dir) == [255, 0, 0]);

    // The turn, then the picture read before the event loop runs again: a decode on this thread would already be done.
    assert!(scene.set_property_now("scene", "page", "1"));
    let now = status(&mut scene);
    let drawn_now = centre(&mut scene, &dir);
    // Measured on 2026-11-15, not asserted: nothing is drawn while the page loads, as trunk's PdfPageImage drew nothing.
    eprintln!("in the turn: status {now}, centre {drawn_now:?}");
    assert_eq!(now, 2.0, "the turned page is still loading in the turn that asked for it");
    until(&mut scene, "the second page ready", |s| status(s) == 1.0);
    until(&mut scene, "the second page blue", |s| centre(s, &dir)[2] > 200 && centre(s, &dir)[0] < 50);
    scene.assert_quiet();
    std::fs::remove_dir_all(&dir).ok();
}
