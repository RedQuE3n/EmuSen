namespace EmuSen.Cores.Nintendo.Mars.Memory
{
    // The serial interface: sixty-four bytes each way between memory and PIF RAM - see Mars_Serial.md §2.
    public sealed class SiInterface
    {
        public const uint DramAddress = 0x00;
        public const uint PifAddressRead = 0x04;
        public const uint PifAddressWrite = 0x10;
        public const uint Status = 0x18;

        public const uint StatusDmaBusy = 0x0001;
        public const uint StatusIoBusy = 0x0002;
        public const uint StatusDmaError = 0x0008;
        public const uint StatusInterrupt = 0x1000;

        // Four ports, of which the first holds a controller until a frontend says otherwise - see §3.1.
        public readonly Controller[] Controllers = { new() { Present = true }, new(), new(), new() };

        private readonly MemoryBus _bus;

        private uint _dramAddress;

        public SiInterface(MemoryBus bus) => _bus = bus;

        public uint Read32(uint offset)
        {
            switch (offset & 0x1C)
            {
                case DramAddress: return _dramAddress;

                // Never busy, because every transfer has already finished - see §2.1.
                case Status: return _bus.Mi.Pending.HasFlag(MiInterrupt.SerialInterface) ? StatusInterrupt : 0;

                default: return 0;
            }
        }

        public void Write32(uint offset, uint value)
        {
            switch (offset & 0x1C)
            {
                case DramAddress:
                    _dramAddress = value & 0x00FF_FFF8;
                    break;

                case PifAddressRead:
                    Transfer(toPif: false);
                    break;

                case PifAddressWrite:
                    Transfer(toPif: true);
                    break;

                case Status:
                    _bus.Mi.Clear(MiInterrupt.SerialInterface);
                    break;
            }
        }

        // Sixty-four bytes, and on the way in the PIF runs whatever block the game asked it to - see §2.
        private void Transfer(bool toPif)
        {
            byte[] ram = _bus.PifRam;

            for (uint i = 0; i < MemoryMap.PifRamSize; i++)
            {
                uint address = _dramAddress + i;
                if (address >= _bus.Rdram.Length) break;

                if (toPif) ram[i] = _bus.Rdram[address];
                else _bus.Rdram[address] = ram[i];
            }

            if (toPif && (ram[^1] & 1) != 0) Joybus.Run(ram, Controllers);

            _bus.Mi.Raise(MiInterrupt.SerialInterface);
        }
    }
}
