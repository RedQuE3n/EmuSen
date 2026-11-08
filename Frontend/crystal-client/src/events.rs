//! Events, pulled: a subscription drained on the client's timer or waited on with a timeout, never a callback.

use crate::error::Result;
use crate::out::json;
use crate::raw;
use crate::runtime::Runtime;
use crate::shapes::SessionId;
use serde::Deserialize;
use serde_json::Value;
use std::ptr::NonNull;

/// One event, by its "kind"; a kind a later minor adds is `Other`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Event {
    /// "opened" or "closed".
    Session {
        session: SessionId,
        op: String,
    },
    /// "paused", "resumed", "halted" or "stepped".
    Run {
        session: SessionId,
        op: String,
        #[serde(default)]
        processor: Option<u32>,
        #[serde(default)]
        address: Option<u64>,
    },
    /// "saved" or "loaded"; after loaded a client resets its audio rate control.
    State {
        session: SessionId,
        op: String,
        #[serde(default)]
        path: Option<String>,
    },
    /// "stepped", "moved" or "cleared".
    Rewind {
        session: SessionId,
        op: String,
        #[serde(default)]
        frame: Option<u64>,
    },
    Settings {
        session: SessionId,
        scope: Value,
        key: String,
        value: Option<String>,
    },
    /// "inserted", "ejected", "changed" or "lid".
    Disc {
        session: SessionId,
        op: String,
        #[serde(default)]
        index: Option<u32>,
        #[serde(default)]
        open: Option<bool>,
    },
    /// Lines oldest first; "source" is "core" or "runtime".
    Log {
        session: SessionId,
        lines: Vec<String>,
        #[serde(default)]
        source: Option<String>,
    },
    /// The queue overflowed and dropped this many.
    Dropped {
        count: u64,
    },
    #[serde(other)]
    Other,
}

/// The event kinds, as bits; none is every kind.
pub mod kinds {
    pub use crate::raw::{
        EMUSEN_CRYSTAL_EVENT_DISC as DISC, EMUSEN_CRYSTAL_EVENT_LOG as LOG, EMUSEN_CRYSTAL_EVENT_REWIND as REWIND,
        EMUSEN_CRYSTAL_EVENT_RUN as RUN, EMUSEN_CRYSTAL_EVENT_SESSION as SESSION, EMUSEN_CRYSTAL_EVENT_SETTINGS as SETTINGS,
        EMUSEN_CRYSTAL_EVENT_STATE as STATE,
    };
}

/// A subscription to a session's events, or every session's; ended when dropped.
pub struct Subscription {
    rt: Runtime,
    sub: NonNull<raw::emusen_crystal_subscription>,
}

// The subscription locks its own queue; the header puts no thread on it.
unsafe impl Send for Subscription {}

impl Runtime {
    /// A subscription to `session`'s events (0 for every session's) of `kinds` (0 for every kind).
    pub fn subscribe(&self, session: SessionId, kinds: u32) -> Result<Subscription> {
        let sub = unsafe { self.lib().emusen_crystal_subscribe(self.rt(), session, kinds) };
        NonNull::new(sub)
            .map(|sub| Subscription { rt: self.clone(), sub })
            .ok_or_else(|| crate::error::status(raw::EMUSEN_SVC_FAULT, self.words()))
    }
}

impl Subscription {
    /// The events waiting, oldest first, removed from the queue.
    pub fn drain(&self) -> Result<Vec<Event>> {
        let bytes = self.rt.read(|o, l| unsafe { self.rt.lib().emusen_crystal_subscription_drain(self.sub.as_ptr(), o, l) })?;
        if bytes.is_empty() {
            return Ok(Vec::new());
        }
        json("subscription_drain", &bytes)
    }

    /// True when events are waiting, false when `timeout_ms` passed first.
    pub fn wait(&self, timeout_ms: u32) -> Result<bool> {
        self.rt.status(unsafe { self.rt.lib().emusen_crystal_subscription_wait(self.sub.as_ptr(), timeout_ms) }.into()).map(|n| n == 1)
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        unsafe { self.rt.lib().emusen_crystal_unsubscribe(self.sub.as_ptr()) };
    }
}
