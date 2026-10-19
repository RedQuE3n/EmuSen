//! The snapshot rings, the C# `SnapshotRing` and `RingProvider`: a frame writes raw values into records made once and
//! reused round a ring of four, so a frame allocates nothing once warm; a reader takes a copy of the newest, from any
//! thread (EmuSen_Backend.md §14.5).

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

const DEPTH: usize = 4;

/// One record: the values and how many of them hold.
#[derive(Clone, Debug, Default)]
pub struct Values<T> {
    pub values: Vec<T>,
    pub count: usize,
}

pub struct Ring<T> {
    records: [Mutex<Values<T>>; DEPTH],
    latest: AtomicUsize,
}

impl<T: Clone + Default> Ring<T> {
    pub fn new(capacity: usize) -> Ring<T> {
        let make = || Mutex::new(Values { values: vec![T::default(); capacity.max(1)], count: 0 });
        Ring { records: [make(), make(), make(), make()], latest: AtomicUsize::new(0) }
    }

    /// `Refresh`: `fill` writes the values and answers how many there are; more than the record holds asks for a
    /// larger one, which is made and filled again.
    pub fn refresh(&self, mut fill: impl FnMut(&mut [T]) -> usize) {
        let next = (self.latest.load(Ordering::Acquire) + 1) % DEPTH;
        {
            let mut r = self.records[next].lock().unwrap_or_else(|e| e.into_inner());
            let mut n = fill(&mut r.values);
            if n > r.values.len() {
                r.values = vec![T::default(); n];
                n = fill(&mut r.values);
            }
            r.count = n.min(r.values.len());
        }
        self.latest.store(next, Ordering::Release);
    }

    /// The newest record's values, read under its lock.
    pub fn with<R>(&self, f: impl FnOnce(&[T]) -> R) -> R {
        let r = self.records[self.latest.load(Ordering::Acquire)].lock().unwrap_or_else(|e| e.into_inner());
        f(&r.values[..r.count])
    }

    /// A copy of the newest record's values.
    pub fn newest(&self) -> Vec<T> {
        self.with(|v| v.to_vec())
    }
}
