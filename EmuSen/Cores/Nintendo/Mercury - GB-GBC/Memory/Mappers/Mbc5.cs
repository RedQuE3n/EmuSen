using System;
using System.Collections.Generic;

namespace EmuSen.Cores.Nintendo.Mercury.Memory.Mappers
{
    // Nine ROM bank bits split across two registers, and the only board here that can select bank 0 - see Mercury_Memory.md §4.5.
    public sealed class Mbc5 : IMapper
    {
        private readonly Cartridge _cart;

        private bool _ramEnabled;
        private int _romBank = 1;
        private int _ramBank;
        private bool _rumbling;

        public Mbc5(Cartridge cart) => _cart = cart;

        public string Name => _cart.HasRumble ? "MBC5+Rumble" : "MBC5";

        private int RomMask => Math.Max(1, _cart.RomBanks) - 1;

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

                // No 0-becomes-1 remap here: bank 0 through the high window is legal on this board.
                case < 0x3000:
                    _romBank = (_romBank & 0x100) | data;
                    break;

                case < 0x4000:
                    _romBank = (_romBank & 0xFF) | ((data & 0x01) << 8);
                    break;

                case < 0x6000:
                    // On a rumble cart bit 3 drives the motor instead of addressing a fifth RAM bank.
                    if (_cart.HasRumble)
                    {
                        _rumbling = (data & 0x08) != 0;
                        _ramBank = data & 0x07;
                    }
                    else
                    {
                        _ramBank = data & 0x0F;
                    }
                    break;
            }
        }

        public byte ReadRam(ushort address)
        {
            if (!_ramEnabled || _cart.Ram.Length == 0) return 0xFF;

            int offset = _ramBank * Cartridge.RamBankSize + (address & 0x1FFF);
            return offset < _cart.Ram.Length ? _cart.Ram[offset] : (byte)0xFF;
        }

        public void WriteRam(ushort address, byte data)
        {
            if (!_ramEnabled || _cart.Ram.Length == 0) return;

            int offset = _ramBank * Cartridge.RamBankSize + (address & 0x1FFF);
            if (offset < _cart.Ram.Length) _cart.Ram[offset] = data;
        }

        public IReadOnlyList<(string Name, ulong Value, int Bits)> DebugState => new[]
        {
            ("RomBank", (ulong)_romBank, 9),
            ("RamBank", (ulong)_ramBank, 4),
            ("RamEnabled", _ramEnabled ? 1UL : 0UL, 1),
            ("Rumble", _rumbling ? 1UL : 0UL, 1),
        };
    }
}
