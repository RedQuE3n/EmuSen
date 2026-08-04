using System;
using System.Collections.Generic;

namespace EmuSen.Cores.Nintendo.Mercury.Memory.Mappers
{
    // Two bank registers and a mode bit that decides which of them the low ROM window follows - see Mercury_Memory.md §4.2.
    public sealed class Mbc1 : IMapper
    {
        private readonly Cartridge _cart;

        private bool _ramEnabled;
        private int _bank1 = 1;   // 5 bits, never 0
        private int _bank2;       // 2 bits, ROM high bits or RAM bank
        private bool _advancedMode;

        public Mbc1(Cartridge cart) => _cart = cart;

        public string Name => "MBC1";

        // Only mode 1 lets $0000-$3FFF leave bank 0, which is how the 1MB+ carts reach their upper half.
        private int LowBank => _advancedMode ? (_bank2 << 5) & RomMask : 0;

        private int HighBank => ((_bank2 << 5) | _bank1) & RomMask;

        private int RomMask => Math.Max(1, _cart.RomBanks) - 1;

        private int RamBank => _advancedMode ? _bank2 : 0;

        public byte ReadRom(ushort address)
        {
            int bank = address < 0x4000 ? LowBank : HighBank;
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

                // A written 0 becomes 1: this board cannot address bank 0 through the high window.
                case < 0x4000:
                    _bank1 = (data & 0x1F) == 0 ? 1 : data & 0x1F;
                    break;

                case < 0x6000:
                    _bank2 = data & 0x03;
                    break;

                default:
                    _advancedMode = (data & 0x01) != 0;
                    break;
            }
        }

        public byte ReadRam(ushort address)
        {
            if (!_ramEnabled || _cart.Ram.Length == 0) return 0xFF;

            int offset = RamBank * Cartridge.RamBankSize + (address & 0x1FFF);
            return offset < _cart.Ram.Length ? _cart.Ram[offset] : (byte)0xFF;
        }

        public void WriteRam(ushort address, byte data)
        {
            if (!_ramEnabled || _cart.Ram.Length == 0) return;

            int offset = RamBank * Cartridge.RamBankSize + (address & 0x1FFF);
            if (offset < _cart.Ram.Length) _cart.Ram[offset] = data;
        }

        public IReadOnlyList<(string Name, ulong Value, int Bits)> DebugState => new[]
        {
            ("Bank1", (ulong)_bank1, 5),
            ("Bank2", (ulong)_bank2, 2),
            ("Mode", _advancedMode ? 1UL : 0UL, 1),
            ("RomBank", (ulong)HighBank, 7),
            ("RamEnabled", _ramEnabled ? 1UL : 0UL, 1),
        };
    }
}
