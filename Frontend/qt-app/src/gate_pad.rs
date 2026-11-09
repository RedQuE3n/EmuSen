//! The gates' pad, built only with the `gate-pad` feature (EmuSen_Qt.md §9.3.6): an SDL virtual pad inside this process,
//! pressed by lines read from the FIFO `EMUSEN_GATE_PAD` names, so no input device exists outside the program.

use emusen_kit_core::input::PadControl;
use emusen_qt_endymion::reading::sdl_button;
use emusen_qt_test::virtual_pad::Sdl;
use std::io::BufRead;

/// The environment variable naming the FIFO; unset, no pad is made.
pub const FIFO: &str = "EMUSEN_GATE_PAD";

/// One line of the FIFO: a control by its name in `PadControl` and 1 (down) or 0 (up), as "Right 1".
pub fn parse(line: &str) -> Option<(PadControl, bool)> {
    let mut words = line.split_whitespace();
    let name = words.next()?;
    let control = PadControl::ALL.into_iter().find(|c| format!("{c:?}") == name)?;
    let down = match words.next()? {
        "1" => true,
        "0" => false,
        _ => return None,
    };
    words.next().is_none().then_some((control, down))
}

/// When `EMUSEN_GATE_PAD` names a FIFO: the pad attached, a thread pressing it line by line until the writer closes, and
/// the pad's id for `onlyPads`, so the window reads this pad alone.
pub fn start() -> Option<String> {
    let path = std::env::var_os(FIFO)?;
    let sdl = Sdl::open();
    let pad = sdl.attach("EmuSen gate pad", 0x1209, 0x0001);
    let id = pad.id.to_string();
    std::thread::spawn(move || {
        let Ok(fifo) = std::fs::File::open(&path) else {
            eprintln!("gate pad: {} cannot be read", path.to_string_lossy());
            return;
        };
        for line in std::io::BufReader::new(fifo).lines().map_while(Result::ok) {
            match parse(&line).and_then(|(control, down)| Some((sdl_button(control)?, down))) {
                Some((button, down)) => sdl.press(&pad, button, down),
                None => eprintln!("gate pad: not a press: {line}"),
            }
        }
    });
    Some(id)
}
