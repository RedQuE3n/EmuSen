using System;

namespace EmuSen.Cores.Nintendo.Mars.Memory
{
    // SRAM in the cartridge's second domain: 32KB repeating, or 32KB banks picked by address bits 18 and 19 - see Mars_Save.md §3.
    public sealed class Sram
    {
        public const int BankSize = 0x8000;

        public readonly byte[] Data;

        public bool Dirty;

        private readonly int _banks;

        public Sram(int banks, byte[]? saved)
        {
            _banks = banks;
            Data = new byte[banks * BankSize];
            Data.AsSpan().Fill(0xFF);
            if (saved != null) saved.AsSpan(0, Math.Min(saved.Length, Data.Length)).CopyTo(Data);
        }

        public byte Read8(uint offset) => Locate(offset) is int at ? Data[at] : (byte)0;

        public void Write8(uint offset, byte value)
        {
            if (Locate(offset) is not int at) return;

            Data[at] = value;
            Dirty = true;
        }

        // One bank repeats across the domain; banked SRAM has nothing past its last bank - see §3.
        private int? Locate(uint offset)
        {
            if (_banks == 1) return (int)(offset & (BankSize - 1));

            int bank = (int)(offset >> 18) & 3;
            return bank < _banks ? bank * BankSize + (int)(offset & (BankSize - 1)) : null;
        }
    }
}
