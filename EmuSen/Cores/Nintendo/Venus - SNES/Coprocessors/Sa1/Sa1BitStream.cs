using EmuSen.Common;

namespace EmuSen.Cores.Nintendo.Venus.Coprocessors.Sa1
{
    // Variable-length bit stream reader ($2258-$225B in, $230C/D out). Pulls
    // 1-16 bit fields out of ROM at arbitrary bit offsets, which is what the
    // SA-1 decompressors are built on - see Venus_SA1.md §7.
    public sealed class Sa1BitStream
    {
        [SkipInState] private readonly byte[] _rom;

        private byte _control;
        private uint _address;
        private int _bitOffset;

        public Sa1BitStream(byte[] rom)
        {
            _rom = rom;
        }

        private bool AutoIncrement => (_control & 0x80) != 0;

        // Bits per field; 0 in the register means a full 16.
        private int Length => (_control & 0x0F) == 0 ? 16 : (_control & 0x0F);

        // The 16 bits currently under the cursor, LSB-first within the byte stream.
        public ushort Data
        {
            get
            {
                uint window = (uint)(RomByte(_address) | (RomByte(_address + 1) << 8) | (RomByte(_address + 2) << 16));
                return (ushort)(window >> _bitOffset);
            }
        }

        private byte RomByte(uint offset) => offset < _rom.Length ? _rom[offset] : (byte)0x00;

        public void WriteControl(byte value)
        {
            _control = value;
            // In fixed mode the write itself is the advance; auto mode advances on read instead.
            if (!AutoIncrement) Advance();
        }

        public void WriteAddressByte(int index, byte value)
        {
            int shift = index * 8;
            _address = (_address & ~(0xFFu << shift)) | ((uint)value << shift);

            // Loading the high byte restarts the stream at bit 0 of that address.
            if (index == 2) _bitOffset = 0;
        }

        public void OnDataHighRead()
        {
            if (AutoIncrement) Advance();
        }

        private void Advance()
        {
            _bitOffset += Length;
            _address += (uint)(_bitOffset >> 3);
            _bitOffset &= 7;
        }
    }
}
