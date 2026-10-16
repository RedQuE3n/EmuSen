//! A line's round trip through the shell core alone, as a host's submit and take make it: run with
//! `cargo test --release -p emusen-dianash --test bench -- --ignored --nocapture`. See EmuSen_DianaSh.md §13.7.

use emusen_dianash::abi::{dianash_bytes, dianash_call, dianash_host_commands, dianash_result};
use emusen_dianash::shell::{Config, Shell};
use std::ffi::c_void;
use std::time::Instant;

unsafe extern "C" fn nothing(_: *mut c_void, _: *const dianash_call, _: *mut dianash_result) -> i32 {
    0
}

/// A host's 70 commands, as many as the C# frontends register, none of them run: a table the size a host's is.
fn with_host_commands(shell: &mut Shell) {
    let commands: Vec<String> = (0..70).map(|i| format!(r#"{{"name":"hostcommand{i}","usage":"  hostcommand{i}","read_only":false}}"#)).collect();
    let manifest = format!(r#"{{"manifest":1,"id":"bench.host","kind":"host","commands":[{}]}}"#, commands.join(","));
    let host = dianash_host_commands { size: std::mem::size_of::<dianash_host_commands>() as u32, manifest: dianash_bytes::of(manifest.as_bytes()), ctx: std::ptr::null_mut(), run: Some(nothing), jobs: None, kill: None };
    shell.add_commands(0, &host).unwrap();
}

#[test]
#[ignore]
fn a_lines_round_trip_is_measured() {
    // DIANASH_BENCH_TREE takes the root from Galaxia's tree, as a host's shell does, rather than naming it.
    let root = std::env::var_os("DIANASH_BENCH_TREE").is_none().then(|| std::env::temp_dir().to_string_lossy().into_owned());
    let mut shell = Shell::new(Config { sessions: false, root, ..Config::default() });
    shell.add_module(&dianash_text::MODULE).unwrap();
    let hosted = std::env::var_os("DIANASH_BENCH_HOSTED").is_some();
    if hosted {
        with_host_commands(&mut shell);
    }
    println!("Rust shell core with dianash.text{}; 2,000 runs after 200 of warm-up, median (95th percentile) in microseconds", if hosted { " and a host's 70 commands" } else { "" });
    for line in ["true", "echo hello", "echo a b c | grep b | wc -l", "for i in 0 1 2 3 4 5 6 7 8 9; do echo $i | grep 1; done"] {
        for _ in 0..200 {
            shell.submit(0, line).unwrap();
            shell.take_output();
        }
        let mut times: Vec<f64> = (0..2000)
            .map(|_| {
                let start = Instant::now();
                shell.submit(0, line).unwrap();
                shell.take_output();
                start.elapsed().as_secs_f64() * 1e6
            })
            .collect();
        times.sort_by(f64::total_cmp);
        println!("  {line}: {:.2} ({:.2})", times[1000], times[1900]);
    }
}
