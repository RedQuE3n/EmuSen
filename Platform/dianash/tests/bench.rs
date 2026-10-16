//! A line's round trip through the shell core alone, as a host's submit and take make it: run with
//! `cargo test --release -p emusen-dianash --test bench -- --ignored --nocapture`. See EmuSen_DianaSh.md §13.7.

use emusen_dianash::shell::{Config, Shell};
use std::time::Instant;

#[test]
#[ignore]
fn a_lines_round_trip_is_measured() {
    let mut shell = Shell::new(Config { sessions: false, root: Some(std::env::temp_dir().to_string_lossy().into_owned()), ..Config::default() });
    println!("Rust shell core; 2,000 runs after 200 of warm-up, median (95th percentile) in microseconds");
    for line in ["true", "echo hello", "echo a b c | echo d", "for i in 0 1 2 3 4 5 6 7 8 9; do echo $i | echo 1; done"] {
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
