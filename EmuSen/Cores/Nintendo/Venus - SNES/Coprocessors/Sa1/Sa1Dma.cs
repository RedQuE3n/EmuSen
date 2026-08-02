using EmuSen.Common;

namespace EmuSen.Cores.Nintendo.Venus.Coprocessors.Sa1
{
    // SA-1 DMA ($2230-$2239, $2240-$224F). Normal block copy is implemented;
    // both character-conversion modes are decoded and reported but not yet
    // converted - see Venus_SA1.md §8.
    public sealed class Sa1Dma
    {
        private const int SourceRom = 0;
        private const int SourceBwRam = 1;
        private const int SourceIRam = 2;

        private const int DestIRam = 0;
        private const int DestBwRam = 1;

        [SkipInState] private readonly Sa1 _sa1;

        private byte _control;
        private byte _characterParams;
        private uint _source, _dest;
        private ushort _counter;
        private byte[] _brf = new byte[16];

        // One line of log per ROM, not per attempted transfer.
        private bool _reportedCharacterConversion;

        public Sa1Dma(Sa1 sa1)
        {
            _sa1 = sa1;
        }

        private bool Enabled => (_control & 0x80) != 0;
        private bool CharacterConversion => (_control & 0x20) != 0;
        private int Source => _control & 0x03;
        private int Dest => (_control & 0x08) != 0 ? DestBwRam : DestIRam;

        public void WriteControl(byte value) => _control = value;
        public void WriteCharacterParams(byte value) => _characterParams = value;

        public void WriteSourceByte(int index, byte value)
        {
            int shift = index * 8;
            _source = (_source & ~(0xFFu << shift)) | ((uint)value << shift);
        }

        public void WriteCounterByte(int index, byte value)
        {
            int shift = index * 8;
            _counter = (ushort)((_counter & ~(0xFF << shift)) | (value << shift));
        }

        // Writing the byte that completes the destination address is the
        // trigger, and which byte that is depends on the destination device.
        public void WriteDestByte(int index, byte value)
        {
            int shift = index * 8;
            _dest = (_dest & ~(0xFFu << shift)) | ((uint)value << shift);

            if (!Enabled || CharacterConversion) return;
            if (index == 1 && Dest == DestIRam) RunNormal();
            if (index == 2 && Dest == DestBwRam) RunNormal();
        }

        public void WriteBrf(int index, byte value)
        {
            _brf[index] = value;

            // Completing either half of the register file triggers a type-2 conversion.
            if (index != 0x07 && index != 0x0F) return;
            if (!Enabled || !CharacterConversion) return;
            ReportUnimplementedCharacterConversion(type: 2);
        }

        private void RunNormal()
        {
            while (_counter-- > 0)
            {
                uint from = _source++;
                uint to = _dest++;

                // Same-device transfers are ignored by the hardware.
                if (Source == SourceBwRam && Dest == DestBwRam) continue;
                if (Source == SourceIRam && Dest == DestIRam) continue;

                byte data = Source switch
                {
                    SourceBwRam => _sa1.ReadBwRamByte((int)from),
                    SourceIRam => _sa1.IRam[from & (Sa1.IRamSize - 1)],
                    _ => _sa1.ReadSa1(from),
                };

                if (Dest == DestIRam) _sa1.IRam[to & (Sa1.IRamSize - 1)] = data;
                else _sa1.WriteBwRamByte((int)to, data);
            }

            _counter = 0;
            _sa1.RaiseDmaIrq();
        }

        // Type 1 is driven by the S-CPU reading through the $6000-$7FFF window
        // mid-transfer, type 2 by the BRF staging buffer. Neither is emulated
        // yet; a game that needs one gets wrong tile data rather than a hang,
        // so say so loudly instead of failing silently.
        private void ReportUnimplementedCharacterConversion(int type)
        {
            if (_reportedCharacterConversion) return;
            _reportedCharacterConversion = true;
            System.Console.WriteLine(
                $"[SA1] Character-conversion DMA type {type} requested (CDMA=0x{_characterParams:X2}) - not implemented, graphics from it will be wrong.");
        }

        public void ReportIfCharacterConversionArmed()
        {
            if (Enabled && CharacterConversion) ReportUnimplementedCharacterConversion(type: 1);
        }
    }
}
