//! VenusRT as the runtime finds it through the core ABI: its capabilities, the images it claims, and a state it cannot
//! read refused in words with the machine unchanged; moved from WiseMan's VenusRtCoreAbiTests on 2026-11-06.

mod common;

use common::golden::library;
use common::{differential, programs};
use emusen_core_host::Library;
use emusen_native::core::sys::caps;
use serde_json::Value;

#[test]
fn venusrt_claims_the_snes_images_and_its_capabilities() {
    let lib = Library::load(&library()).unwrap();
    let info = serde_json::from_str::<Value>(&lib.info_text().unwrap()).unwrap();
    let systems = info.get("systems").and_then(Value::as_array).unwrap();
    assert_eq!(1, systems.len());
    let extensions: Vec<&str> = systems[0].get("extensions").and_then(Value::as_array).unwrap().iter().filter_map(Value::as_str).collect();
    assert_eq!(vec![".smc", ".sfc"], extensions);
    let wanted = caps::RESET | caps::SNAPSHOT | caps::BATTERY_DIRTY | caps::ROM_PATCHES | caps::CHEAT_POKES | caps::SETTINGS | caps::DEBUG | caps::DEBUG_STACK | caps::DEBUG_REGISTERS | caps::DEBUG_DISASSEMBLE;
    assert_eq!(wanted, lib.capabilities());
}

/// VenusRT_Plan.md §4.5: a state C# Venus wrote, known by its magic "SNES", is refused in words naming Venus (C#), and the machine is unchanged.
#[test]
fn a_venus_state_is_refused_naming_the_engine_that_made_it() {
    let mut m = differential::venusrt_machine(&programs::board(&[]), None, None).unwrap();
    for _ in 0..30 {
        m.advance().unwrap();
    }
    let mut before = vec![0u8; m.state_size(0).unwrap()];
    m.state_save(0, &mut before).unwrap();
    let mut venus = vec![0u8; 4096];
    venus[..4].copy_from_slice(b"SNES");
    let refused = m.state_load(&venus).unwrap_err();
    assert_eq!("it was saved by Venus (C#), the C# SNES engine, whose states VenusRT cannot read", m.words(refused as i64));
    let mut after = vec![0u8; before.len()];
    m.state_save(0, &mut after).unwrap();
    assert_eq!(before, after);
    let mut other = before.clone();
    other[0] ^= 1;
    let refused = m.state_load(&other).unwrap_err();
    assert_eq!("not this core's state", m.words(refused as i64));
}
