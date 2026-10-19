//! The runtime's events, the C# `ManagedEvents` and `ManagedSubscription`: each made once and only when a
//! subscription wants it, `{"kind","session",…}`, into each wanting subscription's bounded queue, the oldest dropped
//! and counted when it is full; drained whole by the client, waited for with a timeout (EmuSen_Backend.md §3.5).

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

use crate::json::Json;

pub const CAPACITY: usize = 256;

#[derive(Default)]
struct Queue {
    events: VecDeque<String>,
    dropped: i64,
}

pub struct Subscription {
    pub session: u32,
    pub kinds: u32,
    /// False once it is ended or its runtime freed, after which its handle answers as no subscription.
    pub live: std::sync::atomic::AtomicBool,
    queue: Mutex<Queue>,
    ready: Condvar,
}

impl Subscription {
    pub fn wants(&self, session: u32, kind: u32) -> bool {
        (self.session == 0 || self.session == session) && (self.kinds == 0 || self.kinds & kind != 0)
    }

    fn add(&self, json: &str) {
        let mut q = self.queue.lock().unwrap_or_else(|p| p.into_inner());
        if q.events.len() == CAPACITY {
            q.events.pop_front();
            q.dropped += 1;
        }
        q.events.push_back(json.to_string());
        self.ready.notify_all();
    }

    /// The waiting events as one array, removed only when `into` takes them all; the array's length in bytes.
    pub fn drain(&self, into: Option<&mut [u8]>) -> usize {
        let mut q = self.queue.lock().unwrap_or_else(|p| p.into_inner());
        let mut text = String::from("[");
        let mut first = true;
        if q.dropped > 0 {
            text.push_str(&format!("{{\"kind\":\"dropped\",\"count\":{}}}", q.dropped));
            first = false;
        }
        for e in &q.events {
            if !first {
                text.push(',');
            }
            text.push_str(e);
            first = false;
        }
        text.push(']');
        if let Some(into) = into
            && into.len() >= text.len()
        {
            into[..text.len()].copy_from_slice(text.as_bytes());
            q.events.clear();
            q.dropped = 0;
        }
        text.len()
    }

    /// True when events are waiting, false when the timeout passed first.
    pub fn wait(&self, timeout_ms: u32) -> bool {
        let q = self.queue.lock().unwrap_or_else(|p| p.into_inner());
        if !q.events.is_empty() || q.dropped > 0 {
            return true;
        }
        let (q, _) = self.ready.wait_timeout(q, std::time::Duration::from_millis(timeout_ms.min(i32::MAX as u32) as u64)).unwrap_or_else(|p| p.into_inner());
        !q.events.is_empty() || q.dropped > 0
    }
}

#[derive(Default)]
pub struct Events {
    subscriptions: Mutex<Vec<Arc<Subscription>>>,
}

impl Events {
    pub fn subscribe(&self, session: u32, kinds: u32) -> Arc<Subscription> {
        let s = Arc::new(Subscription { session, kinds, live: std::sync::atomic::AtomicBool::new(true), queue: Mutex::new(Queue::default()), ready: Condvar::new() });
        self.subscriptions.lock().unwrap_or_else(|p| p.into_inner()).push(s.clone());
        s
    }

    pub fn remove(&self, s: &Arc<Subscription>) {
        s.live.store(false, std::sync::atomic::Ordering::Release);
        self.subscriptions.lock().unwrap_or_else(|p| p.into_inner()).retain(|x| !Arc::ptr_eq(x, s));
    }

    pub fn wanted(&self, session: u32, kind: u32) -> bool {
        self.subscriptions.lock().unwrap_or_else(|p| p.into_inner()).iter().any(|s| s.wants(session, kind))
    }

    /// An event made once and put in every subscription that wants it; nothing made when none does.
    pub fn raise(&self, session: u32, kind: u32, name: &str, fields: impl FnOnce(&mut Json)) {
        if !self.wanted(session, kind) {
            return;
        }
        let mut j = Json::new();
        j.begin_object().str("kind", name).int("session", session as i64);
        fields(&mut j);
        j.end_object();
        let json = j.finish();
        for s in self.subscriptions.lock().unwrap_or_else(|p| p.into_inner()).iter() {
            if s.wants(session, kind) {
                s.add(&json);
            }
        }
    }

    /// Lines of the runtime's own as one log event of source "runtime" (§29.11, Q2).
    pub fn raise_runtime_lines(&self, session: u32, lines: &[String]) {
        if lines.is_empty() {
            return;
        }
        self.raise(session, crate::abi::EVENT_LOG, "log", |j| {
            j.key_array("lines");
            for l in lines {
                j.str_value(l);
            }
            j.end_array().str("source", "runtime");
        });
    }

    /// Every subscription ended, as the runtime is freed.
    pub fn clear(&self) {
        for s in self.subscriptions.lock().unwrap_or_else(|p| p.into_inner()).drain(..) {
            s.live.store(false, std::sync::atomic::Ordering::Release);
        }
    }
}
