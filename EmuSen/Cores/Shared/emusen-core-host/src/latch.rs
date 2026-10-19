//! The buttons, axes and presence the player holds on each port, taken on any thread and handed to the machine on its
//! own before each frame; the C# host's `InputLatch` (Diamond_Players.md §12.7, EmuSen_CoreAPI.md §6.36).

use std::sync::Mutex;

use crate::machine::{Machine, Status};

/// Where the latch sends what it holds: a machine, or a stand-in in a test.
pub trait Inputs {
    fn set_buttons(&mut self, port: u32, mask: u32, changed: u32) -> Result<(), Status>;
    fn set_axis(&mut self, port: u32, axis: u32, value: f64) -> Result<(), Status>;
    fn set_present(&mut self, port: u32, present: bool) -> Result<(), Status>;
}

impl Inputs for Machine {
    fn set_buttons(&mut self, port: u32, mask: u32, changed: u32) -> Result<(), Status> {
        Machine::set_buttons(self, port, mask, changed)
    }

    fn set_axis(&mut self, port: u32, axis: u32, value: f64) -> Result<(), Status> {
        Machine::set_axis(self, port, axis, value)
    }

    fn set_present(&mut self, port: u32, present: bool) -> Result<(), Status> {
        Machine::set_present(self, port, present)
    }
}

#[derive(Default, Clone)]
struct Held {
    buttons: Vec<u32>,
    changed: Vec<u32>,
    axes: Vec<f64>,
    moved: Vec<bool>,
    present: Vec<bool>,
    presence_changed: Vec<bool>,
    /// Every port's presence sent with the next frame, whether or not it changed.
    resend: bool,
    axes_per_port: usize,
}

impl Held {
    fn sized(ports: usize, axes_per_port: usize) -> Held {
        Held {
            buttons: vec![0; ports],
            changed: vec![0; ports],
            axes: vec![0.0; ports * axes_per_port],
            moved: vec![false; ports * axes_per_port],
            present: vec![true; ports],
            presence_changed: vec![false; ports],
            resend: false,
            axes_per_port,
        }
    }

    fn ports(&self) -> usize {
        self.buttons.len()
    }

    /// `self` made the shape of `from` and given its values, allocating only when the shape grew.
    fn copy_from(&mut self, from: &Held) {
        self.buttons.clone_from(&from.buttons);
        self.changed.clone_from(&from.changed);
        self.axes.clone_from(&from.axes);
        self.moved.clone_from(&from.moved);
        self.present.clone_from(&from.present);
        self.presence_changed.clone_from(&from.presence_changed);
        self.resend = from.resend;
        self.axes_per_port = from.axes_per_port;
    }
}

/// Two ports of nothing held until a machine's shape is known.
pub struct InputLatch {
    held: Mutex<Held>,
    /// The frame thread's own copy, so that a send allocates nothing and calls the machine outside the gate.
    sending: Mutex<Held>,
}

impl Default for InputLatch {
    fn default() -> InputLatch {
        InputLatch::new(2, 0)
    }
}

impl InputLatch {
    pub fn new(ports: usize, axes_per_port: usize) -> InputLatch {
        InputLatch { held: Mutex::new(Held::sized(ports, axes_per_port)), sending: Mutex::new(Held::default()) }
    }

    fn gate(&self) -> std::sync::MutexGuard<'_, Held> {
        self.held.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Nothing held, every port full, for a new machine of `ports` ports whose axes are numbered below `axes_per_port`.
    pub fn reset(&self, ports: usize, axes_per_port: usize) {
        *self.gate() = Held::sized(ports, axes_per_port);
    }

    /// A port added while the game runs is held from now.
    pub fn grow(&self, ports: usize) {
        let mut h = self.gate();
        if ports <= h.ports() {
            return;
        }
        let per = h.axes_per_port;
        h.buttons.resize(ports, 0);
        h.changed.resize(ports, 0);
        h.present.resize(ports, true);
        h.presence_changed.resize(ports, false);
        h.axes.resize(ports * per, 0.0);
        h.moved.resize(ports * per, false);
    }

    pub fn ports(&self) -> usize {
        self.gate().ports()
    }

    pub fn buttons(&self, port: usize) -> u32 {
        self.gate().buttons.get(port).copied().unwrap_or(0)
    }

    /// The bits of `changed` take `mask`'s values, as `set_buttons` takes them; a port the machine lacks is ignored.
    pub fn hold(&self, port: usize, mask: u32, changed: u32) {
        let mut h = self.gate();
        if port < h.ports() {
            h.buttons[port] = (h.buttons[port] & !changed) | (mask & changed);
            h.changed[port] |= changed;
        }
    }

    /// A controller plugged in or pulled out; only a change is sent.
    pub fn connect(&self, port: usize, connected: bool) {
        let mut h = self.gate();
        if port < h.ports() && h.present[port] != connected {
            h.present[port] = connected;
            h.presence_changed[port] = !h.presence_changed[port];
        }
    }

    /// Every port's presence sent with the next frame, as after a state load, whether or not it changed.
    pub fn resend_presence(&self) {
        self.gate().resend = true;
    }

    /// An axis's latest value replaces any not yet sent.
    pub fn move_axis(&self, port: usize, axis: usize, value: f64) {
        let mut h = self.gate();
        if port < h.ports() && axis < h.axes_per_port {
            let at = port * h.axes_per_port + axis;
            h.axes[at] = value;
            h.moved[at] = true;
        }
    }

    /// On the machine's own thread: every port's presence change, changed bits and moved axes since the last send.
    pub fn send(&self, to: &mut impl Inputs) {
        let mut s = self.sending.lock().unwrap_or_else(|e| e.into_inner());
        {
            let mut h = self.gate();
            s.copy_from(&h);
            h.changed.fill(0);
            h.moved.fill(false);
            h.presence_changed.fill(false);
            h.resend = false;
        }
        let per = s.axes_per_port;
        for port in 0..s.ports() {
            if s.resend || s.presence_changed[port] {
                let _ = to.set_present(port as u32, s.present[port]);
            }
            if s.changed[port] != 0 {
                let _ = to.set_buttons(port as u32, s.buttons[port], s.changed[port]);
            }
            for axis in 0..per {
                if s.moved[port * per + axis] {
                    let _ = to.set_axis(port as u32, axis as u32, s.axes[port * per + axis]);
                }
            }
        }
    }
}
