//! A tool's request run as a job on a thread of its own, its events queued as lines for whoever reads them: the shell's
//! modules through their services and Mistress's windows through the platform library, which never calls its host
//! (EmuSen_RustPlatform.md §3). Drawn out of the library tool on 2026-10-31 so that the core tool runs on the same one
//! (EmuSen_Cores.md §6); what a request and its events mean stays each tool's.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// A job's queue of event lines, which its thread writes and its reader polls.
pub struct Events {
    lines: Mutex<(VecDeque<String>, bool)>,
    arrived: Condvar,
}

impl Events {
    fn new() -> Events {
        Events { lines: Mutex::new((VecDeque::new(), false)), arrived: Condvar::new() }
    }

    /// One event's line queued for the reader.
    pub fn emit(&self, line: String) {
        let mut guard = self.lines.lock().unwrap_or_else(|p| p.into_inner());
        guard.0.push_back(line);
        self.arrived.notify_all();
    }

    fn end(&self) {
        let mut guard = self.lines.lock().unwrap_or_else(|p| p.into_inner());
        guard.1 = true;
        self.arrived.notify_all();
    }
}

/// A running or finished request.
pub struct Job {
    events: Arc<Events>,
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Job {
    /// Runs `work` on a thread named `name`, handing it the queue and a question that answers true once a stop was asked.
    pub fn spawn<F>(name: &str, work: F) -> Result<Job, String>
    where
        F: FnOnce(&Events, &dyn Fn() -> bool) + Send + 'static,
    {
        let events = Arc::new(Events::new());
        let stop = Arc::new(AtomicBool::new(false));
        let (e, s) = (events.clone(), stop.clone());
        let thread = std::thread::Builder::new()
            .name(name.into())
            .spawn(move || {
                work(&e, &|| s.load(Ordering::Relaxed));
                e.end();
            })
            .map_err(|e| e.to_string())?;
        Ok(Job { events, stop, thread: Some(thread) })
    }

    /// The next event, waiting up to `wait` for one; `None` when none came, and none ever after the last.
    pub fn peek(&self, wait: Duration) -> Option<String> {
        let mut guard = self.events.lines.lock().unwrap_or_else(|p| p.into_inner());
        if guard.0.is_empty() && !guard.1 {
            guard = self.events.arrived.wait_timeout(guard, wait).unwrap_or_else(|p| p.into_inner()).0;
        }
        guard.0.front().cloned()
    }

    /// Takes the event `peek` gave.
    pub fn take(&self) {
        self.events.lines.lock().unwrap_or_else(|p| p.into_inner()).0.pop_front();
    }

    /// Whether the job has ended and every event was taken.
    pub fn finished(&self) -> bool {
        let guard = self.events.lines.lock().unwrap_or_else(|p| p.into_inner());
        guard.1 && guard.0.is_empty()
    }

    /// Asks the job to stop; it ends with what it did, as its tool decides.
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        self.stop();
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_arrive_in_order_and_end_after_the_last() {
        let job = Job::spawn("t", |e, _| {
            e.emit("one".into());
            e.emit("two".into());
        })
        .unwrap();
        let mut got = Vec::new();
        while !job.finished() {
            if let Some(line) = job.peek(Duration::from_millis(50)) {
                got.push(line);
                job.take();
            }
        }
        assert_eq!(got, ["one", "two"]);
        assert_eq!(job.peek(Duration::from_millis(1)), None);
    }

    #[test]
    fn a_stop_reaches_the_work_and_drop_waits_for_it() {
        let job = Job::spawn("t", |e, stop| {
            while !stop() {
                std::thread::sleep(Duration::from_millis(1));
            }
            e.emit("stopped".into());
        })
        .unwrap();
        job.stop();
        let line = loop {
            if let Some(l) = job.peek(Duration::from_millis(100)) {
                break l;
            }
        };
        assert_eq!(line, "stopped");
        drop(job);
    }

    #[test]
    fn peek_does_not_take() {
        let job = Job::spawn("t", |e, _| e.emit("x".into())).unwrap();
        let first = loop {
            if let Some(l) = job.peek(Duration::from_millis(100)) {
                break l;
            }
        };
        assert_eq!(job.peek(Duration::ZERO).as_deref(), Some(first.as_str()));
        job.take();
        while !job.finished() {
            job.peek(Duration::from_millis(10));
        }
    }
}
