//! The game streams MarsNativeRdpTests recorded, replayed here with overflow checks on, against the C# processor's final bytes.

use std::path::Path;

use super::{Rdp, RdpMemory};
use crate::state::{State, StateReader, StateWriter};

/// Where MarsNativeRdpTests keeps its copies of the games and the streams; absent, the test passes unrun.
const FOLDER: &str = "EMUSEN_MARSRT_RDP";
const STREAM: u32 = 0x5344_5052;
const EXPECTED: u32 = 0x5844_5052;

struct Cursor<'a> {
    data: &'a [u8],
    at: usize,
}

impl<'a> Cursor<'a> {
    fn u32(&mut self) -> u32 {
        let v = u32::from_le_bytes(self.data[self.at..self.at + 4].try_into().unwrap());
        self.at += 4;
        v
    }

    fn bytes(&mut self, n: usize) -> &'a [u8] {
        let v = &self.data[self.at..self.at + n];
        self.at += n;
        v
    }
}

fn replay(stream: &Path, expected: &Path) {
    let data = std::fs::read(stream).unwrap();
    let mut s = Cursor { data: &data, at: 0 };
    assert_eq!(s.u32(), STREAM, "{} is not a stream", stream.display());
    let (rdram, hidden, state, frames, words) = (s.u32() as usize, s.u32() as usize, s.u32() as usize, s.u32() as usize, s.u32() as usize);
    let mut memory_rdram = s.bytes(rdram).to_vec();
    let mut memory_hidden = s.bytes(hidden).to_vec();
    let mut rdp = Rdp::default();
    rdp.read_state(&mut StateReader::new(s.bytes(state))).unwrap();
    s.bytes(frames * 4);
    let words: Vec<u64> = s.bytes(words * 8).as_chunks::<8>().0.iter().map(|c| u64::from_le_bytes(*c)).collect();

    let mut memory = RdpMemory::new(&mut memory_rdram, &mut memory_hidden);
    let syncs = words.iter().filter(|&&w| rdp.accept(w, &mut memory)).count();
    drop(memory);

    let data = std::fs::read(expected).unwrap();
    let mut e = Cursor { data: &data, at: 0 };
    assert_eq!(e.u32(), EXPECTED);
    let (rdram, hidden, state) = (e.u32() as usize, e.u32() as usize, e.u32() as usize);
    assert!(e.bytes(rdram) == &memory_rdram[..], "{}: RDRAM differs from C#'s", stream.display());
    assert!(e.bytes(hidden) == &memory_hidden[..], "{}: hidden RDRAM differs from C#'s", stream.display());

    let mut counter = StateWriter::counter();
    rdp.write_state(&mut counter);
    let mut ours = vec![0u8; counter.len()];
    rdp.write_state(&mut StateWriter::new(&mut ours));
    assert!(e.bytes(state) == &ours[..], "{}: state differs from C#'s", stream.display());
    eprintln!("{}: {} words, {syncs} full syncs, identical to C# at the end", stream.display(), words.len());
}

#[test]
fn recorded_game_streams_end_where_the_csharp_processor_ended() {
    let Ok(folder) = std::env::var(FOLDER) else {
        eprintln!("{FOLDER} unset, not run");
        return;
    };
    let mut streams: Vec<_> = std::fs::read_dir(&folder).unwrap().filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().is_some_and(|x| x == "rdp")).collect();
    streams.sort();
    for stream in streams {
        let expected = stream.with_extension("rdp.expected");
        if expected.exists() {
            replay(&stream, &expected);
        }
    }
}
