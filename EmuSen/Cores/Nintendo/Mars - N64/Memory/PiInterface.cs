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

        private readonly MarsBus _bus;

        private uint _dramAddress;
        private uint _cartAddress;
        private bool _interrupt;

        public PiInterface(MarsBus bus) => _bus = bus;

        public uint Read32(uint offset)
        {
            switch (offset & 0x3C)
            {
                case DramAddress: return _dramAddress;
                case CartAddress: return _cartAddress;

                // Never busy, because every transfer has already finished - see Mars_Memory.md §7.1.
                case Status: return _interrupt ? StatusInterrupt : 0;

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
                    if ((value & 0x02) != 0) _interrupt = false;
                    break;
            }
        }

        // Length is encoded one short, like the signal processor's - see Mars_Memory.md §7.
        private void Transfer(uint encoded, bool toCartridge)
        {
            uint length = (encoded & 0x00FF_FFFF) + 1;

            for (uint i = 0; i < length; i++)
            {
                if (toCartridge) _bus.Write8(_cartAddress + i, _bus.Read8(_dramAddress + i));
                else _bus.Write8(_dramAddress + i, _bus.Read8(_cartAddress + i));
            }

            _dramAddress += length;
            _cartAddress += length;
            _interrupt = true;
        }
    }
}
