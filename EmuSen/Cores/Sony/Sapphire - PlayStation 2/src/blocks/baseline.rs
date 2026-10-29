//! The baseline's helpers, which its stubs call in the helper convention (EmuSen_Recompiler.md §14.4)

use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

use super::{Run, fill, replay_steps, watch, watched};
use crate::ee::jit::Context;
use crate::ee::jit::baseline::Rec;

/// Entries and steps into the blocks the baseline covers, and those that ran its static form.
pub(crate) static COUNTS: [AtomicU64; 4] = [const { AtomicU64::new(0) }; 4];

/// The helpers' addresses, in the stubs' order: exit, leave, commit, fill.
pub(crate) fn helpers() -> [u64; 4] {
    [exit as *const () as u64, leave as *const () as u64, commit as *const () as u64, fill_page as *const () as u64]
}

/// One entry into a covered block, which ran its static form or not, and its steps.
pub(crate) fn count(static_form: bool, steps: u32) {
    COUNTS[0].fetch_add(1, Relaxed);
    COUNTS[1].fetch_add(u64::from(steps), Relaxed);
    if static_form {
        COUNTS[2].fetch_add(1, Relaxed);
        COUNTS[3].fetch_add(u64::from(steps), Relaxed);
    }
}

/// `count`'s totals: covered entries and steps, static entries and steps.
pub fn counts() -> [u64; 4] {
    std::array::from_fn(|k| COUNTS[k].load(Relaxed))
}

/// The steps' cycles from the record, as the code would have stored them, and the guard, base and log count.
fn prepare(ctx: &mut Context, rec: &Rec) -> u32 {
    // SAFETY: called by compiled code during `block`, whose `Run` is alive and borrowed by nothing else.
    let sys = unsafe { &*(*ctx.owner.cast::<Run>()).sys };
    let last0 = sys.ee.timing.last();
    let steps = u32::from(rec.steps);
    for (t, &k) in ctx.times.iter_mut().zip(&rec.times).take(steps as usize) {
        *t = last0 + u64::from(k);
    }
    let base = u32::from(rec.base);
    (ctx.guard, ctx.base, ctx.logged) = (base.saturating_sub(1), base, u32::from(rec.logged));
    steps
}

/// An exit: `why` counted where it is one, the board replayed, and with every step standing the record's commit.
extern "C" fn exit(ctx: *mut Context, rec: u64, x: u64, why: u64) -> u64 {
    // SAFETY: the context is the dispatcher's, alive for the call; the record is the block's data, 8-aligned.
    let (ctx, rec) = unsafe { (&mut *ctx, &*(rec as *const Rec)) };
    if why != u64::MAX {
        ctx.left[why as usize] += 1;
    }
    let steps = prepare(ctx, rec);
    let seen = watch(ctx);
    let kept = replay_steps(ctx, steps, false);
    watched(seen, ctx, 0);
    if kept == steps {
        // SAFETY: as `prepare`.
        let sys = unsafe { &mut *(*ctx.owner.cast::<Run>()).sys };
        sys.ee.commit_static(rec, x);
        ctx.steps = steps;
    }
    u64::from(kept)
}

/// A load's or store's leave after some steps (`super::leave`).
extern "C" fn leave(ctx: *mut Context, rec: u64, why: u64, _: u64) -> u64 {
    // SAFETY: as `exit`.
    let (ctx, rec) = unsafe { (&mut *ctx, &*(rec as *const Rec)) };
    ctx.left[why as usize] += 1;
    let steps = prepare(ctx, rec);
    let seen = watch(ctx);
    let kept = replay_steps(ctx, steps, true);
    watched(seen, ctx, 1);
    u64::from(kept)
}

/// The commit of the state after the last load or store, which a rollback past it writes back.
extern "C" fn commit(ctx: *mut Context, rec: u64, x: u64, _: u64) -> u64 {
    // SAFETY: as `exit`.
    let (ctx, rec) = unsafe { (&mut *ctx, &*(rec as *const Rec)) };
    let sys = unsafe { &mut *(*ctx.owner.cast::<Run>()).sys };
    sys.ee.commit_static(rec, x);
    0
}

extern "C" fn fill_page(ctx: *mut Context, vaddr: u64, _: u64, _: u64) -> u64 {
    u64::from(fill(ctx, vaddr as u32))
}
