// EmuSen_Kit.md §7.110: pages of many PDF files, made and then destroyed all at once while their pages are still being
// drawn, as a manual closing while a page is drawn; the process lives and Qt says nothing. A test file of its own, so a
// crash is this test's failure alone.

use emusen_qt_test::Harness;

const SCENE: &str = "qrc:/qt/qml/EmuSen/Scenes/qml/PdfTeardownScene.qml";
// Test inputs: many files of light pages, so the image thread starts a page often (it reads the file then, outside
// Qt PDF's lock), and teardown repeated, so no round's timing decides the result.
const FILES: usize = 48;
const MARKS: usize = 50;
const ROUNDS: usize = 20;

/// One page of `w` by `h` points covered in `marks` small squares of changing grey, its offsets counted as written.
fn busy_page(w: u32, h: u32, marks: usize) -> Vec<u8> {
    let mut content = String::new();
    for i in 0..marks {
        let (x, y) = ((i * 37) % w as usize, (i * 53) % h as usize);
        content.push_str(&format!("{} g {x} {y} 9 9 re f\n", (i % 100) as f64 / 100.0));
    }
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_owned(),
        format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {w} {h}] /Contents 4 0 R >>"),
        format!("<< /Length {} >>\nstream\n{content}endstream", content.len()),
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

// The status of each page's picture in the tree (the scene's only pictures), as Qt's Image numbers it (1 is Ready).
fn statuses(scene: &mut emusen_qt_test::Scene<'_>) -> Vec<f64> {
    let tree = scene.tree();
    let ids: Vec<String> = tree.nodes.iter().filter(|n| n.picture).map(|n| n.id.clone()).collect();
    ids.iter().map(|id| scene.item_number(id, "status")).collect()
}

#[test]
fn kit_7_110_pages_destroyed_while_others_are_drawn_leave_the_process_running() {
    let dir = std::env::temp_dir().join(format!("kit-pdf-teardown-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    // A file a page a round: a page drawn before is in Qt's picture cache and is never drawn again.
    for round in 0..ROUNDS {
        for n in 0..FILES {
            std::fs::write(dir.join(format!("{round}-{n}.pdf")), busy_page(1000, 1000, MARKS)).unwrap();
        }
    }
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 1280, 800);
    assert!(scene.set_property("scene", "folder", &dir.to_string_lossy()));
    for round in 0..ROUNDS {
        assert!(scene.set_property("scene", "round", &round.to_string()));
        assert!(scene.set_property_now("scene", "count", &FILES.to_string()));
        // Destroyed the moment the first page is drawn: a page not yet drawn is being drawn, or waits, as its document goes.
        let mut waited = 0;
        let at_teardown = loop {
            let s = statuses(&mut scene);
            if s.iter().any(|v| *v == 1.0) {
                break s;
            }
            assert!(waited < 20000, "a page drawn within 20 s in round {round}: {s:?}");
            scene.wait(1);
            waited += 1;
        };
        eprintln!("round {round}: pages drawn when destroyed {} of {}", at_teardown.iter().filter(|v| **v == 1.0).count(), at_teardown.len());
        assert!(scene.set_property_now("scene", "count", "0"));
        scene.wait(300);
    }
    assert!(statuses(&mut scene).is_empty(), "every page destroyed");
    scene.assert_quiet();
    std::fs::remove_dir_all(&dir).ok();
}
