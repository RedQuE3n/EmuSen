//! The shell's screens (Diamond_Shell.md §3): Home, listing what the console offers with no disc, and Memory Cards, one
//! slot at a time, with a sheet over it for a save's actions, a card's, and the confirmation of what cannot be undone.
//! The controls are EmuSen's button grammar on the emulated pad (§3.2). A CD player is a second screen of Home's list,
//! added beside this one (§5.2).

use crate::cards::{self, Refusal, State};
use crate::drive::Held;
use crate::ui::{self, ACCENT, BRIGHT, CHOSEN, MUTED, PANEL, SHEET, TEXT, WARN};
use crate::{clock, drive, icons, pads, tty};
use diamond_guest::gpu::{fill, text};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Home,
    Cards,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Action {
    Copy,
    Delete,
    Format,
    Cancel,
}

/// A sheet over the list: its rows' actions and the one chosen; a confirmation starts on Cancel.
#[derive(Clone, Copy)]
struct Sheet {
    rows: [Action; 3],
    count: usize,
    chosen: usize,
    confirm: bool,
}

struct Shell {
    screen: Screen,
    slot: usize,
    chosen: [usize; 2],
    top: [usize; 2],
    sheet: Option<Sheet>,
    status: [u8; 52],
    held: Held,
    repeat: pads::Repeat,
}

const HOME_HINTS: &[u8] = b"Cross open";
const CARD_HINTS: &[u8] = b"Cross save  Triangle card  L1 R1 slot  Circle back";
const SHEET_HINTS: &[u8] = b"Cross choose  Circle cancel";

impl Shell {
    fn say(&mut self, words: &[&[u8]]) {
        self.status.fill(0);
        let mut n = 0;
        for w in words {
            for &c in w.iter().take_while(|&&c| c != 0) {
                if n < 51 {
                    self.status[n] = c;
                    n += 1;
                }
            }
        }
    }

    fn drive_words(&self) -> &'static [u8] {
        match self.held {
            Held::Audio => b"Audio CD",
            Held::Refused => b"Disc not recognised",
            Held::Data => b"Disc found, starting",
            Held::Empty | Held::Unknown => b"No disc",
        }
    }

    /// Each save's icon of the slot shown written to VRAM, once its saves are read.
    fn load_icons(&mut self) {
        let c = &cards::cards()[self.slot];
        for k in 0..c.count {
            icons::load(k, &c.saves[k].frames);
        }
    }

    fn draw(&mut self) {
        ui::frame(self.drive_words());
        match self.screen {
            Screen::Home => self.draw_home(),
            Screen::Cards => self.draw_cards(),
        }
        let hints = if self.sheet.is_some() { SHEET_HINTS } else if self.screen == Screen::Home { HOME_HINTS } else { CARD_HINTS };
        let status = self.status;
        ui::footer(&status, hints);
        if let Some(s) = self.sheet {
            self.draw_sheet(s);
        }
    }

    fn draw_home(&self) {
        text(16, 30, b"With no disc in the drive:", 40, MUTED);
        fill(8, 46, 304, 20, CHOSEN);
        text(16, 52, b"Memory Cards", 20, BRIGHT);
        text(16, 80, b"Put a PlayStation disc in to start it.", 50, MUTED);
    }

    fn draw_cards(&mut self) {
        for k in 0..2 {
            let x = 8 + 70 * k as u32;
            let on = k == self.slot;
            ui::panel(x, 24, 64, 16, if on { CHOSEN } else { PANEL });
            text(x + 14, 28, if k == 0 { b"Slot 1" } else { b"Slot 2" }, 6, if on { BRIGHT } else { TEXT });
        }
        let c = &cards::cards()[self.slot];
        let mut n = [0u8; 4];
        match c.state {
            State::Absent => ui::right(312, 28, b"No card", 20, MUTED),
            State::Blank => ui::right(312, 28, b"Not formatted", 20, WARN),
            State::Ready => {
                let free = ui::number(c.free, &mut n);
                let x = 312 - ui::width(b" blocks free", 12) - ui::width(free, 4);
                text(x, 28, free, 4, TEXT);
                text(x + ui::width(free, 4), 28, b" blocks free", 12, MUTED);
            }
        }
        if c.state != State::Ready {
            return;
        }
        if c.count == 0 {
            text(16, ui::LIST_Y + 6, b"No saves on this card.", 30, MUTED);
            return;
        }
        let chosen = self.chosen[self.slot];
        let top = &mut self.top[self.slot];
        if chosen < *top {
            *top = chosen;
        } else if chosen >= *top + ui::ROWS_SHOWN {
            *top = chosen + 1 - ui::ROWS_SHOWN;
        }
        let now = clock::frames();
        for i in *top..c.count.min(*top + ui::ROWS_SHOWN) {
            let s = &c.saves[i];
            let y = ui::LIST_Y + ui::ROW * (i - *top) as u32;
            if i == chosen {
                fill(8, y, 304, ui::ROW - 1, CHOSEN);
            }
            if icons::frames(&s.frames).is_some() {
                icons::draw(i, &s.frames, 12, y + 1, now);
                text(34, y + 2, &s.title, 40, if i == chosen { BRIGHT } else { TEXT });
            } else {
                text(34, y + 2, b"Damaged save", 20, WARN);
            }
            if let Some(serial) = serial(&s.name) {
                text(34, y + 11, serial, 10, MUTED);
            }
            let mut b = [0u8; 4];
            ui::right(306, y + 6, ui::number(s.blocks, &mut b), 2, MUTED);
        }
        if *top > 0 {
            ui::right(312, ui::LIST_Y - 6, b"^", 1, MUTED);
        }
        if *top + ui::ROWS_SHOWN < c.count {
            ui::right(312, ui::LIST_Y + ui::ROW * ui::ROWS_SHOWN as u32 - 4, b"v", 1, MUTED);
        }
    }

    fn sheet_line(&self, s: Sheet) -> ([u8; 52], [u8; 52]) {
        let mut a = [0u8; 52];
        let mut b = [0u8; 52];
        let c = &cards::cards()[self.slot];
        let title = c.saves.get(self.chosen[self.slot]).map_or(&[][..], |s| &s.title[..]);
        let put = |out: &mut [u8; 52], parts: &[&[u8]]| {
            let mut n = 0;
            for p in parts {
                for &ch in p.iter().take_while(|&&ch| ch != 0) {
                    if n < 46 {
                        out[n] = ch;
                        n += 1;
                    }
                }
            }
        };
        let slot: &[u8] = if self.slot == 0 { b"1" } else { b"2" };
        match (s.confirm, s.rows[0]) {
            (true, Action::Delete) => {
                put(&mut a, &[b"Delete ", title, b"?"]);
                put(&mut b, &[b"From Slot ", slot, b". It cannot be undone."]);
            }
            (true, Action::Format) => {
                put(&mut a, &[b"Format the card in Slot ", slot, b"?"]);
                put(&mut b, &[b"Every save on it is erased."]);
            }
            (false, Action::Format) => put(&mut a, &[b"The card in Slot ", slot]),
            _ => put(&mut a, &[title]),
        }
        (a, b)
    }

    fn draw_sheet(&self, s: Sheet) {
        let h = 44 + 16 * s.count as u32;
        let y = (ui::STATUS_Y - 8 - h) / 2 + 20;
        ui::panel(32, y, 256, h, SHEET);
        let (a, b) = self.sheet_line(s);
        text(42, y + 8, &a, 40, BRIGHT);
        text(42, y + 18, &b, 40, TEXT);
        for (i, &r) in s.rows[..s.count].iter().enumerate() {
            let ry = y + 34 + 16 * i as u32;
            if i == s.chosen {
                fill(40, ry - 3, 240, 14, CHOSEN);
            }
            let other: &[u8] = if self.slot == 0 { b"Copy to Slot 2" } else { b"Copy to Slot 1" };
            let words: &[u8] = match r {
                Action::Copy => other,
                Action::Delete => b"Delete",
                Action::Format => b"Format",
                Action::Cancel => b"Cancel",
            };
            text(48, ry, words, 20, if i == s.chosen { BRIGHT } else { ACCENT });
        }
    }

    fn reread(&mut self, slot: usize) {
        cards::refresh(slot);
        let n = cards::cards()[slot].count;
        self.chosen[slot] = self.chosen[slot].min(n.max(1) - 1);
        let state: &[u8] = match cards::cards()[slot].state {
            State::Absent => b"absent",
            State::Blank => b"blank",
            State::Ready => b"ready",
        };
        tty::line(&[b"shell: card", slot_digit(slot), state]);
        if slot == self.slot {
            self.load_icons();
        }
    }

    fn press(&mut self, p: u32) {
        if p & pads::START != 0 {
            self.sheet = None;
            self.screen = Screen::Home;
            return;
        }
        if let Some(mut s) = self.sheet {
            if p & pads::UP != 0 {
                s.chosen = s.chosen.saturating_sub(1);
            } else if p & pads::DOWN != 0 {
                s.chosen = (s.chosen + 1).min(s.count - 1);
            } else if p & pads::CIRCLE != 0 {
                self.sheet = None;
                self.say(&[]);
                return;
            } else if p & pads::CROSS != 0 {
                self.sheet = None;
                return self.act(s, s.rows[s.chosen]);
            }
            self.sheet = Some(s);
            return;
        }
        match self.screen {
            Screen::Home => {
                if p & pads::CROSS != 0 {
                    self.screen = Screen::Cards;
                    self.say(&[]);
                }
            }
            Screen::Cards => self.press_cards(p),
        }
    }

    fn press_cards(&mut self, p: u32) {
        let c = &cards::cards()[self.slot];
        let count = c.count;
        let ch = &mut self.chosen[self.slot];
        if p & pads::UP != 0 {
            *ch = ch.saturating_sub(1);
        } else if p & pads::DOWN != 0 {
            *ch = (*ch + 1).min(count.max(1) - 1);
        } else if p & pads::L2 != 0 {
            *ch = 0;
        } else if p & pads::R2 != 0 {
            *ch = count.max(1) - 1;
        } else if p & (pads::L1 | pads::R1) != 0 {
            let to = if p & pads::L1 != 0 { 0 } else { 1 };
            if to != self.slot {
                self.slot = to;
                self.load_icons();
            }
        } else if p & pads::CIRCLE != 0 {
            self.screen = Screen::Home;
        } else if p & pads::CROSS != 0 && c.state == State::Ready && count > 0 {
            self.sheet = Some(Sheet { rows: [Action::Copy, Action::Delete, Action::Cancel], count: 3, chosen: 0, confirm: false });
        } else if p & pads::TRIANGLE != 0 && c.state != State::Absent {
            self.sheet = Some(Sheet { rows: [Action::Format, Action::Cancel, Action::Cancel], count: 2, chosen: 0, confirm: false });
        }
    }

    fn act(&mut self, s: Sheet, a: Action) {
        let slot = self.slot;
        let i = self.chosen[slot];
        match a {
            Action::Cancel => self.say(&[]),
            Action::Copy => self.copy(slot, i),
            Action::Delete | Action::Format if !s.confirm => {
                self.sheet = Some(Sheet { rows: [a, Action::Cancel, Action::Cancel], count: 2, chosen: 1, confirm: true });
            }
            Action::Delete => {
                let name = cards::cards()[slot].saves[i].name;
                let ok = cards::erase(slot, &name);
                tty::line(&[b"shell: delete", slot_digit(slot), &name, if ok { b"ok" } else { b"failed" }]);
                self.reread(slot);
                self.say(&[if ok { b"Deleted." } else { b"The save could not be deleted." }]);
            }
            Action::Format => {
                self.say(&[b"Formatting... do not remove the card."]);
                self.draw();
                let ok = cards::format(slot);
                tty::line(&[b"shell: format", slot_digit(slot), if ok { b"ok" } else { b"failed" }]);
                self.reread(slot);
                self.say(&[if ok { b"Formatted." } else { b"The card could not be formatted." }]);
            }
        }
    }

    fn copy(&mut self, from: usize, i: usize) {
        let save = cards::cards()[from].saves[i];
        let to = 1 - from;
        self.say(&[b"Copying... do not remove either card."]);
        self.draw();
        let r = cards::copy(from, &save, |done, total| progress(done, total));
        let words: &[u8] = match r {
            Ok(()) => b"ok",
            Err(Refusal::NotReady) => b"not-ready",
            Err(Refusal::SameName) => b"same-name",
            Err(Refusal::TooFull) => b"too-full",
            Err(Refusal::Unreadable) => b"unreadable",
            Err(Refusal::Unwritable) => b"unwritable",
        };
        tty::line(&[b"shell: copy", slot_digit(from), &save.name, slot_digit(to), words]);
        if r.is_ok() || r == Err(Refusal::Unwritable) {
            cards::refresh(to);
        }
        let other: &[u8] = if to == 0 { b"Slot 1" } else { b"Slot 2" };
        match r {
            Ok(()) => self.say(&[b"Copied to ", other, b"."]),
            Err(Refusal::NotReady) => self.say(&[b"The card in ", other, b" is not ready."]),
            Err(Refusal::SameName) => self.say(&[b"The card in ", other, b" already holds that save."]),
            Err(Refusal::TooFull) => self.say(&[b"Not enough free blocks in ", other, b"."]),
            Err(Refusal::Unreadable) => self.say(&[b"The save could not be read."]),
            Err(Refusal::Unwritable) => self.say(&[b"The copy could not be written."]),
        }
    }
}

/// A copy's progress bar over the status line, a block at a time.
fn progress(done: u32, total: u32) {
    fill(8, ui::STATUS_Y + 9, 304, 3, PANEL);
    if total > 0 {
        fill(8, ui::STATUS_Y + 9, 304 * done / total, 3, ui::BAR);
    }
}

/// The drive's state as the TTY line names it.
fn drive_word(h: Held) -> &'static [u8] {
    match h {
        Held::Audio => b"audio",
        Held::Refused => b"refused",
        Held::Data => b"data",
        Held::Empty | Held::Unknown => b"empty",
    }
}

fn slot_digit(slot: usize) -> &'static [u8] {
    if slot == 0 { b"1" } else { b"2" }
}

/// The product code a commercial save's file name carries after its region (psx-spx "Filename Notes"): "SLUS-00067".
fn serial(name: &[u8; 21]) -> Option<&[u8]> {
    (name[0] == b'B' && name[6] == b'-' && name[2..12].iter().all(|&c| c != 0)).then(|| &name[2..12])
}

/// The shell's loop, a frame a step, until the drive holds a disc to boot, asked about every two seconds.
pub fn run(held: Held) {
    diamond_guest::gpu::init(ui::BACKGROUND);
    pads::start();
    clock::start();
    cards::start();
    let mut shell = Shell {
        screen: Screen::Home,
        slot: 0,
        chosen: [0; 2],
        top: [0; 2],
        sheet: None,
        status: [0; 52],
        held,
        repeat: pads::Repeat::new(),
    };
    tty::line(&[b"shell: drive", drive_word(held)]);
    shell.reread(0);
    shell.reread(1);
    shell.draw();
    loop {
        let ticks = clock::frame();
        let pressed = shell.repeat.step(pads::held());
        let animate = shell.screen == Screen::Cards && ticks % 11 == 0;
        if pressed != 0 {
            shell.press(pressed);
        }
        if ticks % 120 == 0 {
            match drive::identify() {
                Held::Data => {
                    shell.held = Held::Data;
                    shell.draw();
                    tty::line(&[b"shell: boot"]);
                    break;
                }
                Held::Unknown => {}
                h => {
                    if h != shell.held {
                        tty::line(&[b"shell: drive", drive_word(h)]);
                    }
                    shell.held = h;
                }
            }
            for k in 0..2 {
                if cards::changed(k) {
                    shell.reread(k);
                }
            }
        }
        if pressed != 0 || animate || ticks % 120 == 0 {
            shell.draw();
        }
    }
    cards::stop();
    clock::stop();
    pads::stop();
}
