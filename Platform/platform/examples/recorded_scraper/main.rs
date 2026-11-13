//! The recorded-response ScreenScraper as a program, for a test outside this workspace: it prints the scraper's `test`
//! settings member, `{"connect", "roots"}`, as its first line, then each path it is asked, `asked <path>`, a line each,
//! and ends when its input closes (EmuSen_LibraryTools.md §8).

mod server;

use std::io::{Read, Write};
use std::time::Duration;

fn main() {
    let (port, pem, asked) = server::serve();
    println!("{}", serde_json::json!({ "connect": format!("127.0.0.1:{port}"), "roots": pem }));
    let _ = std::io::stdout().flush();
    std::thread::spawn(move || {
        let mut told = 0;
        loop {
            let now: Vec<String> = asked.lock().unwrap_or_else(|p| p.into_inner())[told..].to_vec();
            for path in &now {
                println!("asked {path}");
            }
            told += now.len();
            let _ = std::io::stdout().flush();
            std::thread::sleep(Duration::from_millis(20));
        }
    });
    let _ = std::io::stdin().read_to_end(&mut Vec::new());
}
