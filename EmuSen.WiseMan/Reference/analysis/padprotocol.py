#!/usr/bin/env python3
"""A test program for a four-player adapter's lines, run on a reference and on Nephrite (Nephrite_Native.md §45).

The program is written here, not a game: it writes a list of values to the two ports' data and control registers and
stores what it reads back after each, a byte a step from $FF0000, with $A5 at $FF0FFE when it is done. The reference
and Nephrite run it with the same adapter and the same buttons held, and the bytes are compared step by step.

  padprotocol.py --probe PROBE --core REFERENCE.so --dump NEPHRITE_DUMP [--keep DIR]

The reference is never read, only run.
"""
import argparse, os, struct, subprocess, sys, tempfile

A, B, CTRL_A, CTRL_B = 0x03, 0x05, 0x09, 0x0B


def program(steps):
    """A cartridge that makes each (register, value) write and stores a read of `read` after it."""
    code = [0x46FC, 0x2700, 0x203C, 0x0004, 0x0000, 0x5380, 0x66FC]      # SR, then a delay while the pads settle
    for n, (reg, value, read) in enumerate(steps):
        code += [0x13FC, value, 0x00A1, reg, 0x4E71, 0x4E71, 0x4E71, 0x4E71, 0x13F9, 0x00A1, read, 0x00FF, n]
    code += [0x13FC, 0x00A5, 0x00FF, 0x0FFE, 0x60FE]
    rom = bytearray(b"\xFF" * 0x8000)
    rom[0:8] = struct.pack(">II", 0x00FFFE00, 0x200)
    rom[0x100:0x110] = b"SEGA GENESIS    "
    rom[0x120:0x150] = rom[0x150:0x180] = b"PAD PROTOCOL".ljust(48)
    rom[0x180:0x18E] = b"GM 00000000-00"
    rom[0x190:0x1A0] = b"J".ljust(16)
    rom[0x1A0:0x1B0] = struct.pack(">IIII", 0, 0x7FFF, 0xFF0000, 0xFFFFFF)
    rom[0x1F0:0x1F3] = b"JUE"
    body = b"".join(struct.pack(">H", w) for w in code)
    rom[0x200:0x200 + len(body)] = body
    return bytes(rom)


def team_player():
    """Rest, the two checks, 24 nibbles asked (past any packet's end), restarts with TR low, then TR as an input."""
    steps = [(CTRL_A, 0x60, A), (A, 0x60, A), (A, 0x40, A), (A, 0x60, A), (A, 0x20, A)]
    tr = 0x20
    for _ in range(24):
        tr ^= 0x20
        steps.append((A, tr, A))
    steps += [(A, v, A) for v in (0x60, 0x40, 0x00, 0x20, 0x00, 0x20, 0x40, 0x60, 0x00, 0x40, 0x00, 0x20, 0x60)]
    return steps + [(CTRL_A, 0x40, A), (A, 0x40, A), (A, 0x00, A), (A, 0x40, A)]


def four_way():
    """Each of port B's eight choices with TH high, low and high, port B read back, its lines as inputs, and TH alone driven."""
    steps = [(CTRL_A, 0x40, A), (CTRL_B, 0x7F, A), (A, 0x40, A)]
    for choice in (0x0C, 0x1C, 0x2C, 0x3C, 0x4C, 0x5C, 0x6C, 0x7C):
        steps += [(B, choice, A), (A, 0x40, A), (A, 0x00, A), (A, 0x40, A)]
    steps += [(B, 0x7C, B), (B, 0x0C, B), (B, 0x00, A), (B, 0x70, A), (B, 0x7F, A), (CTRL_B, 0x00, A), (CTRL_B, 0x7F, A), (B, 0x0C, A)]
    # Port B as a game not made for the adapter leaves it: TH alone an output, the choosing lines inputs.
    return steps + [(CTRL_B, 0x40, A), (B, 0x00, A), (B, 0x10, A), (B, 0x20, A), (B, 0x30, A), (B, 0x40, A)]


# Each pad's buttons: the probe's names for the reference, the ABI's bits for Nephrite (Up 0 ... Start 7).
HELD = [(["Up"], 0x01), (["B"], 0x10), (["Left", "Y"], 0x44), (["Start"], 0x80), (["A"], 0x20)]
RUNS = [("Team Player, 3-button pads", team_player, ["MD Joypad 3 Button + Teamplayer", "MD Joypad 3 Button"], "md.teamplayer3,md.pad3"),
        ("Team Player, 6-button pads", team_player, ["MD Joypad 6 Button + Teamplayer", "MD Joypad 6 Button"], "md.teamplayer6,md.pad6"),
        ("4 Way Play, 3-button pads", four_way, ["MD Joypad 3 Button + 4-WayPlay"] * 2, "md.4way3,md.pad3")]


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--probe", required=True); p.add_argument("--core", required=True); p.add_argument("--dump", required=True)
    p.add_argument("--keep")
    a = p.parse_args()
    for name, steps, devices, plugs in RUNS:
        steps = steps()
        with tempfile.TemporaryDirectory() as tmp:
            tmp = a.keep or tmp
            rom = os.path.join(tmp, "padprotocol.bin")
            open(rom, "wb").write(program(steps))
            cmd = [a.probe, rom, os.path.join(tmp, "r"), "200", "200", "--backend", "libretro", "--core", a.core, "--system", "megadrive"]
            for port, device in enumerate(devices):
                cmd += ["--device", f"{port + 1}={device}"]
            for pad, (buttons, _) in enumerate(HELD):
                for b in buttons:
                    cmd += ["--presson", f"{pad + 1}:1:{b}:400"]
            subprocess.run(cmd, check=True, capture_output=True)
            env = dict(os.environ, NEPHRITE_PLUGS=plugs, NEPHRITE_PRESS=",".join(f"1:{bits}:400:{pad + 1}" for pad, (_, bits) in enumerate(HELD)))
            subprocess.run([a.dump, rom, os.path.join(tmp, "n"), "200"], check=True, capture_output=True, env=env)
            words = open(os.path.join(tmp, "r", [f for f in os.listdir(os.path.join(tmp, "r")) if "_ram_f" in f][0]), "rb").read()
            ref = bytes(words[i ^ 1] for i in range(len(words)))
            own = open(os.path.join(tmp, "n", "nephrite_wram_f00200.bin"), "rb").read()
            n = len(steps)
            assert ref[0xFFE] == own[0xFFE] == 0xA5, "the program did not finish"
            print(name)
            print("  reference:", " ".join(f"{x:02X}" for x in ref[:n]))
            print("  Nephrite: ", " ".join(f"{x:02X}" for x in own[:n]))
            print("  steps that differ:", [i for i in range(n) if ref[i] != own[i]])
    return 0


if __name__ == "__main__":
    sys.exit(main())
