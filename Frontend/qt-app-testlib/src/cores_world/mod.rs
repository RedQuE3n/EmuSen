//! A test world whose library lists a chosen number of installed cores, as the splash's planets count them (EmuSen_Qt.md §3.4.8, the splash).

use emusen_client_fixture::{cores, library, World};
use emusen_crystal_client::{Api, Config, Runtime};
use emusen_qt_models::clients::ClientSource;
use emusen_qt_models::library::{installed_cores, LibrarySource};

/// The cores the program lists in this world, read as its library reads them, on the runtime's default folders.
pub fn listed() -> Vec<String> {
    let source = ClientSource::open(library().to_str().unwrap(), &Config::default()).unwrap();
    installed_cores(&source.systems().unwrap())
}

/// The test core approved in the world's /Cores as the Cores window approves one, then copies of its sidecar under other ids and names until the program lists `want` cores.
pub fn install(world: &World, want: usize) {
    let player = world.dir("Home/Cores");
    let core = world.player_core(&player);
    let rt = Runtime::new(&Api::load(library()).unwrap(), &Config::default()).unwrap();
    rt.core_approve(&cores::sha256(&core)).unwrap();
    drop(rt);
    let have = listed();
    assert!(have.len() <= want, "the world lists {} cores before any copy: {have:?}", have.len());
    eprintln!("listed before the copies: {have:?}");
    let sidecar = std::path::PathBuf::from(format!("{}.core.json", core.display()));
    let read: serde_json::Value = serde_json::from_slice(&std::fs::read(&sidecar).expect("the approval wrote the core's sidecar")).unwrap();
    let (id, name) = (read["info"]["id"].as_str().unwrap(), read["info"]["display_name"].as_str().or(read["info"]["name"].as_str()).unwrap());
    for i in 1..=want - have.len() {
        let mut copy = read.clone();
        copy["info"]["id"] = serde_json::json!(format!("{id}-{i}"));
        copy["info"]["display_name"] = serde_json::json!(format!("{name} {i}"));
        std::fs::write(player.join(format!("copy-{i}.core.json")), copy.to_string()).unwrap();
    }
    let now = listed();
    assert_eq!(now.len(), want, "the program lists {now:?}");
}
