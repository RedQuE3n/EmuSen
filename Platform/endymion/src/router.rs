//! Every player's pad and the keyboard routed to the running game's controller ports: the C# `PortRouter`. See
//! EmuSen_Input.md §8.5 and §8.6.
//!
//! The pads are read by whoever owns the devices and handed in; what the router decides comes back as a list of
//! changes in the order the C# sends them, so that no call reaches back into the host.

use crate::pad::{self, AXES, BUTTONS, CONTROLS};
use crate::slots::MAX_PLAYERS;

/// One thing to tell the core.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Change {
    /// A port past the first holds a controller, or does not.
    Connected { port: i32, on: bool },
    /// A button went down or up on a port.
    Button { port: i32, button: u32, held: bool },
    /// An axis's value on a port, sent every time.
    Axis { port: i32, axis: u32, value: f64 },
}

/// What the router reads besides the pads: the keyboard's controls held, and which players have a pad seated, as bits from bit 0 for player 1.
#[derive(Clone, Copy, Debug, Default)]
pub struct Inputs {
    pub keyboard: u32,
    pub seated: u32,
}

#[derive(Clone, Debug)]
pub struct PortRouter {
    /// What each port was last told, so a button goes to the core only when it changes.
    sent: Vec<[bool; BUTTONS]>,
    axes: Vec<u32>,
    pad_held: [[bool; BUTTONS]; MAX_PLAYERS],
    pad_axes: [[f64; AXES]; MAX_PLAYERS],
    /// 1-based; the keyboard's controls count as this player's pad.
    pub keyboard_player: i32,
    pub mirror_player1_to_player2: bool,
}

impl Default for PortRouter {
    fn default() -> PortRouter {
        PortRouter { sent: Vec::new(), axes: Vec::new(), pad_held: [[false; BUTTONS]; MAX_PLAYERS], pad_axes: [[0.0; AXES]; MAX_PLAYERS], keyboard_player: 1, mirror_player1_to_player2: false }
    }
}

impl PortRouter {
    pub fn new() -> PortRouter {
        PortRouter::default()
    }

    /// How many controller ports the running game has.
    pub fn ports(&self) -> i32 {
        self.sent.len() as i32
    }

    pub fn axes(&self) -> &[u32] {
        &self.axes
    }

    /// The players a poll reads: as many as there are ports, and no more than there are seats.
    pub fn players_read(&self) -> usize {
        MAX_PLAYERS.min(self.sent.len())
    }

    /// A game just started: its ports and the axes it reads, with nothing held and nothing sent.
    pub fn reset(&mut self, ports: i32, axes: &[u32]) {
        self.sent = vec![[false; BUTTONS]; ports.max(0) as usize];
        self.axes = axes.to_vec();
        self.pad_held = [[false; BUTTONS]; MAX_PLAYERS];
        self.pad_axes = [[0.0; AXES]; MAX_PLAYERS];
    }

    /// The game's ports changed in number: a port taken away lets go of its buttons, and the rest keep what they were sent.
    pub fn resize(&mut self, ports: i32) -> Vec<Change> {
        let ports = ports.max(0) as usize;
        let mut changes = Vec::new();
        for port in ports..self.sent.len() {
            for button in 0..BUTTONS {
                if self.sent[port][button] {
                    changes.push(Change::Button { port: port as i32, button: button as u32, held: false });
                }
            }
        }
        self.sent.resize(ports, [false; BUTTONS]);
        changes
    }

    /// The pads as read for each player a port hears, from player 1: the buttons as bits by `PadButton`, and the read axes' values in the order of `axes`. Then sends what changed.
    pub fn poll(&mut self, held: &[u16], axes: &[f64], inputs: Inputs) -> Vec<Change> {
        let players = self.players_read();
        let count = self.axes.len();
        for player in 0..players {
            let bits = held.get(player).copied().unwrap_or(0);
            for button in 0..BUTTONS {
                self.pad_held[player][button] = bits & (1 << button) != 0;
            }
            for (k, &axis) in self.axes.iter().enumerate() {
                if (axis as usize) < AXES {
                    self.pad_axes[player][axis as usize] = axes.get(player * count + k).copied().unwrap_or(0.0);
                }
            }
        }
        self.send(inputs)
    }

    /// A player's pad as the last poll read it.
    pub fn pad_held(&self, player: i32, button: u32) -> bool {
        (1..=MAX_PLAYERS as i32).contains(&player) && (button as usize) < BUTTONS && self.pad_held[player as usize - 1][button as usize]
    }

    /// A port past the first holds a controller while its player has a pad seated, connected or reserved, or the keyboard.
    pub fn connected(&self, port: i32, inputs: Inputs) -> bool {
        port == 0 || self.has_input(port + 1, inputs) || (self.mirrored(port) > 0 && self.has_input(self.mirrored(port), inputs))
    }

    fn has_input(&self, player: i32, inputs: Inputs) -> bool {
        player == self.keyboard_player || ((1..=32).contains(&player) && inputs.seated & (1 << (player - 1)) != 0)
    }

    /// Every port's flags and buttons as they now are: after a poll, or after a key went down or up with the pads as last read.
    pub fn send(&mut self, inputs: Inputs) -> Vec<Change> {
        let ports = self.sent.len();
        let mut changes = Vec::with_capacity(ports.saturating_sub(1) + ports * (BUTTONS + self.axes.len()));
        for port in 1..ports {
            changes.push(Change::Connected { port: port as i32, on: self.connected(port as i32, inputs) });
        }
        for port in 0..ports {
            for button in 0..BUTTONS {
                let held = self.held(port as i32, button as u32, inputs);
                if held == self.sent[port][button] {
                    continue;
                }
                self.sent[port][button] = held;
                changes.push(Change::Button { port: port as i32, button: button as u32, held });
            }
            for &axis in &self.axes {
                let value = pad::resolve(axis, self.raw_axis(port, axis), |control| self.held(port as i32, control, inputs));
                changes.push(Change::Axis { port: port as i32, axis, value });
            }
        }
        changes
    }

    /// A port hears its own player, and the second port player 1 as well while mirroring.
    fn mirrored(&self, port: i32) -> i32 {
        if port == 1 && self.mirror_player1_to_player2 { 1 } else { 0 }
    }

    fn held(&self, port: i32, control: u32, inputs: Inputs) -> bool {
        self.held_by(port + 1, control, inputs) || (self.mirrored(port) > 0 && self.held_by(self.mirrored(port), control, inputs))
    }

    fn held_by(&self, player: i32, control: u32, inputs: Inputs) -> bool {
        if player == self.keyboard_player && (control as usize) < CONTROLS && inputs.keyboard & (1 << control) != 0 {
            return true;
        }
        player <= MAX_PLAYERS as i32 && pad::as_button(control).is_some_and(|button| self.pad_held[player as usize - 1][button as usize])
    }

    /// Of the pads a port hears, the stick pushed furthest.
    fn raw_axis(&self, port: usize, axis: u32) -> f64 {
        let axis = axis as usize;
        if axis >= AXES {
            return 0.0;
        }
        let mut value = if port < MAX_PLAYERS { self.pad_axes[port][axis] } else { 0.0 };
        let also = self.mirrored(port as i32);
        if also > 0 && self.pad_axes[also as usize - 1][axis].abs() > value.abs() {
            value = self.pad_axes[also as usize - 1][axis];
        }
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buttons(changes: &[Change]) -> Vec<(i32, u32, bool)> {
        changes.iter().filter_map(|c| if let Change::Button { port, button, held } = *c { Some((port, button, held)) } else { None }).collect()
    }

    #[test]
    fn each_players_pad_reaches_its_own_port_and_a_button_is_sent_when_it_changes() {
        let mut router = PortRouter::new();
        router.reset(4, &[]);
        let inputs = Inputs { keyboard: 0, seated: 0b1111 };
        let changes = router.poll(&[0, 1, 0, 1 << 3], &[], inputs);
        assert_eq!(buttons(&changes), [(1, 0, true), (3, 3, true)]);
        assert_eq!(changes[..3], [Change::Connected { port: 1, on: true }, Change::Connected { port: 2, on: true }, Change::Connected { port: 3, on: true }]);
        assert!(buttons(&router.poll(&[0, 1, 0, 1 << 3], &[], inputs)).is_empty());
        assert_eq!(buttons(&router.poll(&[0, 0, 0, 1 << 3], &[], inputs)), [(1, 0, false)]);
        assert!(router.pad_held(4, 3) && !router.pad_held(9, 3));
    }

    #[test]
    fn the_keyboard_plays_as_its_player_and_the_mirror_gives_port_two_player_one() {
        let mut router = PortRouter::new();
        router.reset(2, &[pad::LEFT_X]);
        router.keyboard_player = 2;
        let changes = router.poll(&[0, 0], &[0.25, -0.75], Inputs { keyboard: 1 << 8 | 1 << pad::LEFT_STICK_RIGHT, seated: 1 });
        assert_eq!(buttons(&changes), [(1, 8, true)]);
        assert!(changes.contains(&Change::Axis { port: 1, axis: pad::LEFT_X, value: 1.0 }));
        assert!(changes.contains(&Change::Axis { port: 0, axis: pad::LEFT_X, value: 0.25 }));
        router.keyboard_player = 1;
        router.mirror_player1_to_player2 = true;
        let changes = router.send(Inputs { keyboard: 0, seated: 1 });
        assert!(changes.contains(&Change::Axis { port: 1, axis: pad::LEFT_X, value: -0.75 }));
        assert!(changes.contains(&Change::Connected { port: 1, on: true }));
    }

    #[test]
    fn a_port_taken_away_lets_go_of_its_buttons() {
        let mut router = PortRouter::new();
        router.reset(3, &[]);
        router.poll(&[1, 1, 1 | 1 << 15], &[], Inputs { keyboard: 0, seated: 0b111 });
        assert_eq!(buttons(&router.resize(1)), [(1, 0, false), (2, 0, false), (2, 15, false)]);
        assert_eq!(router.ports(), 1);
        assert!(router.resize(3).is_empty());
        assert_eq!(router.ports(), 3);
        // Port 0 still holds what it was sent, and a count below zero is none.
        assert_eq!(buttons(&router.resize(-2)), [(0, 0, false)]);
        assert_eq!(router.ports(), 0);
    }
}
