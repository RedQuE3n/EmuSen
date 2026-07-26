using System;
using System.Collections.Generic;
using EmuSen.Audio;

namespace EmuSen.Cores.Nintendo.Venus.Apu
{
    // The S-DSP: register file, the 8 voices (DspVoice.cs), and final stereo
    // mixing - see Venus_APU.md §3.
    public class SDsp
    {
        private byte[] _registers = new byte[128];
        private byte _registerAddress;
        private byte[] _ram = null!;

        private readonly DspVoice[] _voices = new DspVoice[8];
        private byte _prevKon;
        private byte _prevKoff;

        // Global (non-per-voice) register offsets - see Venus_APU.md §3 /
        // anomie's DSP doc for the full register map. Named here purely
        // for readability; the underlying storage is still the flat
        // _registers[128] array everything else already uses.
        private const int RegMasterVolLeft = 0x0C;
        private const int RegEchoFeedback = 0x0D;
        private const int RegMasterVolRight = 0x1C;
        private const int RegEchoVolLeft = 0x2C;
        private const int RegEchoVolRight = 0x3C;
        private const int RegKeyOn = 0x4C;
        private const int RegEchoOn = 0x4D; // per-voice bitmask: which voices feed the echo buffer
        private const int RegKeyOff = 0x5C;
        private const int RegSourceDir = 0x5D;
        private const int RegFlags = 0x6C;
        private const int RegEchoRingBufferAddr = 0x6D;
        private const int RegEchoDelay = 0x7D; // EDL - low nibble, buffer length = EDL*2KB

        // Echo unit state - the one whole S-DSP subsystem that was
        // entirely unimplemented before this (see the DspVoice rewrite's
        // own comment on why it was deferred as separate work). An 8-tap
        // FIR filter applied to a rolling window of samples read one at a
        // time from a ring/delay buffer in SPC RAM, fed by whichever
        // voices have their EON bit set, with feedback (EFB) mixing the
        // filtered result back into what gets written to the ring buffer
        // each sample - ported from Mesen2's Dsp.cpp Echo* methods,
        // flattened into one pass per output sample the same way
        // DspVoice's own rewrite flattened voice processing (the exact
        // 32-step interleaved microcode schedule isn't replicated, just
        // the logical read-before-write ordering that actually matters
        // for correct output).
        private readonly short[] _echoHistoryL = new short[8];
        private readonly short[] _echoHistoryR = new short[8];
        private int _echoHistoryPos;
        private int _echoOffset;
        private int _echoLength;

        // Pending output samples, not game state - excluded from save states
        // the same reasoning as Renderer's own pixel buffer. Explicit backing
        // field (not an auto-property) specifically so [SkipInState], which
        // only targets fields, can actually be applied to it.
        [EmuSen.Common.SkipInState]
        private Queue<short> _audioBuffer = new Queue<short>();
        public Queue<short> AudioBuffer => _audioBuffer;

        private int _dspCycles;

        // Debug-only per-voice mute mask for the `mute`/`channels` commands
        // (see GetVoiceDebugInfo/SetVoiceMuted below) - NOT real hardware
        // state (there's no such SNES register), so excluded from save
        // states the same way _sheetPixels/_audioBuffer are. A muted voice
        // still runs GetNextSample() every tick (so its own envelope/
        // playback position doesn't desync from "what really happened"),
        // it's just excluded from the final mix and from feeding the echo
        // buffer - letting a suspected-broken instrument be isolated
        // (mute everything else, dump audio) or ruled out (mute just it,
        // confirm the rest of the mix is unaffected) without needing a
        // separate solo-rendering pipeline.
        [EmuSen.Common.SkipInState]
        private byte _debugMuteMask;

        public SDsp()
        {
            for (int i = 0; i < 8; i++) _voices[i] = new DspVoice();
        }

        // Called once by Spc700's constructor so voices can read BRR sample
        // data and the sample directory table directly out of SPC RAM.
        public void AttachMemory(byte[] ram)
        {
            _ram = ram;
            foreach (var voice in _voices) voice.AttachMemory(ram);
        }

        public void Reset()
        {
            Array.Clear(_registers, 0, _registers.Length);

            // Real hardware's FLG register always reads as 0xE0 right
            // after power-on/reset (soft-reset + mute + echo-disable bits
            // all set), regardless of what a plain register-clear would
            // otherwise leave it as - see anomie's DSP doc. Without this,
            // FLG's echo-disable bit (0x20) reads as clear (enabled) with
            // ESA still at its cleared value of 0, so the newly-added echo
            // unit would start writing into zero-page SPC RAM - where the
            // sound driver's own variables and call stack live - before
            // the game ever gets a chance to configure or disable it,
            // corrupting the driver entirely (caught as "audio went
            // completely silent" the first time echo was added).
            _registers[RegFlags] = 0xE0;

            _registerAddress = 0;
            _dspCycles = 0;
            _prevKon = 0;
            _prevKoff = 0;
            Array.Clear(_echoHistoryL, 0, 8);
            Array.Clear(_echoHistoryR, 0, 8);
            _echoHistoryPos = 0;
            _echoOffset = 0;
            _echoLength = 0;
            foreach (var voice in _voices) voice.Reset();
            AudioBuffer.Clear();
        }

        // Stores the RAW byte written, unmasked - only 7 bits are actually
        // used to index the 128-entry register file (masked at each point
        // of use below), but real hardware's address latch still holds
        // and echoes back the full 8 bits on a subsequent read, including
        // the otherwise-unused top bit. Previously masked at write time,
        // which was unrecoverable on read - caught via the SpcValidation
        // harness (an "OR A, dp" test reading this port back got the
        // masked value XORed against the real one, differing by exactly
        // bit 7). Almost certainly inaudible in any real game (nothing
        // sane relies on this bit surviving a round trip), fixed anyway
        // since it was cheap once found.
        public void SetRegisterAddress(byte address)
        {
            _registerAddress = address;
        }

        public byte GetRegisterAddress()
        {
            return _registerAddress;
        }

        public byte ReadRegister()
        {
            int index = _registerAddress & 0x7F;
            // ENDX - see Venus_APU.md §3.2.
            if (index == 0x7C)
            {
                byte endx = 0;
                for (int v = 0; v < 8; v++)
                {
                    if (_voices[v].Ended) endx |= (byte)(1 << v);
                }
                return endx;
            }
            return _registers[index];
        }

        // Non-mutating register peek for the debug toolchain
        // (SnesDebugTarget.GetApuRegisters) - unlike ReadRegister() above,
        // this never touches _registerAddress, so a debugger inspecting
        // DSP state can't disturb whatever multi-step address/data
        // sequence the actual sound driver is mid-way through.
        public byte PeekRegister(int index)
        {
            index &= 0x7F;
            if (index == 0x7C)
            {
                byte endx = 0;
                for (int v = 0; v < 8; v++)
                {
                    if (_voices[v].Ended) endx |= (byte)(1 << v);
                }
                return endx;
            }
            return _registers[index];
        }

        public void WriteRegister(byte data)
        {
            int index = _registerAddress & 0x7F;
            _registers[index] = data;

            // Any write to ENDX clears all its bits - see Venus_APU.md §3.2.
            if (index == 0x7C)
            {
                foreach (var voice in _voices) ClearEnded(voice);
                return;
            }

            int voiceIdx = index >> 4;
            int voiceReg = index & 0x0F;
            if (voiceIdx < 8)
            {
                DspVoice v = _voices[voiceIdx];
                switch (voiceReg)
                {
                    case 0x0: v.VolL = data; break;
                    case 0x1: v.VolR = data; break;
                    case 0x2: v.Pitch = (ushort)((v.Pitch & 0xFF00) | data); break;
                    case 0x3: v.Pitch = (ushort)((v.Pitch & 0x00FF) | ((data & 0x3F) << 8)); break;
                    case 0x4: v.Srcn = data; break;
                    case 0x5: v.Adsr1 = data; break;
                    case 0x6: v.Adsr2 = data; break;
                    case 0x7: v.Gain = data; break;
                    // 0x8 (ENVX)/0x9 (OUTX) writes accepted but ignored - unused for voice logic.
                }
            }
        }

        private void ClearEnded(DspVoice voice)
        {
            voice.ClearEndedFlag();
        }

        public void Tick(int cycles)
        {
            int cyclesPerSample = 1024000 / AudioSettings.SampleRate;
            _dspCycles += cycles;
            while (_dspCycles >= cyclesPerSample)
            {
                _dspCycles -= cyclesPerSample;
                GenerateSample();
            }
        }

        // Diagnostic only (see DebugSettings.DspKeyOnLogging) - lets a
        // KeyOn log line show how far apart consecutive triggers actually
        // are, to distinguish a legitimate fast rhythmic pattern from a
        // suspicious retrigger burst.
        private long _sampleCounter;

        private void GenerateSample()
        {
            _sampleCounter++;
            ProcessKeyEvents();

            short leftSample = 0;
            short rightSample = 0;

            if (AudioSettings.AudioEnabled && !AudioSettings.Muted)
            {
                int dryL = 0, dryR = 0;
                int echoInputL = 0, echoInputR = 0;
                byte echoOn = _registers[RegEchoOn];

                for (int i = 0; i < 8; i++)
                {
                    var voice = _voices[i];
                    short s = voice.GetNextSample();
                    // Debug-only solo/mute (see _debugMuteMask's own
                    // comment) - GetNextSample() above still ran, so this
                    // voice's own playback/envelope state stays in sync;
                    // it's just excluded from the mix and from feeding echo.
                    if ((_debugMuteMask & (1 << i)) != 0) continue;
                    int contribL = (s * (sbyte)voice.VolL) >> 7;
                    int contribR = (s * (sbyte)voice.VolR) >> 7;
                    dryL += contribL;
                    dryR += contribR;
                    if ((echoOn & (1 << i)) != 0)
                    {
                        echoInputL += contribL;
                        echoInputR += contribR;
                    }
                }

                sbyte mvolL = (sbyte)_registers[RegMasterVolLeft];
                sbyte mvolR = (sbyte)_registers[RegMasterVolRight];
                dryL = (dryL * mvolL) >> 7;
                dryR = (dryR * mvolR) >> 7;

                (int echoInL, int echoInR) = ProcessEcho(echoInputL, echoInputR);

                sbyte evolL = (sbyte)_registers[RegEchoVolLeft];
                sbyte evolR = (sbyte)_registers[RegEchoVolRight];
                int mixL = dryL + ((echoInL * evolL) >> 7);
                int mixR = dryR + ((echoInR * evolR) >> 7);

                mixL = (int)(mixL * AudioSettings.MasterVolume);
                mixR = (int)(mixR * AudioSettings.MasterVolume);

                // FLG bit 6 (real hardware's own software mute) only
                // silences the final DAC-bound output - voice/envelope
                // stepping and the echo buffer's own read/write/feedback
                // above still run every sample regardless, matching real
                // hardware (muting doesn't pause the echo delay line).
                bool hardwareMuted = (_registers[RegFlags] & 0x40) != 0;
                if (!hardwareMuted)
                {
                    leftSample = (short)Math.Clamp(mixL, short.MinValue, short.MaxValue);
                    rightSample = (short)Math.Clamp(mixR, short.MinValue, short.MaxValue);
                }
            }
            else
            {
                // Still advance voice playback state when muted - see Venus_APU.md §3.4.
                foreach (var voice in _voices) voice.GetNextSample();
            }

            AudioBuffer.Enqueue(leftSample);
            AudioBuffer.Enqueue(rightSample);

            if (AudioBuffer.Count > AudioSettings.AudioBufferMaxSamples)
            {
                AudioBuffer.Dequeue();
                AudioBuffer.Dequeue();
            }
        }

        // The 8-tap FIR echo unit - ported from Mesen2's Dsp.cpp Echo*
        // methods (see this file's own header comment on the fields
        // involved). dryEchoInputL/R is the pre-computed sum of just the
        // EON-enabled voices' output (separate from the full dry mix every
        // voice contributes to regardless of EON) - this is what actually
        // feeds the delay line, mixed with feedback from the existing
        // (filtered) echo signal. Returns the FIR-filtered echo signal
        // (before EVOL scaling), which is also what SDsp.GenerateSample
        // combines with the dry mix for final output.
        private (int echoInL, int echoInR) ProcessEcho(int dryEchoInputL, int dryEchoInputR)
        {
            int esa = _registers[RegEchoRingBufferAddr] << 8;
            uint pointer = (uint)(esa + _echoOffset) & 0xFFFF;

            _echoHistoryPos = (_echoHistoryPos + 1) & 0x07;
            short sampleL = (short)(_ram[pointer] | (_ram[(pointer + 1) & 0xFFFF] << 8));
            short sampleR = (short)(_ram[(pointer + 2) & 0xFFFF] | (_ram[(pointer + 3) & 0xFFFF] << 8));
            _echoHistoryL[_echoHistoryPos] = (short)(sampleL >> 1);
            _echoHistoryR[_echoHistoryPos] = (short)(sampleR >> 1);

            int firL = 0, firR = 0;
            for (int i = 0; i < 8; i++)
            {
                sbyte coeff = (sbyte)_registers[0x0F + (i << 4)];
                int histPos = (_echoHistoryPos + i + 1) & 0x07;
                firL += (_echoHistoryL[histPos] * coeff) >> 6;
                firR += (_echoHistoryR[histPos] * coeff) >> 6;
            }
            firL = Math.Clamp(firL, short.MinValue, short.MaxValue) & ~1;
            firR = Math.Clamp(firR, short.MinValue, short.MaxValue) & ~1;

            // Feedback: mix the freshly-filtered echo signal (scaled by
            // EFB) back into the dry, EON-gated voice sum - this combined
            // value is what actually gets written to the delay line, so
            // each repeat blends new input with a scaled copy of the
            // previous repeat rather than just replaying the raw input.
            sbyte efb = (sbyte)_registers[RegEchoFeedback];
            int echoOutL = Math.Clamp(dryEchoInputL + ((firL * efb) >> 7), short.MinValue, short.MaxValue) & ~1;
            int echoOutR = Math.Clamp(dryEchoInputR + ((firR * efb) >> 7), short.MinValue, short.MaxValue) & ~1;

            // FLG bit 5 clear = echo buffer writes enabled ("ECEN" is
            // active-low - see Venus_APU.md §3 / anomie's DSP doc).
            if ((_registers[RegFlags] & 0x20) == 0)
            {
                _ram[pointer] = (byte)(echoOutL & 0xFF);
                _ram[(pointer + 1) & 0xFFFF] = (byte)((echoOutL >> 8) & 0xFF);
                _ram[(pointer + 2) & 0xFFFF] = (byte)(echoOutR & 0xFF);
                _ram[(pointer + 3) & 0xFFFF] = (byte)((echoOutR >> 8) & 0xFF);
            }

            // EDL (buffer length) is only re-read at the instant the
            // offset wraps back to the start - a write to EDL takes
            // effect on the NEXT lap of the delay line, not immediately,
            // matching real hardware.
            if (_echoOffset == 0)
            {
                _echoLength = (_registers[RegEchoDelay] & 0x0F) << 11;
            }
            _echoOffset += 4;
            if (_echoOffset >= _echoLength) _echoOffset = 0;

            return (firL, firR);
        }

        // KON/KOFF edge detection - see Venus_APU.md §3.3.
        private void ProcessKeyEvents()
        {
            byte kon = _registers[0x4C];
            byte koff = _registers[0x5C];
            byte dir = _registers[0x5D];
            int dirTableAddr = dir << 8;

            byte konRising = (byte)(kon & ~_prevKon);
            byte koffRising = (byte)(koff & ~_prevKoff);

            for (int v = 0; v < 8; v++)
            {
                bool konBit = (konRising & (1 << v)) != 0;
                bool koffBit = (koffRising & (1 << v)) != 0;

                if (konBit) _voices[v].KeyOn(dirTableAddr, _sampleCounter);
                // KeyOff wins if both are newly set - see Venus_APU.md §3.3.
                if (koffBit) _voices[v].KeyOff();
            }

            _prevKon = kon;
            _prevKoff = koff;
        }

        // Plain data snapshot of one voice's current debug-relevant state -
        // backs SnesDebugTarget.GetAudioChannels() (see IDebugTarget's own
        // comment on why that's modeled as a generic "channel" concept
        // rather than SNES-specific). Everything here already exists on
        // DspVoice; this just gathers it into one struct instead of
        // exposing 10 separate getters to the debug layer.
        public readonly struct VoiceDebugInfo
        {
            public bool Active { get; }
            public int Envelope { get; }
            public string Stage { get; }
            public byte VolL { get; }
            public byte VolR { get; }
            public byte Srcn { get; }
            public ushort Pitch { get; }
            public byte Adsr1 { get; }
            public byte Adsr2 { get; }
            public byte Gain { get; }
            public bool Ended { get; }
            public bool Muted { get; }
            public int KeyOnCount { get; }
            public long LastKeyOnSample { get; }

            public VoiceDebugInfo(bool active, int envelope, string stage, byte volL, byte volR, byte srcn, ushort pitch, byte adsr1, byte adsr2, byte gain, bool ended, bool muted, int keyOnCount, long lastKeyOnSample)
            {
                Active = active;
                Envelope = envelope;
                Stage = stage;
                VolL = volL;
                VolR = volR;
                Srcn = srcn;
                Pitch = pitch;
                Adsr1 = adsr1;
                Adsr2 = adsr2;
                Gain = gain;
                Ended = ended;
                Muted = muted;
                KeyOnCount = keyOnCount;
                LastKeyOnSample = lastKeyOnSample;
            }
        }

        public VoiceDebugInfo GetVoiceDebugInfo(int index)
        {
            DspVoice v = _voices[index];
            bool muted = (_debugMuteMask & (1 << index)) != 0;
            return new VoiceDebugInfo(v.IsActive, v.EnvelopeLevel, v.StageName, v.VolL, v.VolR, v.Srcn, v.Pitch, v.Adsr1, v.Adsr2, v.Gain, v.Ended, muted, v.KeyOnCount, v.LastKeyOnSample);
        }

        public void SetVoiceMuted(int index, bool muted)
        {
            if (index < 0 || index >= 8) return;
            if (muted) _debugMuteMask |= (byte)(1 << index);
            else _debugMuteMask &= (byte)~(1 << index);
        }

        // Current sample-clock position, for VoiceDebugInfo.LastKeyOnSample
        // to be interpreted against ("how many samples ago did this voice
        // last trigger") without the debug layer needing its own copy of
        // this counter.
        public long SampleCounter => _sampleCounter;
    }
}
