//! Safe RAII wrapper around DianaSh's host C interface (`Platform/dianash/include/dianash.h`).

use crate::error::{status, Error, Result};
use crate::out::{answer, check, text, words};
use crate::platform::Platform;
use crate::raw;
use std::ffi::c_void;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex};

/// Configuration passed when creating a `DianashShell`.
#[derive(Clone, Copy, Debug, Default)]
pub struct DianashConfig<'a> {
    pub sessions: bool,
    pub interactive: bool,
    pub root: Option<&'a str>,
    pub budget_ms: u32,
}

/// Outcome of submitting one line to a shell session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubmitResult {
    Done,
    NeedsMore,
    Exit,
}

/// One service table passed into `set_services` or `session_set_services`.
#[derive(Clone, Copy, Debug)]
pub struct ServiceEntry<'a> {
    pub name: &'a str,
    pub major: u32,
    pub minor: u32,
    pub table: *const c_void,
}

/// Owned `dianash_shell *` freed on drop.
pub struct DianashShell {
    platform: Arc<Platform>,
    shell: NonNull<raw::dianash_shell>,
    run_lock: Mutex<()>,
}

// `dianash_shell` serialises calls internally via its shell mutex.
unsafe impl Send for DianashShell {}
unsafe impl Sync for DianashShell {}

impl DianashShell {
    /// Creates a new DianaSh shell backed by `platform`.
    pub fn new(platform: Arc<Platform>, config: Option<DianashConfig<'_>>) -> Result<DianashShell> {
        let abi = unsafe { platform.lib.dianash_abi_version() };
        if abi >> 16 != raw::DIANASH_ABI_MAJOR {
            return Err(Error::Refused {
                path: platform.path().display().to_string(),
                words: format!("DianaSh host ABI major {} does not match {}.", abi >> 16, raw::DIANASH_ABI_MAJOR),
            });
        }
        let cfg = config.map(|c| {
            let mut flags = 0u32;
            if c.sessions {
                flags |= raw::DIANASH_CONFIG_SESSIONS;
            }
            if c.interactive {
                flags |= raw::DIANASH_CONFIG_INTERACTIVE;
            }
            let (root_ptr, root_len) = text(c.root);
            raw::dianash_config {
                size: std::mem::size_of::<raw::dianash_config>() as u32,
                flags,
                root: raw::dianash_bytes { data: root_ptr, len: root_len },
                budget_ms: c.budget_ms,
            }
        });
        let cfg_ptr = cfg.as_ref().map_or(std::ptr::null(), |c| c as *const raw::dianash_config);
        let ptr = unsafe { platform.lib.dianash_new(cfg_ptr) };
        let shell = NonNull::new(ptr).ok_or_else(|| {
            let msg = words(|o, l| unsafe { platform.lib.dianash_last_error(o, l) });
            status(raw::DIANASH_FAULT, msg)
        })?;
        Ok(DianashShell { platform, shell, run_lock: Mutex::new(()) })
    }

    fn last_error(&self) -> String {
        words(|o, l| unsafe { self.platform.lib.dianash_last_error(o, l) })
    }

    fn check_code(&self, code: i32) -> Result<i32> {
        check(i64::from(code), || self.last_error()).map(|n| n as i32)
    }

    /// # Safety: Every non-null `ServiceEntry::table` pointer must remain valid for the shell's lifetime or until cleared.
    pub unsafe fn set_services(&self, services: &[ServiceEntry<'_>]) -> Result<()> {
        let raw_list = build_services(services);
        self.check_code(unsafe { self.platform.lib.dianash_set_services(self.shell.as_ptr(), raw_list.as_ptr(), raw_list.len()) })?;
        Ok(())
    }

    /// # Safety: Every non-null `ServiceEntry::table` pointer must remain valid for the shell's lifetime or until cleared.
    pub unsafe fn session_set_services(&self, session: u32, services: &[ServiceEntry<'_>]) -> Result<()> {
        let raw_list = build_services(services);
        self.check_code(unsafe {
            self.platform.lib.dianash_session_set_services(self.shell.as_ptr(), session, raw_list.as_ptr(), raw_list.len())
        })?;
        Ok(())
    }

    /// # Safety: If non-null, `session_service` must point to a valid `emusen_session_v1` that outlives the shell or attachment.
    pub unsafe fn attach_session(&self, session: u32, session_service: *const c_void) -> Result<()> {
        if session_service.is_null() {
            return unsafe { self.session_set_services(session, &[]) };
        }
        let svc = ServiceEntry { name: "emusen.session", major: 1, minor: 4, table: session_service };
        unsafe { self.session_set_services(session, &[svc]) }
    }

    /// Submits one line to `session` (`0` for the current session).
    pub fn submit(&self, session: u32, line: &str) -> Result<SubmitResult> {
        let code = self.check_code(unsafe { self.platform.lib.dianash_submit(self.shell.as_ptr(), session, line.as_ptr(), line.len()) })?;
        Ok(match code as u32 {
            raw::DIANASH_NEEDS_MORE => SubmitResult::NeedsMore,
            raw::DIANASH_EXIT => SubmitResult::Exit,
            _ => SubmitResult::Done,
        })
    }

    /// Takes buffered output from the most recent submitted line.
    pub fn output_take(&self) -> Result<String> {
        let bytes = answer(|o, l| unsafe { self.platform.lib.dianash_output_take(self.shell.as_ptr(), o, l) }, || self.last_error())?;
        emusen_c_calls::string("dianash_output_take", bytes)
    }

    /// Reads the exit status of the last completed command in `session`.
    pub fn last_status(&self, session: u32) -> Result<i32> {
        let mut st = 0i32;
        self.check_code(unsafe { self.platform.lib.dianash_last_status(self.shell.as_ptr(), session, &mut st) })?;
        Ok(st)
    }

    /// Submits `line`, drains its output, and reads the command exit status atomically under the shell run lock.
    pub fn run(&self, session: u32, line: &str) -> Result<(SubmitResult, i32, String)> {
        let _guard = self.run_lock.lock().unwrap_or_else(|e| e.into_inner());
        let submitted = self.submit(session, line)?;
        let output = self.output_take()?;
        let status = self.last_status(session)?;
        Ok((submitted, status, output))
    }

    /// Requests cancellation of any running command from any thread.
    pub fn cancel(&self) -> Result<()> {
        self.check_code(unsafe { self.platform.lib.dianash_cancel(self.shell.as_ptr()) })?;
        Ok(())
    }

    /// Returns whether `line` is a read-only command safe to run without waiting.
    pub fn is_read_only(&self, session: u32, line: &str) -> Result<bool> {
        let code = self.check_code(unsafe { self.platform.lib.dianash_is_read_only(self.shell.as_ptr(), session, line.as_ptr(), line.len()) })?;
        Ok(code != 0)
    }

    /// Creates a new unlisted shell session and returns its id.
    pub fn session_new(&self) -> Result<u32> {
        let id = check(unsafe { self.platform.lib.dianash_session_new(self.shell.as_ptr()) }, || self.last_error())?;
        Ok(id as u32)
    }

    /// Attaches a session under an optional name and makes it current.
    pub fn session_attach(&self, session: u32, name: Option<&str>) -> Result<()> {
        let (ptr, len) = text(name);
        self.check_code(unsafe { self.platform.lib.dianash_session_attach(self.shell.as_ptr(), session, ptr, len) })?;
        Ok(())
    }

    /// Switches the current session to `session`.
    pub fn session_switch(&self, session: u32) -> Result<()> {
        self.check_code(unsafe { self.platform.lib.dianash_session_switch(self.shell.as_ptr(), session) })?;
        Ok(())
    }

    /// Kills or unlists `session`.
    pub fn session_kill(&self, session: u32) -> Result<()> {
        self.check_code(unsafe { self.platform.lib.dianash_session_kill(self.shell.as_ptr(), session) })?;
        Ok(())
    }
}

impl Drop for DianashShell {
    fn drop(&mut self) {
        unsafe {
            let _ = self.platform.lib.dianash_free(self.shell.as_ptr());
        }
    }
}

fn build_services(services: &[ServiceEntry<'_>]) -> Vec<raw::dianash_service> {
    services
        .iter()
        .map(|s| raw::dianash_service {
            size: std::mem::size_of::<raw::dianash_service>() as u32,
            name: raw::dianash_bytes { data: s.name.as_ptr(), len: s.name.len() },
            major: s.major,
            minor: s.minor,
            table: s.table,
        })
        .collect()
}
