using System;
using System.Collections.Generic;

namespace EmuSen.Cores.Nintendo.Mercury.Memory.Mappers
{
    // Seven ROM bank bits, and the $A000 window can point at a clock register instead of RAM - see Mercury_Memory.md §4.4.
    public sealed class Mbc3 : IMapper
    {
        private readonly Cartridge _cart;

        private bool _ramEnabled;
        private int _romBank = 1;
        private int _ramBank;

        // The live clock, and the frozen copy $6000 latches for reading.
        private readonly byte[] _clock = new byte[5];
        private readonly byte[] _latched = new byte[5];
        private byte _lastLatchWrite = 0xFF;

        private long _cyclesIntoSecond;

        public Mbc3(Cartridge cart) => _cart = cart;

        public string Name => _cart.HasTimer ? "MBC3+RTC" : "MBC3";

        private int RomMask => Math.Max(1, _cart.RomBanks) - 1;

        private bool ClockSelected => _ramBank >= 0x08 && _ramBank <= 0x0C;

        public byte ReadRom(ushort address)
        {
            int bank = address < 0x4000 ? 0 : _romBank & RomMask;
            int offset = bank * Cartridge.RomBankSize + (address & 0x3FFF);
            return offset < _cart.Rom.Length ? _cart.Rom[offset] : (byte)0xFF;
        }

        public void WriteRom(ushort address, byte data)
        {
            switch (address)
            {
                case < 0x2000:
                    _ramEnabled = (data & 0x0F) == 0x0A;
                    break;

                case < 0x4000:
                    _romBank = (data & 0x7F) == 0 ? 1 : data & 0x7F;
                    break;

                case < 0x6000:
                    _ramBank = data & 0x0F;
                    break;

                // 0 then 1 copies the running clock into the registers a game actually reads.
                default:
                    if (_lastLatchWrite == 0x00 && data == 0x01) Array.Copy(_clock, _latched, _clock.Length);
                    _lastLatchWrite = data;
                    break;
            }
        }

        public byte ReadRam(ushort address)
        {
            if (!_ramEnabled) return 0xFF;
            if (ClockSelected) return _latched[_ramBank - 0x08];
            if (_cart.Ram.Length == 0) return 0xFF;

            int offset = _ramBank * Cartridge.RamBankSize + (address & 0x1FFF);
            return offset < _cart.Ram.Length ? _cart.Ram[offset] : (byte)0xFF;
        }

        public void WriteRam(ushort address, byte data)
        {
            if (!_ramEnabled) return;

            if (ClockSelected)
            {
                _clock[_ramBank - 0x08] = data;
                return;
            }

            if (_cart.Ram.Length == 0) return;

            int offset = _ramBank * Cartridge.RamBankSize + (address & 0x1FFF);
            if (offset < _cart.Ram.Length) _cart.Ram[offset] = data;
        }

        // Bit 6 of the day-high register halts the clock; everything else keeps counting.
        public void Tick(int cycles)
        {
            if (!_cart.HasTimer || (_clock[4] & 0x40) != 0) return;

            _cyclesIntoSecond += cycles;
            while (_cyclesIntoSecond >= MercuryCore.CpuClockHz)
            {
                _cyclesIntoSecond -= MercuryCore.CpuClockHz;
                AdvanceOneSecond();
            }
        }

        private void AdvanceOneSecond()
        {
            if (++_clock[0] < 60) return;
            _clock[0] = 0;

            if (++_clock[1] < 60) return;
            _clock[1] = 0;

            if (++_clock[2] < 24) return;
            _clock[2] = 0;

            if (++_clock[3] != 0) return;

            // Day 511 wraps to 0 and latches the overflow bit, which stays set until a game clears it.
            if ((_clock[4] & 0x01) != 0) _clock[4] = (byte)((_clock[4] & ~0x01) | 0x80);
            else _clock[4] |= 0x01;
        }

        public IReadOnlyList<(string Name, ulong Value, int Bits)> DebugState => new[]
        {
            ("RomBank", (ulong)_romBank, 7),
            ("RamBank", (ulong)_ramBank, 4),
            ("RamEnabled", _ramEnabled ? 1UL : 0UL, 1),
            ("RtcS", _clock[0], 8),
            ("RtcM", _clock[1], 8),
            ("RtcH", _clock[2], 8),
            ("RtcDL", _clock[3], 8),
            ("RtcDH", _clock[4], 8),
        };
    }
}
