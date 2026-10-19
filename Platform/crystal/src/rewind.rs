//! Rewind, the C# `RewindBuffer`, `XorDeltaCodec` and `RewindThumbnail`: the newest snapshot whole, a chain of XOR
//! deltas back from it encoded on a worker thread of the session's own, the moments a reel shows with their pictures,
//! and the accounting the reel answers with, the C#'s (EmuSen_Rewind_And_FastForward.md §1 and §5,
//! EmuSen_Backend.md §29.4). Each delta's buffer is used again once the chain lets it go, so a capture allocates nothing
//! once the chain has reached its budget.

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

pub const DEFAULT_INTERVAL_FRAMES: i32 = 4;
pub const DEFAULT_BUDGET_BYTES: i64 = 96 * 1024 * 1024;
pub const DEFAULT_THUMBNAIL_BUDGET_BYTES: i64 = 32 * 1024 * 1024;

/// `[varint identical run][varint differing run][the run's XOR]`, repeated, an identical tail left out, into `out`.
pub fn encode(baseline: &[u8], target: &[u8], out: &mut Vec<u8>) {
    out.clear();
    let len = target.len().min(baseline.len());
    let mut i = 0;
    while i < len {
        let start = i;
        while i + 8 <= len && baseline[i..i + 8] == target[i..i + 8] {
            i += 8;
        }
        while i < len && baseline[i] == target[i] {
            i += 1;
        }
        let identical = i - start;
        let diff_start = i;
        while i < len && baseline[i] != target[i] {
            i += 1;
        }
        let diff = i - diff_start;
        if diff == 0 {
            break;
        }
        write_varint(out, identical as u32);
        write_varint(out, diff as u32);
        out.extend(baseline[diff_start..i].iter().zip(&target[diff_start..i]).map(|(a, b)| a ^ b));
    }
}

/// The delta applied in place, its own inverse; the C#'s words where it does not fit.
pub fn apply(state: &mut [u8], delta: &[u8]) -> Result<(), String> {
    let (mut i, mut pos) = (0i64, 0usize);
    while pos < delta.len() {
        i += read_varint(delta, &mut pos)? as i64;
        let diff = read_varint(delta, &mut pos)? as i64;
        if i < 0 || diff < 0 || i + diff > state.len() as i64 || pos as i64 + diff > delta.len() as i64 {
            return Err("XorDeltaCodec: delta does not fit the state it was applied to.".to_string());
        }
        let (at, n) = (i as usize, diff as usize);
        for (s, d) in state[at..at + n].iter_mut().zip(&delta[pos..pos + n]) {
            *s ^= d;
        }
        i += diff;
        pos += n;
    }
    Ok(())
}

fn write_varint(out: &mut Vec<u8>, value: u32) {
    let mut v = value;
    while v >= 0x80 {
        out.push((v | 0x80) as u8);
        v >>= 7;
    }
    out.push(v as u8);
}

/// As the C# reads it: an `int` that wraps where five bytes overflow it.
fn read_varint(buffer: &[u8], pos: &mut usize) -> Result<i32, String> {
    let (mut result, mut shift) = (0i32, 0u32);
    loop {
        let Some(&b) = buffer.get(*pos) else { return Err("XorDeltaCodec: truncated varint.".to_string()) };
        *pos += 1;
        result |= ((b & 0x7F) as i32).wrapping_shl(shift);
        if b & 0x80 == 0 {
            return Ok(result);
        }
        shift += 7;
        if shift > 28 {
            return Err("XorDeltaCodec: varint too long.".to_string());
        }
    }
}

/// A delta that does not fit, in the words the C#'s `InvalidDataException` reaches the API with.
fn invalid(words: String) -> (i32, String) {
    (emusen_session_service::abi::status::FAULT, format!("InvalidDataException: {words}"))
}

/// One moment's picture, downscaled and held as RGB565.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Thumbnail {
    pub width: i32,
    pub height: i32,
    pixels: Vec<u8>,
}

impl Thumbnail {
    /// A frame of `rows` rows each shown `row_repeat` times, sampled four times a pixel, `RewindThumbnail.From`.
    pub fn from(rgba: &[u8], width: i32, rows: i32, row_repeat: i32, target_width: i32) -> Option<Thumbnail> {
        if width <= 0 || rows <= 0 || target_width <= 0 || (rgba.len() as i64) < width as i64 * rows as i64 * 4 {
            return None;
        }
        let repeat = row_repeat.max(1) as i64;
        let shown = rows as i64 * repeat;
        let tw = target_width.min(width) as i64;
        let th = ((tw as f64 * shown as f64 / width as f64).round_ties_even() as i64).max(1);
        let w = width as i64;
        let mut pixels = vec![0u8; (tw * th * 2) as usize];
        let (left, right): (Vec<usize>, Vec<usize>) = (0..tw)
            .map(|x| {
                let x0 = x * w / tw;
                let x1 = (x0 + 1).max((x + 1) * w / tw);
                (((x0 + (x1 - x0) / 4) * 4) as usize, ((x0 + 3 * (x1 - x0) / 4) * 4) as usize)
            })
            .unzip();
        let row = (w * 4) as usize;
        let mut o = 0;
        for y in 0..th {
            let y0 = y * shown / th;
            let y1 = (y0 + 1).max((y + 1) * shown / th);
            let a_at = ((y0 + (y1 - y0) / 4) / repeat) as usize * row;
            let b_at = ((y0 + 3 * (y1 - y0) / 4) / repeat) as usize * row;
            let (ra, rb) = (&rgba[a_at..a_at + row], &rgba[b_at..b_at + row]);
            for x in 0..tw as usize {
                let (a, b) = (left[x], right[x]);
                let avg = |c: usize| (ra[a + c] as i32 + ra[b + c] as i32 + rb[a + c] as i32 + rb[b + c] as i32 + 2) >> 2;
                let packed = (((avg(0) >> 3) << 11) | ((avg(1) >> 2) << 5) | (avg(2) >> 3)) as u16;
                pixels[o] = packed as u8;
                pixels[o + 1] = (packed >> 8) as u8;
                o += 2;
            }
        }
        Some(Thumbnail { width: tw as i32, height: th as i32, pixels })
    }

    pub fn bytes(&self) -> i64 {
        self.pixels.len() as i64
    }

    /// Opaque RGBA8888, `ToRgba`.
    pub fn to_rgba(&self, out: &mut Vec<u8>) {
        for p in self.pixels.as_chunks::<2>().0 {
            let v = p[0] as u32 | (p[1] as u32) << 8;
            let (r, g, b) = ((v >> 11) & 31, (v >> 5) & 63, v & 31);
            out.extend_from_slice(&[((r << 3) | (r >> 2)) as u8, ((g << 2) | (g >> 4)) as u8, ((b << 3) | (b >> 2)) as u8, 255]);
        }
    }
}

/// One held snapshot as a reel sees it.
#[derive(Clone, Debug)]
pub struct Moment {
    pub frame: i64,
    pub core_frames: i64,
    pub thumbnail: Option<Thumbnail>,
}

/// An encoding handed to the worker and handed back: the earlier state, the newer, and the delta's buffer.
struct Job {
    previous: Vec<u8>,
    state: Vec<u8>,
    delta: Vec<u8>,
}

#[derive(Default)]
struct Slots {
    asked: Option<Job>,
    done: Option<Job>,
    quit: bool,
}

/// The one-slot mailbox between the session's frame thread and its encoding thread.
#[derive(Default)]
struct Mailbox {
    slots: Mutex<Slots>,
    ready: Condvar,
}

struct Worker {
    mail: Arc<Mailbox>,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn start() -> Worker {
        let mail = Arc::new(Mailbox::default());
        let theirs = mail.clone();
        let thread = std::thread::Builder::new()
            .name("crystal-rewind".to_string())
            .spawn(move || {
                loop {
                    let mut job = {
                        let mut s = theirs.slots.lock().unwrap_or_else(|p| p.into_inner());
                        loop {
                            if s.quit {
                                return;
                            }
                            if let Some(job) = s.asked.take() {
                                break job;
                            }
                            s = theirs.ready.wait(s).unwrap_or_else(|p| p.into_inner());
                        }
                    };
                    encode(&job.previous, &job.state, &mut job.delta);
                    theirs.slots.lock().unwrap_or_else(|p| p.into_inner()).done = Some(job);
                    theirs.ready.notify_all();
                }
            })
            .ok();
        Worker { mail, thread }
    }

    fn ask(&mut self, job: Job) {
        if self.thread.is_none() {
            let mut job = job;
            encode(&job.previous, &job.state, &mut job.delta);
            self.mail.slots.lock().unwrap_or_else(|p| p.into_inner()).done = Some(job);
            return;
        }
        self.mail.slots.lock().unwrap_or_else(|p| p.into_inner()).asked = Some(job);
        self.mail.ready.notify_all();
    }

    fn wait(&self) -> Job {
        let mut s = self.mail.slots.lock().unwrap_or_else(|p| p.into_inner());
        loop {
            if let Some(job) = s.done.take() {
                return job;
            }
            s = self.mail.ready.wait(s).unwrap_or_else(|p| p.into_inner());
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.mail.slots.lock().unwrap_or_else(|p| p.into_inner()).quit = true;
        self.mail.ready.notify_all();
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

/// What rewind asks of the engine: a snapshot into a buffer, a state loaded, the frames it has run.
pub trait Snapshots {
    /// The snapshot's bytes into `into`, sized to it; 0 where the engine keeps no state.
    fn snapshot(&mut self, into: &mut Vec<u8>) -> Result<usize, (i32, String)>;
    fn load(&mut self, state: &[u8]) -> Result<(), (i32, String)>;
    fn total_frames(&self) -> i64;
}

pub struct Rewind {
    pub enabled: bool,
    interval: i32,
    pub budget: i64,
    pub thumbnail_width: i32,
    pub thumbnail_budget: i64,
    /// Oldest first; each turns its successor's state back into its predecessor's.
    deltas: VecDeque<Vec<u8>>,
    delta_bytes: i64,
    /// Deltas' buffers the chain let go, used again.
    free: Vec<Vec<u8>>,
    newest: Option<Vec<u8>>,
    newest_len: usize,
    spare: Option<Vec<u8>>,
    encoding: bool,
    since_capture: i32,
    frames: i64,
    moments: VecDeque<Moment>,
    thumbnail_bytes: i64,
    worker: Option<Worker>,
}

impl Default for Rewind {
    fn default() -> Rewind {
        Rewind {
            enabled: false,
            interval: DEFAULT_INTERVAL_FRAMES,
            budget: DEFAULT_BUDGET_BYTES,
            thumbnail_width: 0,
            thumbnail_budget: DEFAULT_THUMBNAIL_BUDGET_BYTES,
            deltas: VecDeque::new(),
            delta_bytes: 0,
            free: Vec::new(),
            newest: None,
            newest_len: 0,
            spare: None,
            encoding: false,
            since_capture: 0,
            frames: 0,
            moments: VecDeque::new(),
            thumbnail_bytes: 0,
            worker: None,
        }
    }
}

impl Rewind {
    pub fn interval(&self) -> i32 {
        self.interval
    }

    pub fn set_interval(&mut self, frames: i32) {
        self.interval = frames.max(1);
    }

    /// Frames completed while on, rewound with the chain; a moment's frame is this at its capture.
    pub fn frame(&self) -> i64 {
        self.frames
    }

    /// Steps available now, counting the one still encoding.
    pub fn depth(&self) -> i64 {
        self.deltas.len() as i64 + self.encoding as i64
    }

    /// The bytes held; a delta still encoding joins when it settles.
    pub fn buffered_bytes(&self) -> i64 {
        self.delta_bytes + if self.has_newest() { self.newest_len as i64 } else { 0 }
    }

    pub fn snapshot_bytes(&self) -> i64 {
        if self.has_newest() { self.newest_len as i64 } else { 0 }
    }

    fn has_newest(&self) -> bool {
        self.newest.is_some() || self.encoding
    }

    pub fn buffered_seconds(&self, hz: f64) -> f64 {
        if hz <= 0.0 { 0.0 } else { self.depth() as f64 * self.interval as f64 / hz }
    }

    /// Once per completed frame; true when it took a snapshot, whose picture the caller may attach.
    pub fn on_frame_completed(&mut self, e: &mut dyn Snapshots) -> Result<bool, (i32, String)> {
        if !self.enabled {
            return Ok(false);
        }
        self.frames += 1;
        self.since_capture += 1;
        if self.since_capture < self.interval {
            return Ok(false);
        }
        self.since_capture = 0;
        self.capture(e)
    }

    /// A snapshot now, whatever the interval: the chain's seed.
    pub fn capture_now(&mut self, e: &mut dyn Snapshots) -> Result<bool, (i32, String)> {
        self.capture(e)
    }

    fn capture(&mut self, e: &mut dyn Snapshots) -> Result<bool, (i32, String)> {
        self.settle();
        let mut state = self.spare.take().unwrap_or_default();
        let length = e.snapshot(&mut state)?;
        if length == 0 {
            self.spare = Some(state);
            self.clear();
            return Ok(false);
        }
        let moment = Moment { frame: self.frames, core_frames: e.total_frames(), thumbnail: None };
        match self.newest.take() {
            Some(previous) if previous.len() == length => {
                let delta = self.free.pop().unwrap_or_default();
                self.newest_len = length;
                self.encoding = true;
                self.worker.get_or_insert_with(Worker::start).ask(Job { previous, state, delta });
            }
            _ => {
                self.clear();
                self.newest_len = length;
                self.newest = Some(state);
            }
        }
        self.moments.push_back(moment);
        Ok(true)
    }

    /// The picture of the snapshot just taken; false when pictures are off or it has one.
    pub fn attach_thumbnail(&mut self, rgba: &[u8], width: i32, rows: i32, row_repeat: i32) -> bool {
        if self.thumbnail_width <= 0 || !matches!(self.moments.back(), Some(Moment { thumbnail: None, .. })) {
            return false;
        }
        let Some(t) = Thumbnail::from(rgba, width, rows, row_repeat, self.thumbnail_width) else { return false };
        self.thumbnail_bytes += t.bytes();
        if let Some(m) = self.moments.back_mut() {
            m.thumbnail = Some(t);
        }
        self.trim_thumbnails();
        true
    }

    /// Oldest first, the chain settled.
    pub fn moments(&mut self) -> &VecDeque<Moment> {
        self.settle();
        &self.moments
    }

    /// The step back to the snapshot before: the chain's newest delta applied and the state loaded; false once the
    /// chain is spent.
    pub fn step(&mut self, e: &mut dyn Snapshots) -> Result<bool, (i32, String)> {
        self.settle();
        let (Some(newest), Some(delta)) = (self.newest.as_mut(), self.deltas.back()) else { return Ok(false) };
        apply(newest, delta).map_err(invalid)?;
        let delta = self.deltas.pop_back().expect("held");
        self.delta_bytes -= delta.len() as i64;
        self.free.push(delta);
        self.drop_newest_moment();
        e.load(self.newest.as_ref().expect("held"))?;
        self.since_capture = 0;
        self.frames = self.moments.back().map_or(0, |m| m.frame);
        Ok(true)
    }

    /// Straight to the moment of `frame`, every newer one dropped; false where none is held.
    pub fn to(&mut self, e: &mut dyn Snapshots, frame: i64) -> Result<bool, (i32, String)> {
        self.settle();
        let Some(steps) = self.steps_to(frame) else { return Ok(false) };
        for _ in 0..steps {
            let delta = self.deltas.pop_back().expect("held");
            self.delta_bytes -= delta.len() as i64;
            apply(self.newest.as_mut().expect("held"), &delta).map_err(invalid)?;
            self.free.push(delta);
            self.drop_newest_moment();
        }
        e.load(self.newest.as_ref().expect("held"))?;
        self.since_capture = 0;
        self.frames = frame;
        Ok(true)
    }

    fn steps_to(&self, frame: i64) -> Option<usize> {
        self.newest.as_ref()?;
        self.moments.iter().rev().position(|m| m.frame == frame)
    }

    fn drop_newest_moment(&mut self) {
        if let Some(m) = self.moments.pop_back() {
            self.thumbnail_bytes -= m.thumbnail.map_or(0, |t| t.bytes());
        }
    }

    fn drop_oldest_moment(&mut self) {
        if let Some(m) = self.moments.pop_front() {
            self.thumbnail_bytes -= m.thumbnail.map_or(0, |t| t.bytes());
        }
    }

    /// Every second picture of the older half goes, so the reel keeps its reach and loses density with age.
    fn trim_thumbnails(&mut self) {
        while self.thumbnail_bytes > self.thumbnail_budget {
            let pictured: Vec<usize> = self.moments.iter().enumerate().filter(|(_, m)| m.thumbnail.is_some()).map(|(i, _)| i).collect();
            if pictured.is_empty() {
                return;
            }
            let older = (pictured.len() / 2).max(1);
            let mut i = if older >= 2 { 1 } else { 0 };
            while i < older {
                if let Some(t) = self.moments[pictured[i]].thumbnail.take() {
                    self.thumbnail_bytes -= t.bytes();
                }
                i += 2;
            }
        }
    }

    /// The delta an earlier capture left encoding taken into the chain, so the chain is whole before it is moved.
    fn settle(&mut self) {
        if !self.encoding {
            return;
        }
        self.encoding = false;
        let job = self.worker.as_ref().expect("an encoding has a worker").wait();
        self.newest = Some(job.state);
        self.spare = Some(job.previous);
        self.delta_bytes += job.delta.len() as i64;
        self.deltas.push_back(job.delta);
        self.trim_to_budget();
    }

    /// Everything let go, as on any jump that is not a step back.
    pub fn clear(&mut self) {
        self.settle();
        self.free.extend(self.deltas.drain(..));
        self.delta_bytes = 0;
        if let Some(n) = self.newest.take() {
            self.spare.get_or_insert(n);
        }
        self.newest_len = 0;
        self.since_capture = 0;
        self.moments.clear();
        self.thumbnail_bytes = 0;
    }

    /// The oldest deltas dropped, costing reach and nothing else.
    fn trim_to_budget(&mut self) {
        while self.delta_bytes > self.budget {
            let Some(d) = self.deltas.pop_front() else { break };
            self.delta_bytes -= d.len() as i64;
            self.free.push(d);
            self.drop_oldest_moment();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_codec_writes_the_c_sharp_bytes_and_undoes_itself() {
        let a: Vec<u8> = (0..300u32).map(|i| i as u8).collect();
        let mut b = a.clone();
        b[3] ^= 0xFF;
        b[200..210].iter_mut().for_each(|x| *x = 7);
        let mut d = Vec::new();
        encode(&a, &b, &mut d);
        assert_eq!(&d[..5], &[3, 1, 0xFF, 0xC4, 0x01]);
        let mut c = b.clone();
        apply(&mut c, &d).unwrap();
        assert_eq!(c, a);
        encode(&a, &a, &mut d);
        assert!(d.is_empty());
        assert!(apply(&mut c, &[5, 9, 1]).is_err());
    }
}
