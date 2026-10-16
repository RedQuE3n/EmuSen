//! The Rust table held to Crystal's `emusen_session.h`: every field's offset and the table's size as a C compiler
//! lays them out, and every number the header defines. See EmuSen_DianaSh.md §13.11.

use emusen_session_service::abi::{self, emusen_session_v1};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

fn include() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../EmuSen.Crystal/include")
}

fn header() -> String {
    std::fs::read_to_string(include().join("emusen_session.h")).expect("emusen_session.h")
}

/// `#define EMUSEN_* <number>` lines, with their `u` suffix and parentheses removed; the composed ones left out.
fn numbers() -> BTreeMap<String, i64> {
    let mut found = BTreeMap::new();
    for line in header().lines() {
        let Some(rest) = line.trim().strip_prefix("#define ") else { continue };
        let mut words = rest.split_whitespace();
        let (Some(name), Some(value)) = (words.next(), words.next()) else { continue };
        let value = value.trim_matches(['(', ')']).trim_end_matches('u');
        if let Ok(number) = value.parse::<i64>() {
            found.insert(name.to_string(), number);
        }
    }
    found
}

#[test]
fn every_number_is_the_header_s() {
    use abi::*;
    let rust: Vec<(&str, i64)> = vec![
        ("EMUSEN_SESSION_MAJOR", MAJOR.into()),
        ("EMUSEN_SESSION_MINOR", MINOR.into()),
        ("EMUSEN_SVC_NULL", status::NULL.into()),
        ("EMUSEN_SVC_BAD_STRING", status::BAD_STRING.into()),
        ("EMUSEN_SVC_NO_SESSION", status::NO_SESSION.into()),
        ("EMUSEN_SVC_NO_SUCH_SPACE", status::NO_SUCH_SPACE.into()),
        ("EMUSEN_SVC_READ_ONLY", status::READ_ONLY.into()),
        ("EMUSEN_SVC_NOT_SUPPORTED", status::NOT_SUPPORTED.into()),
        ("EMUSEN_SVC_BAD_ARGUMENT", status::BAD_ARGUMENT.into()),
        ("EMUSEN_SVC_NO_SUCH_ID", status::NO_SUCH_ID.into()),
        ("EMUSEN_SVC_PARSE", status::PARSE.into()),
        ("EMUSEN_SVC_ABSENT", status::ABSENT.into()),
        ("EMUSEN_SVC_FAULT", status::FAULT.into()),
        ("EMUSEN_SESSION_PROCESSOR_SESSION", PROCESSOR_SESSION.into()),
        ("EMUSEN_SESSION_RUN_PAUSE", run::PAUSE.into()),
        ("EMUSEN_SESSION_RUN_RESUME", run::RESUME.into()),
        ("EMUSEN_SESSION_RUN_STEP", run::STEP.into()),
        ("EMUSEN_SESSION_RUN_STEP_OVER", run::STEP_OVER.into()),
        ("EMUSEN_SESSION_RUN_STEP_OUT", run::STEP_OUT.into()),
        ("EMUSEN_SESSION_RUN_TO_ADDRESS", run::TO_ADDRESS.into()),
        ("EMUSEN_SESSION_RUN_TO_INTERRUPT", run::TO_INTERRUPT.into()),
        ("EMUSEN_SESSION_RUN_TO_SCANLINE", run::TO_SCANLINE.into()),
        ("EMUSEN_SESSION_RUN_TO_FRAME", run::TO_FRAME.into()),
        ("EMUSEN_SESSION_FRAME_CALL", frame::CALL.into()),
        ("EMUSEN_SESSION_INTERRUPT_NMI", frame::NMI.into()),
        ("EMUSEN_SESSION_INTERRUPT_IRQ", frame::IRQ.into()),
        ("EMUSEN_SESSION_INTERRUPT_BRK", frame::BRK.into()),
        ("EMUSEN_SESSION_INTERRUPT_COP", frame::COP.into()),
        ("EMUSEN_SESSION_WATCH_WRITE", watch::WRITE.into()),
        ("EMUSEN_SESSION_WATCH_READ", watch::READ.into()),
        ("EMUSEN_SESSION_WATCH_BOTH", watch::BOTH.into()),
        ("EMUSEN_SESSION_CONDITION_LOG_ONLY", condition::LOG_ONLY.into()),
        ("EMUSEN_SESSION_CONDITION_DISARM", condition::DISARM.into()),
        ("EMUSEN_SESSION_COVERAGE_ARM", coverage::ARM.into()),
        ("EMUSEN_SESSION_COVERAGE_DISARM", coverage::DISARM.into()),
        ("EMUSEN_SESSION_COVERAGE_CLEAR", coverage::CLEAR.into()),
        ("EMUSEN_SESSION_COVERAGE_MARK", coverage::MARK.into()),
        ("EMUSEN_SESSION_COVERAGE_CLEAR_MARK", coverage::CLEAR_MARK.into()),
        ("EMUSEN_SESSION_COVERAGE_STATUS", coverage::STATUS.into()),
        ("EMUSEN_SESSION_COVERAGE_QUERY", coverage::QUERY.into()),
        ("EMUSEN_SESSION_COVERAGE_NEW_SINCE_MARK", coverage::NEW_SINCE_MARK.into()),
        ("EMUSEN_SESSION_COVERAGE_ENTRY_POINTS", coverage::ENTRY_POINTS.into()),
        ("EMUSEN_SESSION_COVERAGE_EXPORT", coverage::EXPORT.into()),
        ("EMUSEN_SESSION_COVERAGE_IMPORT", coverage::IMPORT.into()),
        ("EMUSEN_SESSION_COVERAGE_MERGE", coverage::MERGE.into()),
        ("EMUSEN_SESSION_PROFILE_ARM", profile::ARM.into()),
        ("EMUSEN_SESSION_PROFILE_DISARM", profile::DISARM.into()),
        ("EMUSEN_SESSION_PROFILE_CLEAR", profile::CLEAR.into()),
        ("EMUSEN_SESSION_PROFILE_STATUS", profile::STATUS.into()),
        ("EMUSEN_SESSION_PROFILE_HOTTEST", profile::HOTTEST.into()),
        ("EMUSEN_SESSION_PROFILE_RESET_STACK", profile::RESET_STACK.into()),
        ("EMUSEN_SESSION_COUNTERS_ARM", counters::ARM.into()),
        ("EMUSEN_SESSION_COUNTERS_DISARM", counters::DISARM.into()),
        ("EMUSEN_SESSION_COUNTERS_CLEAR", counters::CLEAR.into()),
        ("EMUSEN_SESSION_COUNTERS_STATUS", counters::STATUS.into()),
        ("EMUSEN_SESSION_COUNTERS_AT", counters::AT.into()),
        ("EMUSEN_SESSION_COUNTERS_TOTALS", counters::TOTALS.into()),
        ("EMUSEN_SESSION_COUNTERS_HOTTEST", counters::HOTTEST.into()),
        ("EMUSEN_SESSION_COUNTERS_UNTOUCHED", counters::UNTOUCHED.into()),
        ("EMUSEN_SESSION_DMA_LOG_ARM", dma_log::ARM.into()),
        ("EMUSEN_SESSION_DMA_LOG_DISARM", dma_log::DISARM.into()),
        ("EMUSEN_SESSION_DMA_LOG_CLEAR", dma_log::CLEAR.into()),
        ("EMUSEN_SESSION_DMA_LOG_STATUS", dma_log::STATUS.into()),
        ("EMUSEN_SESSION_DMA_LOG_TAIL", dma_log::TAIL.into()),
        ("EMUSEN_SESSION_DMA_LOG_CHANNELS", dma_log::CHANNELS.into()),
        ("EMUSEN_SESSION_DMA_LOG_DESTINATIONS", dma_log::DESTINATIONS.into()),
        ("EMUSEN_SESSION_FLOW_ARM", flow::ARM.into()),
        ("EMUSEN_SESSION_FLOW_DISARM", flow::DISARM.into()),
        ("EMUSEN_SESSION_FLOW_CLEAR", flow::CLEAR.into()),
        ("EMUSEN_SESSION_FLOW_STATUS", flow::STATUS.into()),
        ("EMUSEN_SESSION_FLOW_TAIL", flow::TAIL.into()),
        ("EMUSEN_SESSION_FLOW_STATS", flow::STATS.into()),
        ("EMUSEN_SESSION_CHEAT_LIST", cheat::LIST.into()),
        ("EMUSEN_SESSION_CHEAT_ADD", cheat::ADD.into()),
        ("EMUSEN_SESSION_CHEAT_POKE", cheat::POKE.into()),
        ("EMUSEN_SESSION_CHEAT_ROM_PATCH", cheat::ROM_PATCH.into()),
        ("EMUSEN_SESSION_CHEAT_ENABLE", cheat::ENABLE.into()),
        ("EMUSEN_SESSION_CHEAT_DISABLE", cheat::DISABLE.into()),
        ("EMUSEN_SESSION_CHEAT_REMOVE", cheat::REMOVE.into()),
        ("EMUSEN_SESSION_CHEAT_CLEAR", cheat::CLEAR.into()),
        ("EMUSEN_SESSION_CHEAT_MASTER", cheat::MASTER.into()),
        ("EMUSEN_SESSION_CHEAT_EXPORT", cheat::EXPORT.into()),
        ("EMUSEN_SESSION_CHEAT_IMPORT", cheat::IMPORT.into()),
        ("EMUSEN_SESSION_CHEAT_DECODE", cheat::DECODE.into()),
        ("EMUSEN_SESSION_CHEAT_DB_SYSTEMS", cheat::DB_SYSTEMS.into()),
        ("EMUSEN_SESSION_CHEAT_DB_GAMES", cheat::DB_GAMES.into()),
        ("EMUSEN_SESSION_CHEAT_DB_FIND", cheat::DB_FIND.into()),
        ("EMUSEN_SESSION_CHEAT_CLAIMS", cheat::CLAIMS.into()),
        ("EMUSEN_SESSION_CHEAT_CHT_PARSE", cheat::CHT_PARSE.into()),
        ("EMUSEN_SESSION_CHEAT_CHT_EXPORT", cheat::CHT_EXPORT.into()),
        ("EMUSEN_SESSION_CHEAT_FILES", cheat::FILES.into()),
        ("EMUSEN_SESSION_CHEAT_FILE_READ", cheat::FILE_READ.into()),
        ("EMUSEN_SESSION_CHEAT_FILE_WRITE", cheat::FILE_WRITE.into()),
        ("EMUSEN_SESSION_CHEAT_DB_STATUS", cheat::DB_STATUS.into()),
        ("EMUSEN_SESSION_CHEAT_DB_PARSE", cheat::DB_PARSE.into()),
        ("EMUSEN_SESSION_CHEAT_DB_PRUNE", cheat::DB_PRUNE.into()),
        ("EMUSEN_SESSION_CHEAT_DB_UPDATE", cheat::DB_UPDATE.into()),
        ("EMUSEN_SESSION_TELEMETRY_CPU", telemetry::CPU.into()),
        ("EMUSEN_SESSION_TELEMETRY_VIDEO", telemetry::VIDEO.into()),
        ("EMUSEN_SESSION_TELEMETRY_APU", telemetry::APU.into()),
        ("EMUSEN_SESSION_TELEMETRY_COPROCESSOR", telemetry::COPROCESSOR.into()),
        ("EMUSEN_SESSION_TELEMETRY_SPRITES", telemetry::SPRITES.into()),
        ("EMUSEN_SESSION_TELEMETRY_PALETTES", telemetry::PALETTES.into()),
        ("EMUSEN_SESSION_TELEMETRY_AUDIO_CHANNELS", telemetry::AUDIO_CHANNELS.into()),
        ("EMUSEN_SESSION_TELEMETRY_HARDWARE_LOAD", telemetry::HARDWARE_LOAD.into()),
        ("EMUSEN_SESSION_EVENT_RUN", event::RUN.into()),
    ];
    let rust: BTreeMap<&str, i64> = rust.into_iter().collect();
    let header = numbers();
    assert_eq!(header.keys().map(String::as_str).collect::<Vec<_>>(), rust.keys().copied().collect::<Vec<_>>(), "every number has a Rust constant");
    for (name, value) in &rust {
        assert_eq!(header[*name], *value, "{name}");
    }
    assert!(header.contains_key("EMUSEN_SESSION_MAJOR") && SERVICE == "emusen.session");
}

#[test]
fn the_table_is_laid_out_as_a_c_compiler_lays_it_out() {
    let compiler = std::env::var("CC").unwrap_or_else(|_| "cc".to_string());
    let dir = std::env::temp_dir().join(format!("emusen-session-layout-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut probe = String::from("#include <stdio.h>\n#include <stddef.h>\n#include \"emusen_session.h\"\nint main(void) {\n");
    probe += "    printf(\"sizeof %zu\\n\", sizeof(emusen_session_v1));\n";
    for (field, _) in abi::offsets() {
        probe += &format!("    printf(\"{field} %zu\\n\", offsetof(emusen_session_v1, {field}));\n");
    }
    probe += "    return 0;\n}\n";
    std::fs::write(dir.join("probe.c"), probe).unwrap();
    let Ok(built) = Command::new(&compiler).args(["-std=c99", "-Wall", "-Wextra", "-Werror", "-pedantic", "-I"]).arg(include()).arg(dir.join("probe.c")).arg("-o").arg(dir.join("probe")).output() else {
        eprintln!("no C compiler: the layout was not compared");
        return;
    };
    assert!(built.status.success(), "the header does not compile cleanly:\n{}", String::from_utf8_lossy(&built.stderr));
    let ran = Command::new(dir.join("probe")).output().unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    let c: BTreeMap<String, usize> = String::from_utf8_lossy(&ran.stdout).lines().map(|l| l.split_once(' ').unwrap()).map(|(k, v)| (k.to_string(), v.parse().unwrap())).collect();
    assert_eq!(c["sizeof"], std::mem::size_of::<emusen_session_v1>(), "the table's size");
    for (field, offset) in abi::offsets() {
        assert_eq!(c[field], offset, "{field}");
    }
    let header_fields = header().lines().filter(|l| l.trim_start().starts_with("int32_t (*") || l.trim_start().starts_with("int64_t (*")).count();
    assert_eq!(header_fields + 3, abi::offsets().len(), "every function of the header is a field here");
}
