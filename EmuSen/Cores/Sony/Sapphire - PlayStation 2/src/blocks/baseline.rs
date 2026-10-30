//! The baseline's helpers, which its stubs call in the helper convention (EmuSen_Recompiler.md §14.4)

use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

use super::{Run, fill, replay_steps, watch, watched};
use crate::ee::jit::Context;
use crate::ee::jit::baseline::Rec;
use crate::ee::jit::baseline::prefix::{self, PMAX, Prefix, Table};
use crate::ee::timing::Timing;

/// Entries and steps into the blocks the baseline covers, those whose cycles were the static form's, the entries not at rest, then the prefix's length: converged at 0 to `PMAX`, then never.
pub(crate) static COUNTS: [AtomicU64; 5 + PMAX + 2] = [const { AtomicU64::new(0) }; 5 + PMAX + 2];

/// `Context::prefix`'s flag: the prefix ran to the exit without converging.
const NEVER: u32 = 1 << 31;

/// The helpers' addresses, in the stubs' order: exit, leave, commit, fill, call.
pub(crate) fn helpers() -> [u64; 5] {
    [exit as *const () as u64, leave as *const () as u64, commit as *const () as u64, fill_page as *const () as u64, call as *const () as u64]
}

/// One entry into a covered block, at rest or not, with the steps the prefix took and the steps that stood.
pub(crate) fn count(moved: bool, prefix: u32, steps: u32) {
    let ran = (prefix & !NEVER).min(steps);
    COUNTS[0].fetch_add(1, Relaxed);
    COUNTS[1].fetch_add(u64::from(steps), Relaxed);
    COUNTS[2].fetch_add(u64::from(!moved || ran == 0), Relaxed);
    COUNTS[3].fetch_add(u64::from(steps - ran), Relaxed);
    if moved && steps > 0 {
        COUNTS[4].fetch_add(1, Relaxed);
        let bucket = if prefix & NEVER != 0 { PMAX + 1 } else { ran as usize };
        COUNTS[5 + bucket].fetch_add(1, Relaxed);
    }
}

/// `count`'s totals: covered entries and steps, entries and steps in the static form.
pub fn counts() -> [u64; 4] {
    std::array::from_fn(|k| COUNTS[k].load(Relaxed))
}

/// The entries not at rest, then how many converged after 0 to `PMAX` steps, then those that never did before their exit.
pub fn prefix_counts() -> Vec<u64> {
    COUNTS[4..].iter().map(|c| c.load(Relaxed)).collect()
}

/// The run's machine, during `block`.
fn sys<'a>(ctx: &Context) -> &'a mut crate::board::System {
    // SAFETY: called by compiled code during `block`, whose `Run` is alive and borrowed by nothing else while its helpers run.
    unsafe { &mut *(*ctx.owner.cast::<Run>()).sys }
}

/// The block's table, at its data's start.
fn table<'a>(rec: &Rec) -> Table<'a> {
    // SAFETY: the record and the table are the same block's published data, 8-aligned, never written again.
    unsafe { Table::at((rec as *const Rec).cast::<u8>().sub(rec.table as usize)) }
}

/// The prefix run from the core's model to the exit's steps where the entry was not at rest, the model left in `d`.
fn prefix_of<'a>(ctx: &mut Context, rec: &Rec, taken: bool, d: &mut Option<Timing>) -> Option<(Prefix, Table<'a>)> {
    if ctx.moved == 0 {
        return None;
    }
    let t = table(rec);
    let m = d.insert(sys(ctx).ee.timing.clone());
    let p = prefix::run(&t, m, usize::from(rec.steps), taken);
    ctx.prefix = p.at as u32 | if p.base.is_none() { NEVER } else { 0 };
    Some((p, t))
}

/// The steps' cycles, from the prefix and then the table's rebased, and the guard, base and log count.
fn prepare(ctx: &mut Context, rec: &Rec, pre: Option<&Prefix>) -> u32 {
    let t = table(rec);
    let steps = u32::from(rec.steps);
    let (from, base) = match pre {
        None => (0, sys(ctx).ee.timing.last()),
        Some(p) => {
            ctx.times[..p.at].copy_from_slice(&p.times[..p.at]);
            (p.at, p.base.unwrap_or(0))
        }
    };
    for k in from..steps as usize {
        ctx.times[k] = base + u64::from(t.times[k]);
    }
    let b = u32::from(rec.base);
    (ctx.guard, ctx.base, ctx.logged) = (u32::from(rec.guard).saturating_sub(1), b, u32::from(rec.logged));
    steps
}

/// An exit: `why` counted where it is one, the board replayed, and with every step standing the record's commit.
extern "C" fn exit(ctx: *mut Context, rec: u64, x: u64, why: u64) -> u64 {
    // SAFETY: the context is the dispatcher's, alive for the call; the record is the block's data, 8-aligned.
    let (ctx, rec) = unsafe { (&mut *ctx, &*(rec as *const Rec)) };
    if why != u64::MAX {
        ctx.left[why as usize] += 1;
    }
    let mut d = None;
    let pre = prefix_of(ctx, rec, rec.place(x).3, &mut d);
    let steps = prepare(ctx, rec, pre.as_ref().map(|p| &p.0));
    let seen = watch(ctx);
    let kept = replay_steps(ctx, steps, false);
    watched(seen, ctx, 0);
    if kept == steps {
        let model = pre.as_ref().zip(d.as_ref()).map(|((p, t), d)| (p, d, t));
        sys(ctx).ee.commit_static(rec, x, model);
        ctx.steps = steps;
    }
    u64::from(kept)
}

/// A load's or store's leave after some steps (`super::leave`).
extern "C" fn leave(ctx: *mut Context, rec: u64, why: u64, _: u64) -> u64 {
    // SAFETY: as `exit`.
    let (ctx, rec) = unsafe { (&mut *ctx, &*(rec as *const Rec)) };
    ctx.left[why as usize] += 1;
    let mut d = None;
    let pre = prefix_of(ctx, rec, false, &mut d);
    let steps = prepare(ctx, rec, pre.as_ref().map(|p| &p.0));
    let seen = watch(ctx);
    let kept = replay_steps(ctx, steps, true);
    watched(seen, ctx, 1);
    u64::from(kept)
}

/// The commit of the state after the last load or store, which a rollback past it writes back.
extern "C" fn commit(ctx: *mut Context, rec: u64, x: u64, _: u64) -> u64 {
    // SAFETY: as `exit`.
    let (ctx, rec) = unsafe { (&mut *ctx, &*(rec as *const Rec)) };
    let mut d = None;
    let pre = prefix_of(ctx, rec, rec.place(x).3, &mut d);
    let model = pre.as_ref().zip(d.as_ref()).map(|((p, t), d)| (p, d, t));
    sys(ctx).ee.commit_static(rec, x, model);
    0
}

extern "C" fn fill_page(ctx: *mut Context, vaddr: u64, _: u64, _: u64) -> u64 {
    u64::from(fill(ctx, vaddr as u32))
}

/// A call's step: the values the block handed stored to the core with their undo, the operation's writes logged, and the interpreter's operation run (Sapphire_Speed.md §65).
extern "C" fn call(ctx: *mut Context, word: u64, logged: u64, index: u64) -> u64 {
    // SAFETY: as `exit`.
    let ctx = unsafe { &mut *ctx };
    ctx.calls += 1;
    let crate::board::System { ee, board } = sys(ctx);
    ee.call_in_block(board, ctx, word as u32, logged as usize, index as u16);
    0
}
