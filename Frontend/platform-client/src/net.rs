//! The platform's HTTP client (emusen_platform.h, `emusen_net_download_*`): a GET whose body is held whole, run on a
//! thread of the library's own; the caller reads its progress, waits for its end, stops it, and takes its body or the
//! words for its failure. The request and the failure are the header's JSON and cross as `serde_json::Value`.

use crate::error::Result;
use crate::out::{json, to_json};
use crate::platform::Platform;
use crate::raw;
use serde_json::Value;
use std::ptr::NonNull;
use std::sync::Arc;

/// Where a download stands, the header's `EMUSEN_NET_` values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Running,
    Done,
    Failed,
    Cancelled,
}

impl State {
    fn of(n: i32) -> State {
        match n as u32 {
            raw::EMUSEN_NET_RUNNING => State::Running,
            raw::EMUSEN_NET_DONE => State::Done,
            raw::EMUSEN_NET_CANCELLED => State::Cancelled,
            _ => State::Failed,
        }
    }
}

/// A download; dropping it frees it, and a fetch still running ends on its own.
pub struct Download {
    platform: Arc<Platform>,
    d: NonNull<raw::emusen_net_download>,
}

// The header: its calls may be made from any thread, and it runs on the library's own.
unsafe impl Send for Download {}
unsafe impl Sync for Download {}

impl Download {
    /// A download started from `{"url", "timeout_ms", "test_roots"}`; a request that is not one is refused with words.
    pub fn start(platform: &Arc<Platform>, request: &Value) -> Result<Download> {
        let text = to_json(request);
        let d = unsafe { platform.lib.emusen_net_download_start(text.as_ptr(), text.len()) };
        let d = NonNull::new(d).ok_or_else(|| crate::error::status(raw::EMUSEN_PLATFORM_BAD_ARGUMENT, platform.last_error()))?;
        Ok(Download { platform: platform.clone(), d })
    }

    /// Where it stands, the body's bytes so far, and its length where the answer gave one.
    pub fn progress(&self) -> (State, u64, Option<u64>) {
        let (mut received, mut total) = (0u64, -1i64);
        let state = unsafe { self.platform.lib.emusen_net_download_progress(self.d.as_ptr(), &mut received, &mut total) };
        (State::of(state), received, u64::try_from(total).ok())
    }

    /// Waits up to `ms` for the end, and says where it stands.
    pub fn wait(&self, ms: u32) -> State {
        State::of(unsafe { self.platform.lib.emusen_net_download_wait(self.d.as_ptr(), ms) })
    }

    /// Stopped now: it stands cancelled at once.
    pub fn cancel(&self) {
        unsafe { self.platform.lib.emusen_net_download_cancel(self.d.as_ptr()) }
    }

    /// The body of one that is done; none otherwise.
    pub fn body(&self) -> Result<Option<Vec<u8>>> {
        let lib = &self.platform.lib;
        self.platform.read(|o, l| unsafe { lib.emusen_net_download_body(self.d.as_ptr(), o, l) })
    }

    /// Why one that failed or was stopped gave no body, `{"kind", "message", "inner", "status"}`; none otherwise.
    pub fn failure(&self) -> Result<Option<Value>> {
        let lib = &self.platform.lib;
        let bytes = self.platform.read(|o, l| unsafe { lib.emusen_net_download_failure(self.d.as_ptr(), o, l) })?;
        bytes.map(|b| json("net_download_failure", &b)).transpose()
    }
}

impl Drop for Download {
    fn drop(&mut self) {
        unsafe { self.platform.lib.emusen_net_download_free(self.d.as_ptr()) };
    }
}
