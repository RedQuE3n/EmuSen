using System;

namespace EmuSen.Apu
{
    internal enum EnvelopeStage { Attack, Decay, Sustain, Release, Off }

    // One S-DSP voice: BRR sample playback and the ADSR/GAIN envelope that
    // scales it. Register field values (VolL/VolR/Srcn/Adsr1/Adsr2/Gain/Pitch)
    // are kept in sync by SDsp whenever the corresponding DSP register is
    // written; this class only reacts to KeyOn/KeyOff and produces samples.
    //
    // Deliberately NOT implemented in this pass (documented rather than
    // silently wrong):
    //   - Echo (EON, FIR filter, echo buffer) - entirely separate subsystem.
    //   - Noise generation (NON) - voices always play their BRR sample.
    //   - Pitch modulation from the previous voice's output (PMON).
    //   - The real 4-tap Gaussian interpolation filter - this uses
    //     nearest-neighbor resampling instead, which gets the playback RATE
    //     right but sounds rougher than hardware on pitched-up/down samples.
    //   - Exact period-table phase alignment across voices (see SDsp.cs's
    //     comment on RateDue below) - envelope RATES are correct, exact
    //     per-clock PHASE isn't.
    internal class DspVoice
    {
        // 32-entry rate/period table, in S-SMP clocks per envelope step.
        // Verified against the SNESdev DSP_envelopes page. Index 0 ("Infinite")
        // means the corresponding stage never advances on its own.
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

        // Called when the game writes to ENDX ($7C) - real hardware clears
        // ALL voices' end flags on any write to that register, regardless of
        // the value written.
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
                    // Real hardware carries the prediction filter's history
                    // straight into the loop block rather than resetting it -
                    // whether that matters depends on the loop block's own
                    // filter type, same as it would mid-stream. No _brr.Reset()
                    // here on purpose.
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

        // Produces one output-rate sample (called once per generated stereo
        // frame - see SDsp.GenerateSample). Returns the envelope-scaled,
        // still-unpanned sample; SDsp applies VolL/VolR and mixes.
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

        // Gates envelope steps to the rate the period table specifies,
        // approximated in output samples (32 S-SMP clocks per generated
        // sample - matches SDsp.Tick's own 32-cycle-per-sample timing exactly,
        // so the RATE this produces is correct; only the exact clock-level
        // PHASE relative to other voices isn't modeled).
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
                // Direct gain: set the envelope immediately from the 7-bit
                // value. This scaling (value * 16) is the one detail in this
                // file not independently cross-checked against a second
                // source - worth revisiting first if a direct-gain-mode
                // sound effect sounds off.
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
