using EmuSen.Common;
using EmuSen.Cores.Nintendo.Venus.Memory.Mappers;

namespace EmuSen.Cores.Nintendo.Venus.Coprocessors.Obc1
{
    // Seta OBC1: not a processor at all, just an address generator sitting in
    // front of the cartridge's SRAM so a game can write one sprite's worth of
    // OAM through a fixed four-byte port. Only Metal Combat uses it.
    // See Venus_OBC1.md.
    public sealed class Obc1
    {
        // The chip holds no state of its own - everything it reads and writes,
        // including its own control bytes, lives in the cartridge's SRAM.
        [SkipInState] private readonly byte[] _sram;
        [SkipInState] private readonly int _mask;

        public Obc1(byte[] sram)
        {
            _sram = sram;
            _mask = sram.Length - 1;
        }

        public CartridgeAddress ResolveScpu(byte bank, ushort offset)
        {
            bool obc1Bank = bank <= 0x3F || (bank >= 0x80 && bank <= 0xBF);
            if (obc1Bank && offset >= 0x6000 && offset <= 0x7FFF) return CartridgeAddress.CoprocessorRegister(offset);
            return CartridgeAddress.Unmapped;
        }

        public byte ReadRegister(int address)
        {
            int offset = address & 0x1FFF;
            return offset switch
            {
                0x1FF0 or 0x1FF1 or 0x1FF2 or 0x1FF3 => ReadRam(LowAddress + (offset - 0x1FF0)),
                0x1FF4 => ReadRam(HighAddress),
                _ => ReadRam(offset),
            };
        }

        public void WriteRegister(int address, byte value)
        {
            int offset = address & 0x1FFF;
            switch (offset)
            {
                case 0x1FF0:
                case 0x1FF1:
                case 0x1FF2:
                case 0x1FF3:
                    WriteRam(LowAddress + (offset - 0x1FF0), value);
                    return;

                // The high table packs four sprites per byte, so a write has
                // to merge into the right 2-bit field - see Venus_OBC1.md §2.
                case 0x1FF4:
                {
                    int shift = (ReadRam(0x1FF6) & 0x03) << 1;
                    byte merged = (byte)((ReadRam(HighAddress) & ~(0x03 << shift)) | ((value & 0x03) << shift));
                    WriteRam(HighAddress, merged);
                    return;
                }

                default:
                    WriteRam(offset, value);
                    return;
            }
        }

        // $1FF5 bit 0 picks the OAM mirror's base, $1FF6 the sprite index - see Venus_OBC1.md §2.
        private int BaseAddress => 0x1800 | ((~ReadRam(0x1FF5) & 0x01) << 10);

        private int LowAddress => BaseAddress | ((ReadRam(0x1FF6) & 0x7F) << 2);

        private int HighAddress => (BaseAddress | ((ReadRam(0x1FF6) & 0x7F) >> 2)) + 0x200;

        private byte ReadRam(int address) => _sram[address & _mask];

        private void WriteRam(int address, byte value) => _sram[address & _mask] = value;
    }
}
