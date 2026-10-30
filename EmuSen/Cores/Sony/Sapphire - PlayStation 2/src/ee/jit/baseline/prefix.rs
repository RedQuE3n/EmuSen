//! The dynamic prefix, run by the shared exit for an entry not at rest (EmuSen_Recompiler.md §15.2)

use super::super::super::timing::{CATEGORIES, Issue, Snap, Timing, UNITS, halves, whole};
use super::super::shape;
use super::Static;

/// The most steps the prefix tests for convergence; past them the model runs to the exit.
pub(crate) const PMAX: usize = 8;

/// A block's table, laid first in its data: this head, the static form's cycles after each step, its steps, what the rest reads first from each step to `kmax`, and the static snapshot after steps 1 to `kmax`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Head {
    pub n: u8,
    pub kmax: u8,
    pub single: u8,
    pub blocking: u8,
    pub pad: u32,
}

/// One step as the issue model sees it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PStep {
    pub reads: [u64; 2],
    pub writes: [u64; 2],
    pub category: u8,
    pub row: u8,
    pub branch: u8,
    pub pad: [u8; 5],
}

/// A block's table, read in place.
#[derive(Clone, Copy)]
pub(crate) struct Table<'a> {
    pub head: Head,
    /// The cycles after each step less the entry's last issue, which every exit's record once held.
    pub times: &'a [u16],
    pub steps: &'a [PStep],
    pub exposed: &'a [[u64; 2]],
    pub snaps: &'a [Snap],
}

fn raw<T>(v: &T) -> &[u8] {
    // SAFETY: a plain `repr(C)` value read as its bytes.
    unsafe { std::slice::from_raw_parts((v as *const T).cast::<u8>(), size_of::<T>()) }
}

/// The table's bytes for a block's static form.
pub(crate) fn table_bytes(st: &Static, single: bool, blocking: bool) -> Vec<u8> {
    let head = Head { n: st.steps.len() as u8, kmax: st.kmax as u8, single: u8::from(single), blocking: u8::from(blocking), pad: 0 };
    let mut out = raw(&head).to_vec();
    for t in &st.times[..st.steps.len()] {
        out.extend_from_slice(&t.to_le_bytes());
    }
    out.resize(out.len().next_multiple_of(8), 0);
    for s in &st.steps {
        out.extend_from_slice(raw(s));
    }
    for e in &st.exposed_from[..=st.kmax] {
        for h in halves(*e) {
            out.extend_from_slice(&h.to_le_bytes());
        }
    }
    for k in 1..=st.kmax {
        out.extend_from_slice(raw(&st.snaps[k][0]));
    }
    out
}

impl Table<'_> {
    /// The table at `p`, which must be 8-aligned and hold `table_bytes`' output, alive and unchanged while the table is used.
    pub(crate) unsafe fn at<'a>(p: *const u8) -> Table<'a> {
        // SAFETY: as the caller promises; each part is 8-aligned in `table_bytes`' layout.
        unsafe {
            let head = *p.cast::<Head>();
            let (n, k) = (usize::from(head.n), usize::from(head.kmax));
            let times = p.add(size_of::<Head>()).cast::<u16>();
            let steps = p.add(size_of::<Head>() + (2 * n).next_multiple_of(8)).cast::<PStep>();
            let exposed = steps.add(n).cast::<[u64; 2]>();
            let snaps = exposed.add(k + 1).cast::<Snap>();
            Table { head, times: std::slice::from_raw_parts(times, n), steps: std::slice::from_raw_parts(steps, n), exposed: std::slice::from_raw_parts(exposed, k + 1), snaps: std::slice::from_raw_parts(snaps, k) }
        }
    }

    fn snap(&self, k: usize) -> Snap {
        if k == 0 { Snap::default() } else { self.snaps[k - 1] }
    }
}

/// What the prefix found: the step it converged at (or the exit's, where it did not) with the base it rebases by, its unpaired issue cycles, and the steps' cycles before it.
pub(crate) struct Prefix {
    pub at: usize,
    pub base: Option<u64>,
    pub unpaired: u8,
    pub times: [u64; shape::MAX],
}

/// The model `d`, the entry's, run over the steps up to the first where it converges with the static form, or to the exit's `j`; `d` is left there.
pub(crate) fn run(t: &Table, d: &mut Timing, j: usize, taken: bool) -> Prefix {
    let mut p = Prefix { at: j, base: None, unpaired: 0, times: [0; shape::MAX] };
    let kmax = usize::from(t.head.kmax);
    for k in 0..j {
        if k <= kmax
            && let Some(b) = d.converges(&t.snap(k), whole(t.exposed[k]))
        {
            (p.at, p.base) = (k, Some(b));
            return p;
        }
        let s = t.steps[k];
        let mut i = Issue::new(CATEGORIES[usize::from(s.category)], usize::from(s.row));
        (i.reads, i.writes) = (whole(s.reads) & !UNITS, whole(s.writes) & !UNITS);
        (i.taken, i.single, i.blocking) = (taken && s.branch != 0, t.head.single != 0, t.head.blocking != 0);
        let at = d.issue(i);
        p.unpaired += u8::from(!d.paired());
        p.times[k] = at + 1;
    }
    p
}

/// The core's model at an exit after `j` steps whose static snapshot is `s`: the prefix's own where it did not converge, otherwise `s` rebased with what only the prefix wrote kept from `d`; the unpaired issue cycles and whether the last paired.
pub(crate) fn model(core: &mut Timing, d: &Timing, p: &Prefix, t: &Table, s: &Snap, j: usize) -> (u8, bool) {
    let Some(b) = p.base else {
        let n = core.instructions;
        core.clone_from(d);
        core.instructions = n;
        return (p.unpaired, d.paired());
    };
    let c = p.at;
    let before = t.steps[..c].iter().fold(0, |m, s| m | whole(s.writes));
    let after = t.steps[c..j].iter().fold(0, |m, s| m | whole(s.writes));
    core.apply(b, s);
    let mut keep = before & !after;
    while keep != 0 {
        let r = keep.trailing_zeros() as usize;
        keep &= keep - 1;
        core.set_ready(r, d.ready_of(r));
    }
    (p.unpaired + s.unpaired - t.snap(c).unpaired, s.paired != 0)
}
