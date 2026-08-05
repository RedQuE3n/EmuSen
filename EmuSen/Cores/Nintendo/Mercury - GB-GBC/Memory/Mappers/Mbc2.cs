using System;
using System.Collections.Generic;

namespace EmuSen.Cores.Nintendo.Mercury.Memory.Mappers
{
    // 512 half-bytes of RAM on the chip itself, and address bit 8 picks which register a write means - see Mercury_Memory.md §4.3.
    public sealed class Mbc2 : IMapper
    {
        private const int RamSize = 512;

        private readonly Cartridge _cart;

        private bool _ramEnabled;
        private int _romBank = 1;

        public Mbc2(Cartridge cart) => _cart = cart;

        public string Name => "MBC2";

        private int RomMask => Math.Max(1, _cart.RomBanks) - 1;

        public byte ReadRom(ushort address)
        {
            int bank = address < 0x4000 ? 0 : _romBank & RomMask;
            int offset = bank * Cartridge.RomBankSize + (address & 0x3FFF);
            return offset < _cart.Rom.Length ? _cart.Rom[offset] : (byte)0xFF;
        }

        public void WriteRom(ushort address, byte data)
        {
            if (address >= 0x4000) return;

            // Bit 8 of the address, not the data, selects between the two registers this board has.
            if ((address & 0x0100) != 0)
            {
                _romBank = (data & 0x0F) == 0 ? 1 : data & 0x0F;
            }
            else
            {
                _ramEnabled = (data & 0x0F) == 0x0A;
            }
        }

        // The 512 nibbles echo through the whole $A000-$BFFF window.
        public byte ReadRam(ushort address)
        {
            if (!_ramEnabled || _cart.Ram.Length == 0) return 0xFF;

            // Only the low nibble is real; the high nibble reads back as open bus.
            return (byte)(_cart.Ram[(address - 0xA000) % RamSize] | 0xF0);
        }

        public void WriteRam(ushort address, byte data)
        {
            if (!_ramEnabled || _cart.Ram.Length == 0) return;
            _cart.Ram[(address - 0xA000) % RamSize] = (byte)(data & 0x0F);
        }

        public IReadOnlyList<(string Name, ulong Value, int Bits)> DebugState => new[]
        {
            ("RomBank", (ulong)_romBank, 4),
            ("RamEnabled", _ramEnabled ? 1UL : 0UL, 1),
        };
    }
}
