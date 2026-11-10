//! The library tool's and the core tool's jobs (emusen_platform.h, `emusen_tool_job_*`): a request started on the
//! library's own thread, its events pulled one at a time by the caller, never pushed. The requests and the events are
//! the tools' JSON (EmuSen_GamePackages.md §12.3, EmuSen_LibraryTools.md §7, EmuSen_Cores.md §6) and cross as
//! `serde_json::Value`, so each tool stays the one owner of its fields.

use crate::error::Result;
use crate::out::{json, to_json};
use crate::platform::Platform;
use crate::raw;
use serde_json::Value;
use std::ptr::NonNull;
use std::sync::Arc;

/// What a pull found: an event, none yet within the wait, or the job ended with every event taken.
#[derive(Clone, Debug, PartialEq)]
pub enum Step {
    Event(Value),
    Waiting,
    Ended,
}

/// A running job; dropping it stops the job, waits for it to end and frees it.
pub struct ToolJob {
    platform: Arc<Platform>,
    job: NonNull<raw::emusen_tool_job>,
}

// The job runs on a thread of the library's own; its handle is pulled from one thread at a time.
unsafe impl Send for ToolJob {}

impl ToolJob {
    /// A job started from a request: `{"op": "core-ls"}`, `{"op": "pkg-ls", "paths": []}`, `{"op": "ls"}` and the rest.
    /// A request that is not one is refused with words; what a job refuses once running is its last event's.
    pub fn start(platform: &Arc<Platform>, request: &Value) -> Result<ToolJob> {
        let text = to_json(request);
        let mut job = std::ptr::null_mut();
        platform.status(unsafe { platform.lib.emusen_tool_job_start(text.as_ptr(), text.len(), &mut job) }.into())?;
        let job = NonNull::new(job).ok_or_else(|| crate::error::status(raw::EMUSEN_PLATFORM_NULL, platform.last_error()))?;
        Ok(ToolJob { platform: platform.clone(), job })
    }

    /// The next event, waiting up to `wait_ms` for one.
    pub fn next(&self, wait_ms: u32) -> Result<Step> {
        let lib = &self.platform.lib;
        match self.platform.read(|o, l| unsafe { lib.emusen_tool_job_next(self.job.as_ptr(), wait_ms, o, l) })? {
            None => Ok(Step::Waiting),
            Some(bytes) if bytes.is_empty() => Ok(Step::Ended),
            Some(bytes) => json("tool_job_next", &bytes).map(Step::Event),
        }
    }

    /// Every event waiting now, taken without waiting and added to `out`; true once the job has ended and all are taken.
    pub fn drain(&self, out: &mut Vec<Value>) -> Result<bool> {
        loop {
            match self.next(0)? {
                Step::Event(e) => out.push(e),
                Step::Waiting => return Ok(false),
                Step::Ended => return Ok(true),
            }
        }
    }

    /// Asks the job to stop; it ends with what it did, its last events still to be pulled.
    pub fn stop(&self) -> Result<()> {
        self.platform.status(unsafe { self.platform.lib.emusen_tool_job_stop(self.job.as_ptr()) }.into()).map(drop)
    }
}

impl Drop for ToolJob {
    fn drop(&mut self) {
        unsafe { self.platform.lib.emusen_tool_job_free(self.job.as_ptr()) };
    }
}
