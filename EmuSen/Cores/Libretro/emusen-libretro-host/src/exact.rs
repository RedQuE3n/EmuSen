//! Core options whose own default changes what the player gets: each core's named exceptions, given as EmuSen's default in
//! place of the core's, the player free to choose the core's (EmuSen_LibretroHost.md §10p).

use crate::console_picture::{Exception, parse};

const DATA: &str = include_str!("../data/exact-defaults.tsv");

/// The exact defaults of the core with this file stem, from the data file, in its order.
pub fn defaults(stem: &str) -> Vec<Exception> {
    parse(DATA, stem)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_data_names_flycasts_threaded_rendering() {
        let e = defaults("flycast");
        assert_eq!(e.iter().map(|x| (x.key.as_str(), x.value.as_str())).collect::<Vec<_>>(), [("reicast_threaded_rendering", "disabled")]);
        assert!(e.iter().all(|x| x.decided == "2026-11-01" && !x.reason.is_empty()));
        assert!(defaults("pcee2").is_empty());
    }
}
