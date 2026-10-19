//! The player's controls into the latch, the C# `CoreEngine.SetButton` and `SetAxis`: a canonical control routed to
//! each bit and axis the port's controller gives it, from any thread and without the session's gate, the latch sent as
//! the next frame starts (Diamond_Players.md §12.7, EmuSen_Backend.md §29.4).

use std::sync::Mutex;

use emusen_core_host::InputLatch;
use emusen_native::json::Value;

use crate::info::{AXES, BUTTONS};

/// One controller's routes: each canonical button's bits, and each axis's control, number and kind.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pad {
    buttons: [u32; BUTTONS.len()],
    axes: Vec<(usize, u32, bool)>,
}

/// The routes of every port as the machine's info names their controllers.
#[derive(Debug, Default)]
pub struct Routes {
    machine: bool,
    /// Each port's controller, by machine info; none where it names one the system does not list.
    ports: Vec<(u32, Option<Pad>)>,
    /// The system's first controller.
    first: Option<Pad>,
}

fn named(names: &[&str], text: Option<&str>) -> Option<usize> {
    let text = text?;
    names.iter().position(|n| *n == text).or_else(|| text.trim().parse::<i64>().ok().filter(|n| *n >= 0 && (*n as usize) < names.len()).map(|n| n as usize))
}

fn pad_of(c: &Value) -> Pad {
    let mut pad = Pad::default();
    for b in c.get("buttons").and_then(Value::as_array).unwrap_or(&[]) {
        let bit = b.get("bit").and_then(Value::as_i64).unwrap_or(0);
        if let Some(control) = named(&BUTTONS, b.get("control").and_then(Value::as_str))
            && (0..=31).contains(&bit)
        {
            pad.buttons[control] |= 1u32 << bit;
        }
    }
    for a in c.get("axes").and_then(Value::as_array).unwrap_or(&[]) {
        if let Some(control) = named(&AXES, a.get("control").and_then(Value::as_str)) {
            let axis = a.get("axis").and_then(Value::as_i64).unwrap_or(0) as u32;
            pad.axes.push((control, axis, matches!(a.get("trigger"), Some(Value::Bool(true)))));
        }
    }
    pad
}

impl Routes {
    /// The routes for a machine of `system` whose info names `ports`, or for no machine yet.
    pub fn of(info: &Value, machine: Option<&crate::machine_info::MachineInfo>) -> Routes {
        let system = machine.map(|m| m.system.as_str());
        let ports: &[crate::machine_info::Port] = machine.map_or(&[], |m| &m.ports);
        let systems = info.get("systems").and_then(Value::as_array).unwrap_or(&[]);
        let chosen = systems.iter().find(|s| system.is_some() && s.get("id").and_then(Value::as_str) == system).or(systems.first());
        let controllers = chosen.and_then(|s| s.get("controllers")).and_then(Value::as_array).unwrap_or(&[]);
        let by_id = |id: Option<&str>| controllers.iter().find(|c| id.is_some() && c.get("id").and_then(Value::as_str) == id).map(pad_of);
        Routes {
            machine: machine.is_some(),
            ports: ports.iter().map(|p| (p.port, by_id(p.controller.as_deref()))).collect(),
            first: controllers.first().map(pad_of),
        }
    }

    /// `ControllerFor`: the port's controller by machine info, else the system's first where the info names no port.
    fn pad(&self, port: u32) -> Option<&Pad> {
        if !self.machine {
            return self.first.as_ref();
        }
        match self.ports.iter().find(|(p, _)| *p == port) {
            Some((_, pad)) => pad.as_ref(),
            None if self.ports.is_empty() => self.first.as_ref(),
            None => None,
        }
    }
}

/// A button held or let go on a port the latch has; a control the controller does not give is no change.
pub fn button(routes: &Mutex<Routes>, latch: &InputLatch, port: u32, control: usize, pressed: bool) {
    if port as usize >= latch.ports() {
        return;
    }
    let routes = routes.lock().unwrap_or_else(|p| p.into_inner());
    let Some(pad) = routes.pad(port) else { return };
    let bits = pad.buttons[control];
    if bits != 0 {
        latch.hold(port as usize, if pressed { bits } else { 0 }, bits);
    }
}

/// An axis moved, clamped to its kind's range; nothing before a machine is made.
pub fn axis(routes: &Mutex<Routes>, latch: &InputLatch, port: u32, control: usize, value: f64) {
    let routes = routes.lock().unwrap_or_else(|p| p.into_inner());
    if !routes.machine {
        return;
    }
    let Some(pad) = routes.pad(port) else { return };
    for &(c, axis, trigger) in &pad.axes {
        if c == control {
            latch.move_axis(port as usize, axis as usize, if trigger { value.clamp(0.0, 1.0) } else { value.clamp(-1.0, 1.0) });
        }
    }
}
