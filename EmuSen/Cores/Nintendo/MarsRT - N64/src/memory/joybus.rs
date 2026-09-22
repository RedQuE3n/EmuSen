//! The PIF walking the command block a game left in PIF RAM, channel by channel: the C# `Joybus`. See Mars_Serial.md §3.

use crate::memory::controller::{CHUNK_SIZE, Controller, ControllerPak};
use crate::memory::save::SaveChip;

pub const END: u8 = 0xFE;
pub const SKIP: u8 = 0xFF;
pub const PAD: u8 = 0xFD;
pub const SILENT: u8 = 0x00;
pub const NO_REPLY: u8 = 0x80;
pub const OVER_RUN: u8 = 0x40;

const INFO: u8 = 0x00;
const RESET: u8 = 0xFF;
const STATE: u8 = 0x01;
const PAK_READ: u8 = 0x02;
const PAK_WRITE: u8 = 0x03;

/// `Run`: to the end byte or the last byte of PIF RAM, whichever comes first.
pub fn run(ram: &mut [u8; 64], ports: &mut [Controller; 4], cartridge: &mut SaveChip) {
    let mut at = 0usize;
    let mut channel = 0usize;
    while at < ram.len() {
        let length = ram[at];
        if length == END {
            break;
        }
        if length == SILENT {
            channel += 1;
            at += 1;
            continue;
        }
        if length == PAD || length == SKIP {
            at += 1;
            continue;
        }
        at += 1;
        if at >= ram.len() || ram[at] == END {
            break;
        }
        let send = (length & 0x3F) as usize;
        let receive = (ram[at] & 0x3F) as usize;
        let length_at = at;
        at += 1;
        let command = at;
        if command + send > ram.len() || command + send + receive > ram.len() {
            break;
        }
        answer(ram, ports, cartridge, channel, command, send, receive, length_at);
        at = command + send + receive;
        channel += 1;
    }
}

#[allow(clippy::too_many_arguments)]
fn answer(ram: &mut [u8; 64], ports: &mut [Controller; 4], cartridge: &mut SaveChip, channel: usize, command: usize, send: usize, receive: usize, length_at: usize) {
    let id = if send > 0 { ram[command] } else { SKIP };
    let mut wrote = 0usize;
    let answered = if channel >= ports.len() {
        cartridge.answer_joybus(ram, command, send, receive, &mut wrote)
    } else {
        let port = &mut ports[channel];
        port.present && answered(ram, port, id, command, send, receive, &mut wrote)
    };
    if !answered {
        ram[length_at] = NO_REPLY | receive as u8;
        return;
    }
    ram[length_at] = (if wrote > receive { OVER_RUN } else { 0 }) | receive as u8;
}

fn answered(ram: &mut [u8; 64], port: &mut Controller, id: u8, command: usize, send: usize, receive: usize, wrote: &mut usize) -> bool {
    *wrote = 0;
    match id {
        INFO | RESET => {
            *wrote = 3;
            let pak = if port.pak.is_some() { 0x01 } else { 0x02 };
            reply(ram, command + send, receive, &[0x05, 0x00, pak]);
            true
        }
        STATE => {
            *wrote = 4;
            reply(ram, command + send, receive, &[(port.buttons >> 8) as u8, port.buttons as u8, port.stick_x as u8, port.stick_y as u8]);
            true
        }
        PAK_READ => {
            let Some(pak) = port.pak.as_ref() else { return false };
            if send < 3 {
                return false;
            }
            *wrote = CHUNK_SIZE + 1;
            let mut chunk = [0u8; CHUNK_SIZE + 1];
            pak.read(ControllerPak::address(ram[command + 1], ram[command + 2]), &mut chunk[..CHUNK_SIZE]);
            chunk[CHUNK_SIZE] = ControllerPak::data_crc(&chunk[..CHUNK_SIZE]);
            reply(ram, command + send, receive, &chunk);
            true
        }
        PAK_WRITE => {
            let Some(pak) = port.pak.as_mut() else { return false };
            if send < 3 + CHUNK_SIZE {
                return false;
            }
            let mut data = [0u8; CHUNK_SIZE];
            data.copy_from_slice(&ram[command + 3..command + 3 + CHUNK_SIZE]);
            pak.write(ControllerPak::address(ram[command + 1], ram[command + 2]), &data);
            *wrote = 1;
            reply(ram, command + send, receive, &[ControllerPak::data_crc(&data)]);
            true
        }
        _ => false,
    }
}

/// `Reply`: a reply longer than the room for it is cut; a shorter one leaves the rest alone.
pub fn reply(ram: &mut [u8; 64], at: usize, receive: usize, bytes: &[u8]) {
    for (i, &b) in bytes.iter().enumerate().take(receive) {
        ram[at + i] = b;
    }
}
