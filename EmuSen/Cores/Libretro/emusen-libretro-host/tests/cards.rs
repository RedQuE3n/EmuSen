// A core with a PlayStation 2 card profile: the test core laid under PCEE2's stem, its cards Sapphire's files (EmuSen_LibretroHost.md §10j).

use std::path::PathBuf;
use std::sync::OnceLock;

use emusen_core_host::library::Library;
use emusen_core_host::machine::{Create, Machine};
use emusen_native::core::sys::event;
use emusen_ps2_card::{CARD_LENGTH, Card, ERASED, MEDIUM, SUFFIXES, Time};

static LIB: OnceLock<(&'static Library, PathBuf)> = OnceLock::new();

const STEM: &str = "pcee2";
const T: Time = Time { sec: 1, min: 2, hour: 3, day: 1, month: 11, year: 2026 };

fn suffix() -> &'static str {
    emusen_core_entry::library_suffix()
}

/// The entry folder of the test core under PCEE2's stem, as Cores lays one out, and the library loaded.
fn library() -> (&'static Library, PathBuf) {
    let (lib, root) = LIB.get_or_init(|| {
        let deps = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("libretro-cards-{}", std::process::id()));
        let entry = root.join("libretro").join(STEM);
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&entry).unwrap();
        let host = entry.join(format!("emusen-libretro-host.{}", suffix()));
        std::fs::copy(deps.join(format!("libemusen_libretro_host.{}", suffix())), &host).unwrap();
        std::fs::copy(deps.join(format!("libemusen_libretro_testcore.{}", suffix())), entry.join(format!("{STEM}_libretro.{}", suffix()))).unwrap();
        std::fs::write(entry.join(format!("{STEM}_libretro.info")), "display_name = \"EmuSen - Test Core\"\ncorename = \"TestCore\"\nlicense = \"GPLv3\"\nauthors = \"EmuSen\"\nsystemid = \"\"\ndatabase = \"\"\nfirmware_count = 2\nfirmware0_path = \"pcsx2/bios\"\nfirmware0_opt = \"false\"\nfirmware1_path = \"pcsx2/resources\"\nfirmware1_opt = \"false\"\n").unwrap();
        let lib = Library::load(&host).unwrap_or_else(|e| panic!("{e:?}"));
        (lib, root.join("libretro"))
    });
    (*lib, root.clone())
}

fn make(files: &[(u32, &[u8])]) -> Machine {
    let (lib, _) = library();
    let c = Create { settings: "", pixel_formats: 0b011, files, ..Create::new(b"TCG1 a test game") };
    Machine::create(lib, &c).unwrap_or_else(|r| panic!("{r:?}"))
}

fn formatted() -> Vec<u8> {
    let mut card = vec![ERASED; CARD_LENGTH];
    emusen_ps2_card::format(&mut card, T).unwrap();
    card
}

fn saves(image: &[u8]) -> Vec<String> {
    let card = Card::open(image).unwrap();
    emusen_ps2_card::saves(&card).map(|s| String::from_utf8_lossy(s.unwrap().1.name()).into_owned()).collect()
}

#[test]
fn the_cards_are_the_card_formats_files_a_card_not_passed_is_blank_and_a_write_comes_back() {
    let (lib, root) = library();
    let info = lib.info_text().unwrap();
    for (n, which) in [2u32, 3].into_iter().enumerate() {
        let entry = format!(r#""which":{which},"suffix":"{}""#, SUFFIXES[n]);
        assert!(info.contains(&entry), "{entry} in {info}");
    }
    assert_eq!(info.matches(&format!(r#""medium":"{MEDIUM}""#)).count(), 2, "{info}");
    let numbers = |from: &str, to: &str| -> Vec<u32> {
        let part = info.split(&format!("\"{from}\":[")).nth(1).unwrap().split(&format!("\"{to}\":")).next().unwrap();
        part.split("\"which\":").skip(1).map(|w| w.split(|c: char| !c.is_ascii_digit()).next().unwrap().parse().unwrap()).collect()
    };
    let (firmware, battery) = (numbers("firmware", "battery"), numbers("battery", "capabilities"));
    // PCEE2 carries its pcsx2/resources, so only pcsx2/bios is firmware (EmuSen_LibretroHost.md §10s).
    assert!(firmware.len() == 1 && firmware.iter().all(|f| !battery.contains(f)), "firmware {firmware:?} and battery files {battery:?} share no number");

    let cards = root.join(".data").join(STEM).join("system/pcsx2/memcards");
    std::fs::create_dir_all(&cards).unwrap();
    let mut left = formatted();
    emusen_ps2_card::mkdir(&mut left, b"", b"BASLUS-20001LEFT", T).unwrap();
    std::fs::write(cards.join("Mcd001.ps2"), &left).unwrap();
    drop(make(&[]));
    for file in ["Mcd001.ps2", "Mcd002.ps2"] {
        let bytes = std::fs::read(cards.join(file)).unwrap();
        assert!(bytes.len() == CARD_LENGTH && bytes.iter().all(|&b| b == ERASED), "{file}: a card not passed is a new, blank card, not the one left there");
    }

    let card = formatted();
    let mut m = make(&[(2, &card), (3, &[])]);
    assert_eq!(std::fs::read(cards.join("Mcd001.ps2")).unwrap(), card, "the card passed is the core's slot 1");
    assert!(std::fs::read(cards.join("Mcd002.ps2")).unwrap().iter().all(|&b| b == ERASED), "an empty file is a blank card");
    assert!(!root.join(".data").join(STEM).join("system/pcsx2/resources").exists(), "a card is never also written as a firmware file");
    m.advance().unwrap();
    let mut written = card.clone();
    emusen_ps2_card::mkdir(&mut written, b"", b"BASLUS-20001TEST", T).unwrap();
    std::fs::write(cards.join("Mcd001.ps2"), &written).unwrap();
    let mut seen = false;
    for _ in 0..30 {
        m.advance().unwrap();
        seen |= m.events().unwrap().iter().any(|e| e.kind == event::BATTERY && e.a == 2);
    }
    assert!(seen, "the card's change is reported");
    let mut got = Vec::new();
    m.battery(2, &mut got).unwrap();
    assert_eq!(saves(&got), ["BASLUS-20001TEST"], "the card the core wrote, read by the card format's reader");
}
