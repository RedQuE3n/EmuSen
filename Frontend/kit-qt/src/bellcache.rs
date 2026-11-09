//! The cues rendered once each and kept, with no static and no Qt (EmuSen_Kit.md §7.5, §5).

use std::collections::HashMap;

use emusen_kit_core::sound::{self, Cue, RATE};

#[derive(Clone, Debug, Default)]
pub struct BellCache {
    cues: HashMap<Cue, Vec<f32>>,
    quick_select: Option<Vec<f32>>,
    rungs: Vec<Vec<f32>>,
    renders: u32,
}

impl BellCache {
    pub fn renders(&self) -> u32 {
        self.renders
    }

    /// A cue's 48 kHz stereo frames, rendered the first time only.
    pub fn cue(&mut self, cue: Cue) -> &[f32] {
        let renders = &mut self.renders;
        self.cues.entry(cue).or_insert_with(|| {
            *renders += 1;
            cue.samples()
        })
    }

    /// The bells of the ladder's first `count` rungs, rendered the first time only (a splash's or a picker's notes).
    pub fn ladder(&mut self, count: usize) -> usize {
        while self.rungs.len() < count {
            self.rungs.push(sound::bell_samples(self.rungs.len()));
            self.renders += 1;
        }
        count
    }

    /// One rung's bell, rendered the first time only.
    pub fn rung(&mut self, rung: usize) -> &[f32] {
        self.ladder(rung + 1);
        &self.rungs[rung]
    }

    pub fn quick_select(&mut self) -> &[f32] {
        if self.quick_select.is_none() {
            self.quick_select = Some(sound::quick_select_samples());
            self.renders += 1;
        }
        self.quick_select.as_deref().unwrap_or(&[])
    }

    /// How long stereo frames last, in milliseconds.
    pub fn duration_ms(samples: &[f32]) -> i32 {
        ((samples.len() / 2) as f64 * 1000.0 / f64::from(RATE)).round() as i32
    }
}
