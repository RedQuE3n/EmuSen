//! The shell's guard over the settings service (module ABI 1.1): a module is handed, in place of the host's table, a
//! table of the shell's that lets through only the keys its manifest names, and an approval only from a command typed
//! at the prompt. See EmuSen_DianaSh.md §9.9.

use crate::abi::{dianash_bytes, dianash_settings_v1, status};
use std::cell::RefCell;
use std::ffi::c_void;

thread_local! {
    /// The guard's own words for its last refusal on this thread, given before the host's.
    static REFUSED: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// One module's guard: what its manifest names, the host's table of this call, and whether this call was typed at the prompt.
pub(crate) struct Guard {
    table: dianash_settings_v1,
    keys: Vec<String>,
    approvals: bool,
    inner: *const dianash_settings_v1,
    at_prompt: bool,
}

impl Guard {
    /// A guard for a module whose manifest names `keys` and `approvals`; none for one that names neither.
    pub fn new(keys: Vec<String>, approvals: bool) -> Option<Box<Guard>> {
        if keys.is_empty() && !approvals {
            return None;
        }
        let mut g = Box::new(Guard {
            table: dianash_settings_v1 { size: std::mem::size_of::<dianash_settings_v1>() as u32, ctx: std::ptr::null_mut(), get, set, approve, revoke, last_error },
            keys,
            approvals,
            inner: std::ptr::null(),
            at_prompt: false,
        });
        g.table.ctx = &mut *g as *mut Guard as *mut c_void;
        Some(g)
    }

    /// The table a call is handed, over the host's `inner`, knowing whether the call was typed at the prompt.
    pub fn bind(&mut self, inner: *const c_void, at_prompt: bool) -> *const c_void {
        self.inner = inner as *const dianash_settings_v1;
        self.at_prompt = at_prompt;
        &self.table as *const dianash_settings_v1 as *const c_void
    }
}

fn refuse(words: String) -> i32 {
    REFUSED.with_borrow_mut(|r| *r = Some(words));
    status::SETTINGS_DENIED
}

unsafe fn guard<'a>(ctx: *mut c_void) -> Option<&'a Guard> {
    REFUSED.with_borrow_mut(|r| *r = None);
    let g = unsafe { (ctx as *const Guard).as_ref() }?;
    (!g.inner.is_null()).then_some(g)
}

fn text(bytes: dianash_bytes) -> String {
    unsafe { bytes.slice() }.map(|b| String::from_utf8_lossy(b).into_owned()).unwrap_or_default()
}

fn named(g: &Guard, key: dianash_bytes) -> Result<(), i32> {
    let key = text(key);
    if g.keys.iter().any(|k| *k == key) { Ok(()) } else { Err(refuse(format!("the module's manifest names no setting {key:?}, so it may not reach it"))) }
}

unsafe extern "C" fn get(ctx: *mut c_void, scope: u32, target: dianash_bytes, key: dianash_bytes, out: *mut u8, cap: usize, needed: *mut i64) -> i32 {
    let Some(g) = (unsafe { guard(ctx) }) else { return status::NULL };
    if let Err(code) = named(g, key) {
        return code;
    }
    unsafe { ((*g.inner).get)((*g.inner).ctx, scope, target, key, out, cap, needed) }
}

unsafe extern "C" fn set(ctx: *mut c_void, scope: u32, target: dianash_bytes, key: dianash_bytes, value: dianash_bytes) -> i32 {
    let Some(g) = (unsafe { guard(ctx) }) else { return status::NULL };
    if let Err(code) = named(g, key) {
        return code;
    }
    unsafe { ((*g.inner).set)((*g.inner).ctx, scope, target, key, value) }
}

unsafe extern "C" fn approve(ctx: *mut c_void, sha256: dianash_bytes) -> i32 {
    let Some(g) = (unsafe { guard(ctx) }) else { return status::NULL };
    if !g.approvals {
        return refuse("the module's manifest asks for no approvals, so it may not approve a core".into());
    }
    if !g.at_prompt {
        return refuse("a core is approved at the prompt, not by a script, a substitution or a pipe".into());
    }
    unsafe { ((*g.inner).approve)((*g.inner).ctx, sha256) }
}

unsafe extern "C" fn revoke(ctx: *mut c_void, sha256: dianash_bytes) -> i32 {
    let Some(g) = (unsafe { guard(ctx) }) else { return status::NULL };
    if !g.approvals {
        return refuse("the module's manifest asks for no approvals, so it may not revoke one".into());
    }
    unsafe { ((*g.inner).revoke)((*g.inner).ctx, sha256) }
}

unsafe extern "C" fn last_error(ctx: *mut c_void, out: *mut u8, cap: usize, needed: *mut i64) -> i32 {
    if let Some(words) = REFUSED.with_borrow(|r| r.clone()) {
        let bytes = words.as_bytes();
        if !needed.is_null() {
            unsafe { *needed = bytes.len() as i64 };
        }
        if bytes.len() > cap || out.is_null() {
            return status::BUFFER_TOO_SMALL;
        }
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), out, bytes.len()) };
        return status::OK;
    }
    let Some(g) = (unsafe { (ctx as *const Guard).as_ref() }) else { return status::NULL };
    if g.inner.is_null() {
        return status::NULL;
    }
    unsafe { ((*g.inner).last_error)((*g.inner).ctx, out, cap, needed) }
}
