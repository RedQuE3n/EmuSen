using EmuSen.Cores.Nintendo.Mars.Rom;

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

        // The bit in PIF RAM's last byte that asks the cartridge's CIC to answer a challenge - see Mars_Boot.md §7.
        public const byte ChallengeRequest = 0x02;
        public const int ChallengeAt = 0x30;
        public const int ChallengeLength = 15;

        // Four ports, of which the first holds a controller until a frontend says otherwise - see §3.1.
        public readonly Controller[] Controllers = { new() { Present = true }, new(), new(), new() };

        [EmuSen.Common.SkipInState] private readonly MemoryBus _bus;

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

            // The PIF works as its RAM is read out: the challenge if one is asked, the joybus walk otherwise - see Mars_Serial.md §2.
            if (!toPif)
            {
                if ((ram[^1] & ChallengeRequest) != 0) AnswerChallenge(ram);
                else Joybus.Run(ram, Controllers, _bus.Save);
            }

            _bus.Dp.WaitForRange(_dramAddress, MemoryMap.PifRamSize, 6);

            for (uint i = 0; i < MemoryMap.PifRamSize; i++)
            {
                uint address = _dramAddress + i;
                if (address >= _bus.Rdram.Length) break;

                if (toPif) ram[i] = _bus.Rdram[address];
                else _bus.Rdram[address] = ram[i];
            }

            _bus.Written++;


            _bus.Mi.Raise(MiInterrupt.SerialInterface);
        }

        // Only a 6105 answers; for any other chip the request is dropped and the challenge left where it was - see Mars_Boot.md §7.2.
        private void AnswerChallenge(byte[] ram)
        {
            if (_bus.Cart?.CicChip == CicChip.Nus6105)
            {
                ram[ChallengeAt - 2] = 0;
                ram[ChallengeAt - 1] = 0;
                Cic.Respond(ram.AsSpan(ChallengeAt, ChallengeLength), ram.AsSpan(ChallengeAt, ChallengeLength));
            }

            ram[^1] &= unchecked((byte)~ChallengeRequest);
        }
    }
}
