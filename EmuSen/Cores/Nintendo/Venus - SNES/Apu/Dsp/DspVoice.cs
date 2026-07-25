using System;
using EmuSen.Audio;
using EmuSen.Debug;

namespace EmuSen.Cores.Nintendo.Venus.Apu
{
    internal enum EnvelopeStage { Attack, Decay, Sustain, Release, Off }

    // One S-DSP voice: BRR sample playback (with real Gaussian
    // interpolation and the hardware's KeyOn startup delay - both ported
    // from Mesen2's DspVoice.cpp after the first pass at this, which
    // decoded a whole 16-sample block up front and picked the nearest raw
    // sample with no interpolation at all, turned out to still sound
    // garbled once the separate SPC700 clock-rate bug was fixed) and the
    // ADSR/GAIN envelope that scales it. See Venus_APU.md §4 for what's
    // still not implemented - echo/FIR is the big remaining gap, not
    // attempted here; it's a genuinely separate subsystem, not something
    // that would explain garbled *voice* output.
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

        // Circular buffer of the last 12 decoded (doubled-representation -
        // see BrrDecoder.DecodeQuad) samples - holds more than just the
        // current quad specifically so Gaussian interpolation has real
        // lookback samples available right at a quad/block boundary,
        // matching real hardware rather than starting each new block from
        // a clean slate.
        private readonly short[] _sampleBuffer = new short[12];
        private int _bufferPos; // cycles 0, 4, 8 - see DecodeNextQuad's own comment

        private ushort _brrAddress;   // current 9-byte BRR block's address
        private int _brrOffset;       // 1,3,5,7 - byte offset of the next quad to decode within the block
        private byte _brrHeader;      // header byte of the current block (bytes/filter/shift/end/loop flags)
        private ushort _pendingStartAddr;
        private ushort _pendingLoopAddr;

        // 15-bit fixed-point playback position within the current quad -
        // bits 12-13 select which of the last 4 decoded samples to center
        // interpolation on, bits 0-11 are the Gaussian fractional weight.
        // Wraps by masking off bit 14 (not a full mod), matching hardware -
        // see GetNextSample's own comment on the exact sequence this
        // mirrors from Mesen's Step3c/Step4.
        private int _interpolationPos;

        // Real hardware delays a newly key-on'd voice by 5 samples before
        // it outputs anything real (envelope pinned to 0 the whole time) -
        // skipping this previously meant a note's very first samples came
        // from a not-yet-properly-primed decode state.
        private int _keyOnDelay;

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
            _brrAddress = 0;
            _brrOffset = 1;
            _brrHeader = 0;
            _pendingStartAddr = 0;
            _pendingLoopAddr = 0;
            Array.Clear(_sampleBuffer, 0, _sampleBuffer.Length);
            _bufferPos = 0;
            _interpolationPos = 0;
            _keyOnDelay = 0;
            _active = false;
            _stage = EnvelopeStage.Off;
            _envelope = 0;
            _envelopeCounter = 0;
            Ended = false;
        }

        // Called on any write to ENDX - see Venus_APU.md §3.2.
        public void ClearEndedFlag() => Ended = false;

        public void AttachMemory(byte[] ram) => _ram = ram;

        private long _lastKeyOnSample = long.MinValue;

        // dirTableAddr is the resolved Sample Directory base (DIR register * 0x100).
        // Each entry is 4 bytes: 16-bit start address, 16-bit loop address.
        public void KeyOn(int dirTableAddr, long sampleCounter = 0)
        {
            int entry = (dirTableAddr + Srcn * 4) & 0xFFFF;
            _pendingStartAddr = (ushort)(_ram[entry] | (_ram[(entry + 1) & 0xFFFF] << 8));
            _pendingLoopAddr = (ushort)(_ram[(entry + 2) & 0xFFFF] | (_ram[(entry + 3) & 0xFFFF] << 8));

            if (DebugSettings.DspKeyOnLogging)
            {
                byte header = _ram[_pendingStartAddr];
                long deltaSamples = _lastKeyOnSample == long.MinValue ? -1 : sampleCounter - _lastKeyOnSample;
                _lastKeyOnSample = sampleCounter;
                Console.WriteLine(
                    $"[DSP-KEYON] t={sampleCounter / (double)AudioSettings.SampleRate:F3}s (+{deltaSamples} samples since this voice's last) Srcn=0x{Srcn:X2} dir=0x{dirTableAddr:X4} entry=0x{entry:X4} " +
                    $"startAddr=0x{_pendingStartAddr:X4} loopAddr=0x{_pendingLoopAddr:X4} header=0x{header:X2} " +
                    $"(shift={(header >> 4) & 0xF} filter={(header >> 2) & 3} end={header & 1} loop={(header >> 1) & 1}) " +
                    $"pitch=0x{Pitch:X4} volL={(sbyte)VolL} volR={(sbyte)VolR} adsr1=0x{Adsr1:X2} adsr2=0x{Adsr2:X2} gain=0x{Gain:X2}");
            }

            _brr.Reset();
            _brrAddress = _pendingStartAddr;
            _brrOffset = 1;
            _brrHeader = _ram[_brrAddress];
            Array.Clear(_sampleBuffer, 0, _sampleBuffer.Length);
            _bufferPos = 0;
            _interpolationPos = 0;
            _keyOnDelay = 5;

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

        // Decodes the next 4 samples (one nibble-pair) into the circular
        // buffer and advances _brrOffset/_brrAddress to the following
        // quad/block - ported from Mesen's DecodeBrrSample + the
        // block-advance logic in its Step4. _bufferPos cycles 0/4/8
        // (matching Mesen's own `if(_bufferPos<=4) _bufferPos+=4; else
        // _bufferPos=0;`, not a plain modulo) so the buffer always keeps
        // the 3 most recent quads (12 samples) available for interpolation
        // lookback even right at a block boundary.
        private void DecodeNextQuad()
        {
            byte b0 = _ram[(_brrAddress + _brrOffset) & 0xFFFF];
            byte b1 = _ram[(_brrAddress + _brrOffset + 1) & 0xFFFF];
            short[] quad = _brr.DecodeQuad(_brrHeader, b0, b1);
            for (int i = 0; i < 4; i++) _sampleBuffer[_bufferPos + i] = quad[i];

            if (_bufferPos <= 4) _bufferPos += 4;
            else _bufferPos = 0;

            if (_brrOffset >= 7)
            {
                if (BrrDecoder.IsEndBlock(_brrHeader))
                {
                    if (BrrDecoder.IsLoopBlock(_brrHeader))
                    {
                        _brrAddress = _pendingLoopAddr;
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
                    _brrAddress = (ushort)(_brrAddress + 9);
                }
                _brrOffset = 1;
                _brrHeader = _ram[_brrAddress];
            }
            else
            {
                _brrOffset += 2;
            }
        }

        // Produces one output-rate sample; SDsp applies VolL/VolR and mixes.
        public short GetNextSample()
        {
            if (!_active) return 0;

            // Silent during the 5-sample KeyOn startup delay - real
            // hardware pins envelope/pitch to 0 the whole time. The
            // circular buffer is primed (zeroed at KeyOn) so interpolation
            // right at the end of the delay has real, if silent, history
            // to reference instead of stale data from a previous note.
            if (_keyOnDelay > 0)
            {
                _keyOnDelay--;
                if (_keyOnDelay == 0)
                {
                    // Force an immediate decode on the very next call
                    // instead of wasting one more sample on an empty
                    // buffer - matches the observable effect of hardware
                    // already having the sample pointer ready to go the
                    // instant the delay elapses.
                    _interpolationPos = 0x4000;
                }
                return 0;
            }

            // Interpolate using the CURRENT position (i.e. from wherever
            // the previous call's advance left it) before touching
            // anything else this call - matches the real chip evaluating
            // output from the position it already has, then deciding
            // whether a new quad is needed, then advancing for next time.
            short raw = DspInterpolation.Gauss4Point(_interpolationPos, _sampleBuffer, _bufferPos);

            if (_interpolationPos >= 0x4000)
            {
                DecodeNextQuad();
            }

            _interpolationPos = (_interpolationPos & 0x3FFF) + Pitch;
            if (_interpolationPos > 0x7FFF) _interpolationPos = 0x7FFF;

            if (!_active)
            {
                // DecodeNextQuad just hit a non-looping end block.
                StepEnvelope();
                return 0;
            }

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
