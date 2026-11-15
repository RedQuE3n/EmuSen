//! A cheat database's archive served on the loopback, for a test outside this workspace: the files under the folder
//! named by its argument zipped by Galaxia's writer, served whole at `/cheats.zip` and, at `/held/cheats.zip`, its first
//! half sent and the rest held back, so a download's progress stands part way until it is stopped, and at
//! `/first-held/cheats.zip` held the first time and whole after. It prints the three addresses, a line each, and ends
//! when its input closes (EmuSen_RustPlatform.md §56).

use emusen_galaxia::archive::ZipWriter;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime};

/// Every file under `dir`, by its path from `root` with forward slashes.
fn files(root: &Path, dir: &Path, out: &mut Vec<(String, std::path::PathBuf)>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir).expect("the folder is listed").map_while(Result::ok).collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let path = e.path();
        if path.is_dir() {
            files(root, &path, out);
        } else {
            let name = path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
            out.push((name, path));
        }
    }
}

fn archive(folder: &Path) -> Vec<u8> {
    let made = std::env::temp_dir().join(format!("emusen-cheats-server-{}.zip", std::process::id()));
    let mut w = ZipWriter::create(&made).expect("the archive is made");
    let mut all = Vec::new();
    files(folder, folder, &mut all);
    for (name, path) in all {
        w.add(&name, &path, SystemTime::now(), false, &|| false).expect("a file is added");
    }
    w.finish().expect("the archive is finished");
    let bytes = std::fs::read(&made).expect("the archive is read");
    let _ = std::fs::remove_file(&made);
    bytes
}

/// Whether `/first-held/cheats.zip` has been asked once.
static ASKED: AtomicBool = AtomicBool::new(false);

/// One request answered: the whole archive, its first half and then nothing until the client goes, or not found.
fn answer(mut stream: TcpStream, zip: &[u8]) {
    let mut line = String::new();
    if BufReader::new(stream.try_clone().unwrap()).read_line(&mut line).is_err() {
        return;
    }
    let mut path = line.split_whitespace().nth(1).unwrap_or_default().to_owned();
    if path == "/first-held/cheats.zip" {
        path = if ASKED.swap(true, Ordering::SeqCst) { "/cheats.zip" } else { "/held/cheats.zip" }.to_owned();
    }
    let head = |status: &str, length: usize| format!("HTTP/1.1 {status}\r\nContent-Type: application/zip\r\nContent-Length: {length}\r\nConnection: close\r\n\r\n");
    match path.as_str() {
        "/cheats.zip" => {
            let _ = stream.write_all(head("200 OK", zip.len()).as_bytes()).and_then(|_| stream.write_all(zip));
        }
        "/held/cheats.zip" => {
            let _ = stream.write_all(head("200 OK", zip.len()).as_bytes()).and_then(|_| stream.write_all(&zip[..zip.len() / 2])).and_then(|_| stream.flush());
            let _ = stream.set_read_timeout(Some(Duration::from_millis(100)));
            let mut byte = [0u8; 1];
            for _ in 0..600 {
                match stream.read(&mut byte) {
                    Ok(0) => return,
                    Ok(_) => continue,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => continue,
                    _ => return,
                }
            }
        }
        _ => {
            let _ = stream.write_all(head("404 Not Found", 0).as_bytes());
        }
    }
}

fn main() {
    let folder = std::env::args().nth(1).expect("the folder of files to serve");
    let zip = archive(Path::new(&folder));
    let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback port");
    let port = listener.local_addr().unwrap().port();
    println!("http://127.0.0.1:{port}/cheats.zip");
    println!("http://127.0.0.1:{port}/held/cheats.zip");
    println!("http://127.0.0.1:{port}/first-held/cheats.zip");
    let _ = std::io::stdout().flush();
    std::thread::spawn(move || {
        for stream in listener.incoming().map_while(Result::ok) {
            let zip = zip.clone();
            std::thread::spawn(move || answer(stream, &zip));
        }
    });
    let _ = std::io::stdin().read_to_end(&mut Vec::new());
}
