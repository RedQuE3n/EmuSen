//! The canonical controls and axes, latched before each frame and answered to `input_state` (EmuSen_LibretroHost.md §4.4).

use emusen_libretro_sys as sys;

/// The ports a machine offers; libretro's pads beyond them read nothing.
pub const PORTS: usize = 8;

/// A stick's -1..1 as libretro's -32768..32767, scaled by 32,767 and rounded half away from zero.
pub fn stick(value: f64) -> i16 {
    (value.clamp(-1.0, 1.0) * 32767.0).round() as i16
}

/// A trigger's 0..1 as an analog button's 0..32767.
pub fn trigger(value: f64) -> i16 {
    (value.clamp(0.0, 1.0) * 32767.0).round() as i16
}

#[derive(Clone, Copy, Default)]
pub struct Pad {
    pub buttons: u16,
    /// Left X, left Y, right X, right Y, left trigger, right trigger, in libretro's units.
    pub axes: [i16; 6],
    /// Whether each trigger has been set as an axis, so that its digital bit no longer answers alone.
    pub analog_triggers: [bool; 2],
    pub present: bool,
}

pub struct Input {
    pub pads: [Pad; PORTS],
}

impl Default for Input {
    fn default() -> Input {
        Input { pads: [Pad { present: true, ..Pad::default() }; PORTS] }
    }
}

impl Input {
    pub fn set_buttons(&mut self, port: usize, mask: u32, changed: u32) {
        let pad = &mut self.pads[port];
        pad.buttons = (pad.buttons & !(changed as u16)) | (mask as u16 & changed as u16);
    }

    /// An axis by `enum emusen_axis`; false for one it does not know.
    pub fn set_axis(&mut self, port: usize, axis: u32, value: f64) -> bool {
        let pad = &mut self.pads[port];
        match axis {
            0..=3 => pad.axes[axis as usize] = stick(value),
            4 | 5 => {
                pad.axes[axis as usize] = trigger(value);
                pad.analog_triggers[axis as usize - 4] = true;
            }
            _ => return false,
        }
        true
    }

    /// libretro's `input_state` for one pad.
    pub fn state(&self, port: u32, device: u32, index: u32, id: u32) -> i16 {
        let Some(pad) = self.pads.get(port as usize) else { return 0 };
        if !pad.present {
            return 0;
        }
        match device & sys::DEVICE_MASK {
            sys::DEVICE_JOYPAD if id == sys::DEVICE_ID_JOYPAD_MASK => pad.buttons as i16,
            sys::DEVICE_JOYPAD if id < 16 => ((pad.buttons >> id) & 1) as i16,
            sys::DEVICE_ANALOG => match (index, id) {
                (sys::DEVICE_INDEX_ANALOG_LEFT, 0 | 1) => pad.axes[id as usize],
                (sys::DEVICE_INDEX_ANALOG_RIGHT, 0 | 1) => pad.axes[2 + id as usize],
                (sys::DEVICE_INDEX_ANALOG_BUTTON, sys::DEVICE_ID_JOYPAD_L2) if pad.analog_triggers[0] => pad.axes[4],
                (sys::DEVICE_INDEX_ANALOG_BUTTON, sys::DEVICE_ID_JOYPAD_R2) if pad.analog_triggers[1] => pad.axes[5],
                (sys::DEVICE_INDEX_ANALOG_BUTTON, b) if b < 16 => ((pad.buttons >> b) & 1) as i16 * 0x7FFF,
                _ => 0,
            },
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sticks_and_triggers_scale_to_libretros_units() {
        assert_eq!((stick(1.0), stick(-1.0), stick(0.5), stick(-0.5), stick(2.0)), (32767, -32767, 16384, -16384, 32767));
        assert_eq!((trigger(1.0), trigger(0.0), trigger(-1.0)), (32767, 0, 0));
    }

    #[test]
    fn the_retropad_ids_are_the_canonical_bits() {
        let mut i = Input::default();
        i.set_buttons(0, 1 << 8 | 1, 0xFFFF);
        assert_eq!(i.state(0, sys::DEVICE_JOYPAD, 0, 8), 1);
        assert_eq!(i.state(0, sys::DEVICE_JOYPAD, 0, 1), 0);
        assert_eq!(i.state(0, sys::DEVICE_JOYPAD, 0, sys::DEVICE_ID_JOYPAD_MASK), 0x101);
        i.set_buttons(0, 0, 1);
        assert_eq!(i.state(0, sys::DEVICE_JOYPAD, 0, sys::DEVICE_ID_JOYPAD_MASK), 0x100);
    }

    #[test]
    fn analog_buttons_read_their_bit_until_the_trigger_is_an_axis() {
        let mut i = Input::default();
        i.set_buttons(1, 1 << 12, 0xFFFF);
        assert_eq!(i.state(1, sys::DEVICE_ANALOG, sys::DEVICE_INDEX_ANALOG_BUTTON, 12), 0x7FFF);
        i.set_axis(1, 4, 0.25);
        assert_eq!(i.state(1, sys::DEVICE_ANALOG, sys::DEVICE_INDEX_ANALOG_BUTTON, 12), 8192);
        assert!(!i.set_axis(1, 6, 0.0));
    }

    #[test]
    fn an_absent_pad_reads_nothing() {
        let mut i = Input::default();
        i.set_buttons(0, 0xFFFF, 0xFFFF);
        i.pads[0].present = false;
        assert_eq!(i.state(0, sys::DEVICE_JOYPAD, 0, sys::DEVICE_ID_JOYPAD_MASK), 0);
    }
}
