//! The faults the broker's proofs ask for: a fault the core handles as a fast-memory recompiler does, one it does not,
//! an abort, a write outside its folders, a network connection, and a frame that never ends (EmuSen_LibretroHost.md §3.5).

/// The page the handled fault touches, and the handler that makes it writable and lets the write run again.
static mut PAGE: *mut libc::c_void = std::ptr::null_mut();

extern "C" fn on_segv(_sig: libc::c_int, info: *mut libc::siginfo_t, _ctx: *mut libc::c_void) {
    unsafe {
        let addr = (*info).si_addr() as usize;
        let page = PAGE as usize;
        if page != 0 && (page..page + 4096).contains(&addr) {
            libc::mprotect(PAGE, 4096, libc::PROT_READ | libc::PROT_WRITE);
            return;
        }
        libc::signal(libc::SIGSEGV, libc::SIG_DFL);
    }
}

/// 1 where the fault was handled or the action done, 2 where the system refused it; a crash or a hang never returns.
pub fn run(kind: &str) -> u8 {
    match kind {
        "handled_segv" => unsafe {
            PAGE = libc::mmap(std::ptr::null_mut(), 4096, libc::PROT_NONE, libc::MAP_PRIVATE | libc::MAP_ANONYMOUS, -1, 0);
            let mut sa: libc::sigaction = std::mem::zeroed();
            sa.sa_sigaction = on_segv as *const () as usize;
            sa.sa_flags = libc::SA_SIGINFO;
            libc::sigaction(libc::SIGSEGV, &sa, std::ptr::null_mut());
            std::ptr::write_volatile(PAGE as *mut u8, 0x5A);
            if std::ptr::read_volatile(PAGE as *const u8) == 0x5A { 1 } else { 0 }
        },
        "segv" => unsafe {
            std::ptr::write_volatile(std::ptr::null_mut::<u8>().wrapping_add(16), 1);
            0
        },
        "abort" => std::process::abort(),
        "write_outside" => {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
            let target = std::path::Path::new(&home).join(format!("emusen-testcore-escape-{}", std::process::id()));
            match std::fs::write(&target, b"escaped") {
                Ok(()) => {
                    let _ = std::fs::remove_file(&target);
                    1
                }
                Err(_) => 2,
            }
        }
        "socket" => match std::net::TcpStream::connect_timeout(&"127.0.0.1:9".parse().unwrap(), std::time::Duration::from_millis(500)) {
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => 2,
            _ => 1,
        },
        "hang" => loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        },
        _ => 0,
    }
}
