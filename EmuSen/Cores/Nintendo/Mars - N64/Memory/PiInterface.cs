using System;

namespace EmuSen.Cores.Nintendo.Mars.Memory
{
    // The peripheral interface: the cartridge's DMA, and the flag the corpus spins on - see Mars_Memory.md §7.
    public sealed class PiInterface
    {
        public const uint DramAddress = 0x00;
        public const uint CartAddress = 0x04;
        public const uint ReadLength = 0x08;
        public const uint WriteLength = 0x0C;
        public const uint Status = 0x10;

        public const uint StatusDmaBusy = 0x01;
        public const uint StatusIoBusy = 0x02;
        public const uint StatusError = 0x04;
        public const uint StatusInterrupt = 0x08;

        private const uint BlockSize = 128;
        private const uint RowSize = 0x800;

        private readonly MemoryBus _bus;

        private uint _dramAddress;
        private uint _cartAddress;

        public PiInterface(MemoryBus bus) => _bus = bus;

        public uint Read32(uint offset)
        {
            switch (offset & 0x3C)
            {
                case DramAddress: return _dramAddress;
                case CartAddress: return _cartAddress;

                // Never busy, because every transfer has already finished - see Mars_Memory.md §7.1.
                case Status: return _bus.Mi.Pending.HasFlag(MiInterrupt.PeripheralInterface) ? StatusInterrupt : 0;

                default: return 0;
            }
        }

        public void Write32(uint offset, uint value)
        {
            switch (offset & 0x3C)
            {
                case DramAddress:
                    _dramAddress = value & 0x00FF_FFFE;
                    break;

                case CartAddress:
                    _cartAddress = value & 0xFFFF_FFFE;
                    break;

                case ReadLength:
                    Transfer(value, toCartridge: true);
                    break;

                case WriteLength:
                    Transfer(value, toCartridge: false);
                    break;

                case Status:
                    if ((value & 0x02) != 0) _bus.Mi.Clear(MiInterrupt.PeripheralInterface);
                    break;
            }
        }

        // Length is encoded one short, like the signal processor's - see Mars_Memory.md §7.
        private void Transfer(uint encoded, bool toCartridge)
        {
            uint length = (encoded & 0x00FF_FFFF) + 1;

            if (toCartridge) ToCartridge(length);
            else FromCartridge(length);

            _bus.Mi.Raise(MiInterrupt.PeripheralInterface);
        }

        private void ToCartridge(uint length)
        {
            for (uint i = 0; i < length; i++) _bus.Write8(_cartAddress + i, _bus.Read8(_dramAddress + i));

            // The cartridge's bus is sixteen bits wide and RDRAM's sixty-four, so each address advances to its own multiple - see §7.2.
            _cartAddress += (length + 1) & ~1u;
            _dramAddress = (_dramAddress + length + 7) & ~7u;
        }

        // Blocks of at most 128 bytes, the first paying for a misaligned address twice - see Mars_Memory.md §7.3.
        private void FromCartridge(uint length)
        {
            uint remaining = length;
            uint largest = BlockSize;
            bool first = true;

            while (remaining > 0)
            {
                uint misaligned = _dramAddress & 7;
                uint toRowEnd = RowSize - (_dramAddress & (RowSize - 1));
                uint block = Math.Min(Math.Min(largest - misaligned, toRowEnd), remaining);
                uint read = (block + 1) & ~1u;

                int stored = (int)block - (int)misaligned;
                bool trimmed = first && block < BlockSize - 1 - misaligned;

                for (int at = 0; at < stored; at += 2)
                {
                    uint from = _cartAddress + (uint)at;
                    _bus.Write8(_dramAddress, _bus.Read8(from));

                    // A trimmed block ends on the first byte of its last pair; the address rounds to the same word either way - see §7.6.
                    if (!trimmed || at + 1 < stored) _bus.Write8(_dramAddress + 1, _bus.Read8(from + 1));
                    _dramAddress += 2;
                }

                _cartAddress += read;
                remaining = remaining > read ? remaining - read : 0;

                // A block that began within eight bytes of the end of a row shortens the next one - see §7.3.
                largest = toRowEnd < 8 ? BlockSize - misaligned : BlockSize;
                _dramAddress = (_dramAddress + 7) & ~7u;
                first = false;
            }
        }
    }
}
