//! One poll of Endymion's pads as the interface reads them, with no Qt and no thread (EmuSen_Kit.md §7.15): every
//! control of every pad the interface reads, turned into the kit's presses by kit-core's table, merged, and given as the
//! buttons that went down or up since the last poll; the help bar's family from the pad last used.

use emusen_kit_core::glyph::PadFamily;
use emusen_kit_core::input::{stick_direction, PadControl, UiButton, TRIGGER_THRESHOLD};
use emusen_kit_qt::router::PadEvent;
use emusen_platform_client::pads::{consts, Pads};
use emusen_platform_client::Result;

const BUTTONS: usize = UiButton::ALL.len();

/// SDL's numbers for the controls the interface reads (SDL_gamepad.h, SDL 3.4), which Endymion's calls take.
mod sdl {
    pub const LEFT_X: u32 = 0;
    pub const LEFT_Y: u32 = 1;
    pub const LEFT_TRIGGER: u32 = 4;
    pub const RIGHT_TRIGGER: u32 = 5;
}

/// The SDL_GamepadButton that is this control; none for a trigger, which is an axis.
pub fn sdl_button(control: PadControl) -> Option<i32> {
    Some(match control {
        PadControl::South => 0,
        PadControl::East => 1,
        PadControl::West => 2,
        PadControl::North => 3,
        PadControl::Select => 4,
        PadControl::Guide => 5,
        PadControl::Start => 6,
        PadControl::LeftStick => 7,
        PadControl::RightStick => 8,
        PadControl::LeftShoulder => 9,
        PadControl::RightShoulder => 10,
        PadControl::Up => 11,
        PadControl::Down => 12,
        PadControl::Left => 13,
        PadControl::Right => 14,
        PadControl::LeftTrigger | PadControl::RightTrigger => return None,
    })
}

/// The SDL_GamepadAxis that is a trigger.
fn sdl_trigger(control: PadControl) -> u32 {
    if control == PadControl::LeftTrigger {
        sdl::LEFT_TRIGGER
    } else {
        sdl::RIGHT_TRIGGER
    }
}

/// Endymion's family number as the kit's; a number this build does not know is the generic face.
pub fn family_of(number: i64) -> PadFamily {
    match u32::try_from(number) {
        Ok(consts::FAMILY_XBOX) => PadFamily::Xbox,
        Ok(consts::FAMILY_PLAYSTATION) => PadFamily::PlayStation,
        Ok(consts::FAMILY_NINTENDO) => PadFamily::Nintendo,
        _ => PadFamily::Generic,
    }
}

/// What one poll changed: the buttons in the kit's order, and the family and the count of pads read when they moved.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Tick {
    pub events: Vec<PadEvent>,
    pub family: Option<PadFamily>,
    pub connected: Option<usize>,
}

impl Tick {
    pub fn is_empty(&self) -> bool {
        self.events.is_empty() && self.family.is_none() && self.connected.is_none()
    }
}

/// What the reading keeps between polls.
#[derive(Clone, Debug, Default)]
pub struct Reading {
    held: [bool; BUTTONS],
    /// Each pad read at the last poll, and whether it held anything.
    active: Vec<(u64, bool)>,
    last_used: Option<u64>,
    family: Option<PadFamily>,
    connected: Option<usize>,
}

impl Reading {
    /// Starts the devices and reads the pads present.
    pub fn start(&mut self, pads: &Pads) -> Result<Tick> {
        pads.start()?;
        rest(pads)?;
        pads.update()?;
        self.read(pads)
    }

    /// One poll: the pads plugged in or pulled out since the last, the devices brought up to date (SDL's events pumped,
    /// as the C# poll ends), then every pad read.
    pub fn poll(&mut self, pads: &Pads) -> Result<Tick> {
        pads.poll()?;
        rest(pads)?;
        pads.update()?;
        self.read(pads)
    }

    fn read(&mut self, pads: &Pads) -> Result<Tick> {
        let open = pads.open()?;
        let count = usize::try_from(pads.get(0, consts::FRONTEND_COUNT, 0)?).unwrap_or(0).min(open.len());
        let read = &open[..count];
        let mut held = [false; BUTTONS];
        let mut active = Vec::with_capacity(read.len());
        for &key in read {
            let mine = held_on(pads, key)?;
            let any = mine.iter().any(|&h| h);
            let was = self.active.iter().any(|&(k, a)| k == key && a);
            if any && !was {
                self.last_used = Some(key);
            }
            active.push((key, any));
            held.iter_mut().zip(mine).for_each(|(h, m)| *h |= m);
        }
        self.active = active;
        let events = UiButton::ALL
            .iter()
            .filter(|b| held[b.index()] != self.held[b.index()])
            .map(|&button| PadEvent { button, down: held[button.index()] })
            .collect();
        self.held = held;
        let help = self.last_used.filter(|k| read.contains(k)).or_else(|| read.first().copied());
        let family = match help {
            Some(key) => family_of(pads.get(key, consts::FAMILY, 0)?),
            None => PadFamily::Generic,
        };
        let tick = Tick {
            events,
            family: (self.family != Some(family)).then_some(family),
            connected: (self.connected != Some(count)).then_some(count),
        };
        self.family = Some(family);
        self.connected = Some(count);
        Ok(tick)
    }
}

/// The changes a start or poll had no room for, taken until none is left; the reading asks the open pads afresh.
fn rest(pads: &Pads) -> Result<()> {
    while !pads.take()?.is_empty() {}
    Ok(())
}

/// The interface's buttons one pad holds: each control by kit-core's table, the left stick as the d-pad.
fn held_on(pads: &Pads, key: u64) -> Result<[bool; BUTTONS]> {
    let mut held = [false; BUTTONS];
    for control in PadControl::ALL {
        let down = match sdl_button(control) {
            Some(button) => pads.get(key, consts::RAW_PRESSED, button)? != 0,
            None => pads.axis(key, consts::AXIS_RAW, sdl_trigger(control))? > TRIGGER_THRESHOLD,
        };
        if let (true, Some(press)) = (down, control.press()) {
            held[press.index()] = true;
        }
    }
    let x = stick_direction(pads.axis(key, consts::AXIS_RAW, sdl::LEFT_X)?);
    let y = stick_direction(pads.axis(key, consts::AXIS_RAW, sdl::LEFT_Y)?);
    for (on, button) in [(x < 0, UiButton::Left), (x > 0, UiButton::Right), (y < 0, UiButton::Up), (y > 0, UiButton::Down)] {
        held[button.index()] |= on;
    }
    Ok(held)
}
