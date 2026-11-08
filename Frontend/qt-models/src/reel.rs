//! The rewind reel as the Pause Card shows it (EmuSen_Interface.md §18.5, §20.1; EmuSen_Qt.md §7.3): the moments the
//! runtime holds with pictures, newest first from the present, the strip's stops, Left and Right, the shoulders and the
//! triggers over them, and the words. Mistress's `MainWindow.Rewind`, restated as plain Rust, so both front ends agree.

/// Two snapshots a second of console time, as Mistress keeps them (EmuSen_Settings_Reference.md §4.112).
pub const SNAPSHOTS_PER_SECOND: f64 = 2.0;

/// The reel's pictures ride along with each snapshot this many pixels wide (EmuSen_Settings_Reference.md §4.49).
pub const PICTURE_WIDTH: u32 = 160;

/// The strip's stops after the present, in seconds back.
pub const STOP_SECONDS: [f64; 4] = [1.5, 4.0, 8.0, 12.0];

/// The shoulders' stride, in seconds.
pub const STRIDE_SECONDS: f64 = 5.0;

/// The frames between two snapshots for a game at `hz`.
pub fn interval_for(hz: f64) -> u32 {
    if hz > 0.0 { (hz / SNAPSHOTS_PER_SECOND).round().max(1.0) as u32 } else { 30 }
}

/// Rewind kept for a session as Mistress keeps it for every game (`WireSession`): a snapshot every half second of
/// console time, the history's budget the runtime's own, each snapshot with its picture.
pub fn keep(runtime: &emusen_crystal_client::Runtime, session: u32) -> emusen_crystal_client::Result<()> {
    let hz = runtime.session_info(session)?.frame_rate_hz;
    runtime.rewind_set(session, true, interval_for(hz), 0, PICTURE_WIDTH)
}

/// One moment of the reel: its frame and how far back it is; the present is `now`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mark {
    pub frame: u64,
    pub seconds_back: f64,
    pub now: bool,
}

impl Mark {
    /// How the moment is named on the rows and under the Rewind row's pictures.
    pub fn label(&self) -> String {
        if self.now { "Now".to_string() } else { ago(self.seconds_back) }
    }

    /// How the moment is named under its picture in the strip.
    pub fn short(&self) -> String {
        if self.now {
            "Now".to_string()
        } else if self.seconds_back < 10.0 {
            format!("{} s", tenths(self.seconds_back))
        } else {
            format!("{} s", self.seconds_back.floor())
        }
    }
}

/// So many seconds back, worded: "Now", "1.5 s ago", "28 s ago".
pub fn ago(seconds: f64) -> String {
    if seconds < 0.05 {
        "Now".to_string()
    } else if seconds < 10.0 {
        format!("{} s ago", tenths(seconds))
    } else {
        format!("{} s ago", seconds.floor())
    }
}

/// A number to a tenth, its ".0" dropped, as C#'s "0.#" writes it.
fn tenths(seconds: f64) -> String {
    let t = (seconds * 10.0).round() / 10.0;
    if t.fract() == 0.0 { format!("{t:.0}") } else { format!("{t:.1}") }
}

/// The present and the held moments with pictures before it, newest first; `held` is the frames of the moments with a
/// picture, `now` the session's frame count.
pub fn marks_of(held: &[u64], now: u64, hz: f64) -> Vec<Mark> {
    let hz = if hz > 0.0 { hz } else { 60.0 };
    let now = held.iter().copied().fold(now, u64::max);
    let mut marks = vec![Mark { frame: now, seconds_back: 0.0, now: true }];
    let mut before: Vec<u64> = held.iter().copied().filter(|&f| f < now).collect();
    before.sort_unstable_by(|a, b| b.cmp(a));
    marks.extend(before.into_iter().map(|f| Mark { frame: f, seconds_back: (now - f) as f64 / hz, now: false }));
    marks
}

/// The strip's stops as indexes of the marks: the present, the moments nearest 1.5, 4, 8 and 12 s back, and the oldest.
pub fn stops_of(marks: &[Mark]) -> Vec<usize> {
    let mut stops = vec![0];
    let Some(oldest) = marks.last() else { return stops };
    for seconds in STOP_SECONDS {
        if seconds >= oldest.seconds_back {
            break;
        }
        let nearest = (1..marks.len()).min_by(|&a, &b| (marks[a].seconds_back - seconds).abs().total_cmp(&(marks[b].seconds_back - seconds).abs()));
        if let Some(nearest) = nearest.filter(|&n| n > stops[stops.len() - 1]) {
            stops.push(nearest);
        }
    }
    if marks.len() - 1 > stops[stops.len() - 1] {
        stops.push(marks.len() - 1);
    }
    stops
}

/// The reel's state on its page: the marks, their stops, and the one focused.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Reel {
    pub marks: Vec<Mark>,
    pub stops: Vec<usize>,
    pub at: usize,
}

impl Reel {
    /// The reel opened on its first stop after the present; none where there is nothing before it.
    pub fn open(marks: Vec<Mark>) -> Option<Reel> {
        if marks.len() < 2 {
            return None;
        }
        let stops = stops_of(&marks);
        let at = stops.get(1).copied().unwrap_or(0);
        Some(Reel { marks, stops, at })
    }

    pub fn focused(&self) -> Option<&Mark> {
        self.marks.get(self.at)
    }

    /// Left (-1) or Right (+1): a press goes to the next stop that way, Right further back; held, one moment at a time.
    pub fn step(&mut self, by: i32, held: bool) {
        let last = self.marks.len().saturating_sub(1);
        if held {
            self.at = (self.at as i64 + by as i64).clamp(0, last as i64) as usize;
        } else if by > 0 {
            self.at = self.stops.iter().copied().find(|&i| i > self.at).unwrap_or(last);
        } else {
            self.at = self.stops.iter().copied().rev().find(|&i| i < self.at).unwrap_or(0);
        }
    }

    /// A shoulder: the nearest moment at least five seconds further that way (+1 back, -1 towards now), else the end.
    pub fn stride(&mut self, back: i32) {
        let Some(from) = self.focused().map(|m| m.seconds_back) else { return };
        let last = self.marks.len() - 1;
        let mut target = if back > 0 { last } else { 0 };
        let mut i = self.at as i64 + back as i64;
        while i >= 0 && i <= last as i64 {
            let s = self.marks[i as usize].seconds_back;
            if (back > 0 && s >= from + STRIDE_SECONDS) || (back < 0 && s <= from - STRIDE_SECONDS) {
                target = i as usize;
                break;
            }
            i += back as i64;
        }
        self.at = target;
    }

    /// A trigger: the oldest moment (+1) or the present (-1).
    pub fn end(&mut self, back: i32) {
        self.at = if back > 0 { self.marks.len().saturating_sub(1) } else { 0 };
    }

    /// The strip as shown: the stops, a moment scrubbed to between two of them standing in for the nearer, in order.
    pub fn strip(&self) -> Vec<usize> {
        let mut shown = self.stops.clone();
        if !shown.contains(&self.at) && shown.len() > 1 {
            let nearer = (1..shown.len()).min_by_key(|&i| shown[i].abs_diff(self.at)).unwrap_or(1);
            shown[nearer] = self.at;
            shown.sort_unstable();
        }
        shown
    }

    /// The first row's words: what A does.
    pub fn go_back_words(&self) -> String {
        match self.focused() {
            Some(m) if !m.now => format!("Go back to {}", m.label()),
            _ => "Stay at Now".to_string(),
        }
    }

    /// The pane's sentence under the strip.
    pub fn sentence(&self) -> String {
        match self.focused() {
            Some(m) if !m.now => format!("{}. A goes back there; the moments after it are kept until you play on.", m.label()),
            _ => "Now, where the game is paused. A closes the card and plays on from here.".to_string(),
        }
    }
}

/// The Rewind row's pane caption: what the reel holds and what A does.
pub fn held_words(seconds: f64) -> String {
    let held = if seconds >= 1.0 { format!("{} s available", seconds.floor()) } else { "Under a second available".to_string() };
    format!("{held}. A opens the reel; Left and Right step through it.")
}
