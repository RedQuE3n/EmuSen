//! The system view's gameselector (THEMES.md, gameselector; EmuSen_Qt.md §4.8.6): which games each selector picks from
//! the shown system, and which pick an element of the view draws.

use crate::loader::{Element, View};

use super::data::System;

/// splitmix64: a small, well-mixed sequence from one seed, so a random pick is the same for the same seed.
struct Mix(u64);

impl Mix {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

/// The games one selector picks, as indices into the system's games: the last played, the most played, or a shuffle by
/// `seed`, never a folder or a game excluded from the counter; `allowDuplicates` fills a short shuffle with repeats.
pub fn pick(e: &Element, system: &System, seed: u64) -> Vec<usize> {
    let count = e.uint("gameCount").unwrap_or(1).clamp(1, 30) as usize;
    let mut pool: Vec<usize> = (0..system.games.len()).filter(|&i| !system.games[i].folder && !system.games[i].not_counted).collect();
    match e.string("selection") {
        Some("lastplayed") => {
            pool.retain(|&i| system.games[i].last_played.is_some());
            pool.sort_by(|a, b| system.games[*b].last_played.cmp(&system.games[*a].last_played));
            pool.truncate(count);
            pool
        }
        Some("mostplayed") => {
            pool.retain(|&i| system.games[i].play_count > 0);
            pool.sort_by(|a, b| system.games[*b].play_count.cmp(&system.games[*a].play_count));
            pool.truncate(count);
            pool
        }
        _ => {
            let mut mix = Mix(seed ^ super::draw::key(&[&e.name]));
            let mut shuffled = pool.clone();
            for i in (1..shuffled.len()).rev() {
                shuffled.swap(i, mix.below(i + 1));
            }
            shuffled.truncate(count);
            if e.bool("allowDuplicates") == Some(true) {
                while shuffled.len() < count && !pool.is_empty() {
                    shuffled.push(pool[mix.below(pool.len())]);
                }
            }
            shuffled
        }
    }
}

/// The game an element of the system view draws: its named selector's, else the selector whose name sorts first, as
/// ES-DE 3.4.1 was measured to choose (EmuSen_BigPicture.md §36); its `gameselectorEntry`, clamped to the selector's
/// count. None when the view has no selector or the entry is not filled, in which case ES-DE renders no element.
pub fn game_for(e: &Element, view: &View, system: &System, seed: u64) -> Option<usize> {
    let mut selectors: Vec<&Element> = view.of_kind("gameselector").collect();
    selectors.sort_by(|a, b| a.name.cmp(&b.name));
    let chosen = e.string("gameselector").and_then(|n| selectors.iter().find(|s| s.name == n)).or(selectors.first())?;
    let count = chosen.uint("gameCount").unwrap_or(1).clamp(1, 30);
    let entry = e.uint("gameselectorEntry").unwrap_or(0).min(count - 1) as usize;
    pick(chosen, system, seed).get(entry).copied()
}
