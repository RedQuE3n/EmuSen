using System;

namespace EmuSen.Cores.Nintendo.Venus.Controllers
{
    // The 12 real SNES controller buttons. Deliberately excludes any concept of
    // "which key on my keyboard" - that mapping belongs in the front end (Program.cs),
    // not here.
    public enum SnesButton
    {
        B, Y, Select, Start, Up, Down, Left, Right, A, X, L, R
    }

    // SNES controller emulation at the register level: the $4218-$421B auto-joypad-read
    // latches (what essentially every game, including SMW, actually uses) and the
    // $4016/$4017 manual serial shift-register protocol (older-style polling, included
    // for completeness). No dependency on Raylib or any other input library - the front
    // end calls SetButton() based on whatever real input device it's reading, which
    // keeps this class portable and easy to test in isolation.
    public class Input
    {
        // Bit position within the 16-bit controller word for each button, matching
        // real hardware's shift order (MSB first): B Y Select Start Up Down Left
        // Right A X L R 0 0 0 0. This exact order is what $4218/$4219 (and the
        // $4016 serial sequence) actually shift out on real hardware.
        private static readonly int[] ButtonBit =
        {
            15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4
        };

        // Live button state as of the most recent SetButton calls this frame.
        private ushort _liveJoy1;
        private ushort _liveJoy2;

        // Latched state as of the last auto-joypad-read - this is what $4218-$421B
        // actually return. Real hardware re-latches automatically once per frame
        // during vblank; call LatchAutoJoypad() at that point.
        private ushort _latchedJoy1;
        private ushort _latchedJoy2;

        // $4016 manual serial-read state.
        private bool _strobe;
        private ushort _shiftJoy1;
        private ushort _shiftJoy2;

        // Called by the front end whenever a real input device's button state
        // changes. controller is 1 or 2.
        public void SetButton(SnesButton button, bool pressed, int controller = 1)
        {
            int bit = ButtonBit[(int)button];
            ushort mask = (ushort)(1 << bit);

            if (controller == 1)
            {
                _liveJoy1 = pressed ? (ushort)(_liveJoy1 | mask) : (ushort)(_liveJoy1 & ~mask);
            }
            else
            {
                _liveJoy2 = pressed ? (ushort)(_liveJoy2 | mask) : (ushort)(_liveJoy2 & ~mask);
            }
        }

        // Mirrors real hardware's automatic joypad read, which happens once per
        // frame at the start of vblank (gated on NMITIMEN bit 0 on real hardware;
        // treated as always-on here since essentially every game that reads
        // $4218 leaves it enabled for the whole time it's doing so).
        public void LatchAutoJoypad()
        {
            _latchedJoy1 = _liveJoy1;
            _latchedJoy2 = _liveJoy2;
        }

        // $4016 write: bit 0 is the strobe line. While held high, the shift
        // registers continuously re-latch the live state; the read sequence only
        // advances once strobe goes low.
        public void WriteStrobe(byte data)
        {
            _strobe = (data & 0x01) != 0;
            if (_strobe)
            {
                _shiftJoy1 = _latchedJoy1;
                _shiftJoy2 = _latchedJoy2;
            }
        }

        // $4016 read, bit 0 of the return value is the next Joy1 bit (MSB first).
        // Bits beyond the real 16 read back as 1, matching real hardware's
        // "no more buttons" convention.
        public byte ReadJoy1Serial()
        {
            if (_strobe)
            {
                return (byte)((_liveJoy1 >> 15) & 0x01);
            }
            int bit = (_shiftJoy1 >> 15) & 0x01;
            _shiftJoy1 = (ushort)((_shiftJoy1 << 1) | 1);
            return (byte)bit;
        }

        // $4017 read - same protocol as $4016, for controller 2.
        public byte ReadJoy2Serial()
        {
            if (_strobe)
            {
                return (byte)((_liveJoy2 >> 15) & 0x01);
            }
            int bit = (_shiftJoy2 >> 15) & 0x01;
            _shiftJoy2 = (ushort)((_shiftJoy2 << 1) | 1);
            return (byte)bit;
        }

        // $4218/$4219 - Joy1 auto-read low/high byte.
        public byte ReadJoy1Low() => (byte)(_latchedJoy1 & 0xFF);
        public byte ReadJoy1High() => (byte)(_latchedJoy1 >> 8);

        // $421A/$421B - Joy2 auto-read low/high byte.
        public byte ReadJoy2Low() => (byte)(_latchedJoy2 & 0xFF);
        public byte ReadJoy2High() => (byte)(_latchedJoy2 >> 8);
    }
}
