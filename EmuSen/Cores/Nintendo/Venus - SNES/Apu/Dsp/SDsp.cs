using System;
using System.Collections.Generic;
using EmuSen.Audio;

namespace EmuSen.Cores.Nintendo.Venus.Apu
{
    // Register file, eight voices and final mixing - see Venus_APU.md §3.
    public class SDsp
    {
        private byte[] _registers = new byte[128];
        private byte _registerAddress;
        // Spc700.Ram, attached in AttachMemory - see EmuSen_Save_States.md §2.
        [EmuSen.Common.AliasOfSerializedField] private byte[] _ram = null!;

        private readonly DspVoice[] _voices = new DspVoice[8];

        // KON latch - an event register, not a level. See Venus_APU.md §3.3.
        private byte _pendingKon;
        private byte _prevKoff;

        // Named for readability; storage is still the flat _registers[128] array - see Venus_APU.md §3.
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

        // The echo unit, ported from Mesen2's Dsp.cpp and flattened per sample - see Venus_APU.md §3.
        private readonly short[] _echoHistoryL = new short[8];
        private readonly short[] _echoHistoryR = new short[8];
        private int _echoHistoryPos;
        private int _echoOffset;
        private int _echoLength;

        // Pending output, not game state; an explicit field so [SkipInState] can apply.
        [EmuSen.Common.SkipInState]
        private Queue<short> _audioBuffer = new Queue<short>();
        public Queue<short> AudioBuffer => _audioBuffer;

        private int _dspCycles;

        // Debug-only, not hardware state: a muted voice still steps, it just leaves the mix.
        [EmuSen.Common.SkipInState]
        private byte _debugMuteMask;

        public SDsp()
        {
            for (int i = 0; i < 8; i++) _voices[i] = new DspVoice();
        }

        // Called by Spc700's constructor so voices can reach BRR data in SPC RAM.
        public void AttachMemory(byte[] ram)
        {
            _ram = ram;
            foreach (var voice in _voices) voice.AttachMemory(ram);
        }

        public void Reset()
        {
            Array.Clear(_registers, 0, _registers.Length);

            // FLG reads 0xE0 after reset, or echo writes into zero-page RAM before a game configures it.
            _registers[RegFlags] = 0xE0;

            _registerAddress = 0;
            _dspCycles = 0;
            _pendingKon = 0;
            _prevKoff = 0;
            Array.Clear(_echoHistoryL, 0, 8);
            Array.Clear(_echoHistoryR, 0, 8);
            _echoHistoryPos = 0;
            _echoOffset = 0;
            _echoLength = 0;
            foreach (var voice in _voices) voice.Reset();
            AudioBuffer.Clear();
        }

        // The raw byte, unmasked: hardware echoes bit 7 back on a read - caught by SpcValidation.
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

        // Never touches _registerAddress, so inspecting cannot disturb a driver mid-sequence.
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

            if (index == RegKeyOn) _pendingKon |= data; // see Venus_APU.md §3.3.1

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

        // Lets a KeyOn log distinguish a rhythmic pattern from a retrigger burst.
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
                    // The voice already stepped, so only the mix and the echo feed skip it.
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

                // FLG bit 6 silences the DAC only; voices and the echo delay line still run.
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

            // A safety valve when nothing drains; pairs only, or L/R swap - see EmuSen_Audio_Sync.md §4.
            while (AudioBuffer.Count > AudioSettings.AudioBufferMaxSamples && AudioBuffer.Count >= 2)
            {
                AudioBuffer.Dequeue();
                AudioBuffer.Dequeue();
            }
        }

        // The 8-tap FIR echo, fed by the EON-gated voice sum - see Venus_APU.md §3.
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

            // Feedback blends each repeat with a scaled copy of the previous one.
            sbyte efb = (sbyte)_registers[RegEchoFeedback];
            int echoOutL = Math.Clamp(dryEchoInputL + ((firL * efb) >> 7), short.MinValue, short.MaxValue) & ~1;
            int echoOutR = Math.Clamp(dryEchoInputR + ((firR * efb) >> 7), short.MinValue, short.MaxValue) & ~1;

            // FLG bit 5 clear enables echo writes: ECEN is active-low - see Venus_APU.md §3.
            if ((_registers[RegFlags] & 0x20) == 0)
            {
                _ram[pointer] = (byte)(echoOutL & 0xFF);
                _ram[(pointer + 1) & 0xFFFF] = (byte)((echoOutL >> 8) & 0xFF);
                _ram[(pointer + 2) & 0xFFFF] = (byte)(echoOutR & 0xFF);
                _ram[(pointer + 3) & 0xFFFF] = (byte)((echoOutR >> 8) & 0xFF);
            }

            // EDL is re-read only on wrap, so a write takes effect next lap, as on hardware.
            if (_echoOffset == 0)
            {
                _echoLength = (_registers[RegEchoDelay] & 0x0F) << 11;
            }
            _echoOffset += 4;
            if (_echoOffset >= _echoLength) _echoOffset = 0;

            return (firL, firR);
        }

        // Drains the KON latch, edge-detects KOFF - see Venus_APU.md §3.3.
        private void ProcessKeyEvents()
        {
            byte koff = _registers[RegKeyOff];
            byte dir = _registers[RegSourceDir];
            int dirTableAddr = dir << 8;

            // Every latched KON write fires exactly once, then drains.
            byte konPending = _pendingKon;
            _pendingKon = 0;
            byte koffRising = (byte)(koff & ~_prevKoff);

            for (int v = 0; v < 8; v++)
            {
                bool konBit = (konPending & (1 << v)) != 0;
                bool koffBit = (koffRising & (1 << v)) != 0;

                if (konBit) _voices[v].KeyOn(dirTableAddr, _sampleCounter);
                // KeyOff wins if both land in the same window - see Venus_APU.md §3.3.
                if (koffBit) _voices[v].KeyOff();
            }

            _prevKoff = koff;
        }

        // Gathers one voice's state into a struct instead of ten getters for the debug layer.
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

        // So LastKeyOnSample can be read as "how many samples ago" without a second counter.
        public long SampleCounter => _sampleCounter;
    }
}
