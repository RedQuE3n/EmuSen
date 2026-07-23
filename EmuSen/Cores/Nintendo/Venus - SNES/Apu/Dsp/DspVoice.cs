using System;

namespace EmuSen.Cores.Nintendo.Venus.Apu
{
    internal enum EnvelopeStage { Attack, Decay, Sustain, Release, Off }

    // One S-DSP voice: BRR sample playback and the ADSR/GAIN envelope that
    // scales it. Register field values are kept in sync by SDsp. See
    // Venus_APU.md §4 for what's not implemented.
    internal class DspVoice
    {
        // Rate/period table - see Venus_APU.md §4.4.
        private static readonly int[] PeriodTable =
        {
            int.MaxValue, 2048, 1536, 1280, 1024, 768, 640, 512, 384, 320, 256, 192,
            160, 128, 96, 80, 64, 48, 40, 32, 24, 20, 16, 12, 10, 8, 6, 5, 4, 3, 2, 1
        };

        private readonly BrrDecoder _brr = new BrrDecoder();
        private byte[] _ram = null!;

        private ushort _blockAddr;
        private short[] _currentBlock = new short[16];
        private int _blockPos;
        private int _sampleFrac; // 12-bit fractional playback position; 0x1000 = one full source sample
        private ushort _loopAddr;
        private bool _active;

        private EnvelopeStage _stage = EnvelopeStage.Off;
        private int _envelope; // 0-2047 (11-bit)
        private int _envelopeCounter;

        // Registers - kept current by SDsp.WriteRegister.
        public byte VolL, VolR, Srcn, Adsr1, Adsr2, Gain;
        public ushort Pitch;

        public bool Ended { get; private set; }

        public void Reset()
        {
            _brr.Reset();
            _blockAddr = 0;
            _blockPos = 0;
            _sampleFrac = 0;
            _loopAddr = 0;
            _active = false;
            _stage = EnvelopeStage.Off;
            _envelope = 0;
            _envelopeCounter = 0;
            Ended = false;
        }

        // Called on any write to ENDX - see Venus_APU.md §3.2.
        public void ClearEndedFlag() => Ended = false;

        public void AttachMemory(byte[] ram) => _ram = ram;

        // dirTableAddr is the resolved Sample Directory base (DIR register * 0x100).
        // Each entry is 4 bytes: 16-bit start address, 16-bit loop address.
        public void KeyOn(int dirTableAddr)
        {
            int entry = (dirTableAddr + Srcn * 4) & 0xFFFF;
            ushort startAddr = (ushort)(_ram[entry] | (_ram[(entry + 1) & 0xFFFF] << 8));
            _loopAddr = (ushort)(_ram[(entry + 2) & 0xFFFF] | (_ram[(entry + 3) & 0xFFFF] << 8));

            _blockAddr = startAddr;
            _brr.Reset();
            DecodeCurrentBlock();
            _blockPos = 0;
            _sampleFrac = 0;
            _envelope = 0;
            _envelopeCounter = 0;
            _stage = EnvelopeStage.Attack;
            _active = true;
            Ended = false;
        }

        public void KeyOff()
        {
            if (_active) _stage = EnvelopeStage.Release;
        }

        private void DecodeCurrentBlock()
        {
            _currentBlock = _brr.DecodeBlock(_ram, _blockAddr);
        }

        private void AdvanceSourceSample()
        {
            _blockPos++;
            if (_blockPos < 16) return;

            byte header = _ram[_blockAddr];
            if (BrrDecoder.IsEndBlock(header))
            {
                if (BrrDecoder.IsLoopBlock(header))
                {
                    // No _brr.Reset() here on purpose - see Venus_APU.md §4.2.
                    _blockAddr = _loopAddr;
                }
                else
                {
                    _active = false;
                    Ended = true;
                    _stage = EnvelopeStage.Off;
                    return;
                }
            }
            else
            {
                _blockAddr = (ushort)(_blockAddr + 9);
            }

            DecodeCurrentBlock();
            _blockPos = 0;
        }

        // Produces one output-rate sample; SDsp applies VolL/VolR and mixes.
        public short GetNextSample()
        {
            if (!_active) return 0;

            _sampleFrac += Pitch;
            while (_sampleFrac >= 0x1000 && _active)
            {
                _sampleFrac -= 0x1000;
                AdvanceSourceSample();
            }

            if (!_active) return 0;

            int raw = _currentBlock[_blockPos];
            int scaled = (raw * _envelope) >> 11;

            StepEnvelope();

            return (short)Math.Clamp(scaled, short.MinValue, short.MaxValue);
        }

        // Gates envelope steps to the period table's rate - see Venus_APU.md §4.4.
        private bool RateDue(int periodIndex)
        {
            if (periodIndex == 0) return false;
            int periodSamples = Math.Max(1, PeriodTable[periodIndex] / 32);
            _envelopeCounter++;
            if (_envelopeCounter >= periodSamples)
            {
                _envelopeCounter = 0;
                return true;
            }
            return false;
        }

        private void StepEnvelope()
        {
            if (_stage == EnvelopeStage.Off) return;

            if (_stage == EnvelopeStage.Release)
            {
                // Fixed rate, every sample, no period-table gating.
                _envelope -= 8;
                if (_envelope <= 0)
                {
                    _envelope = 0;
                    _stage = EnvelopeStage.Off;
                    _active = false;
                }
                return;
            }

            bool useAdsr = (Adsr1 & 0x80) != 0;
            if (useAdsr) StepAdsr();
            else StepGain();

            _envelope = Math.Clamp(_envelope, 0, 2047);
        }

        private void StepAdsr()
        {
            switch (_stage)
            {
                case EnvelopeStage.Attack:
                {
                    int attackRate = Adsr1 & 0x0F;
                    if (RateDue(attackRate * 2 + 1))
                    {
                        _envelope += (attackRate == 0x0F) ? 1024 : 32;
                        if (_envelope >= 2047)
                        {
                            _envelope = 2047;
                            _stage = EnvelopeStage.Decay;
                        }
                    }
                    break;
                }
                case EnvelopeStage.Decay:
                {
                    int decayRate = (Adsr1 >> 4) & 0x07;
                    int sustainThreshold = (((Adsr2 >> 5) & 0x07) + 1) * 256;
                    if (RateDue(decayRate * 2 + 16))
                    {
                        _envelope -= 1 + (_envelope >> 8);
                        if (_envelope <= sustainThreshold)
                        {
                            _stage = EnvelopeStage.Sustain;
                        }
                    }
                    break;
                }
                case EnvelopeStage.Sustain:
                {
                    int sustainRate = Adsr2 & 0x1F;
                    if (RateDue(sustainRate))
                    {
                        _envelope -= 1 + (_envelope >> 8);
                    }
                    break;
                }
            }
        }

        private void StepGain()
        {
            if ((Gain & 0x80) == 0)
            {
                // Direct gain scaling - not independently cross-checked, see
                // Venus_APU.md §4.5.
                _envelope = (Gain & 0x7F) * 16;
                return;
            }

            int mode = (Gain >> 5) & 0x03;
            int rate = Gain & 0x1F;
            if (!RateDue(rate)) return;

            switch (mode)
            {
                case 0: _envelope -= 32; break;                                  // decrease linear
                case 1: _envelope -= 1 + (_envelope >> 8); break;                // decrease exponential
                case 2: _envelope += 32; break;                                  // increase linear
                case 3: _envelope += (_envelope < 0x600) ? 32 : 8; break;        // increase bent line
            }
        }
    }
}
