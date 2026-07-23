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

        private readonly DspVoice[] _voices = new DspVoice[8];
        private byte _prevKon;
        private byte _prevKoff;

        // Pending output samples, not game state - excluded from save states
        // the same reasoning as Renderer's own pixel buffer. Explicit backing
        // field (not an auto-property) specifically so [SkipInState], which
        // only targets fields, can actually be applied to it.
        [EmuSen.Common.SkipInState]
        private Queue<short> _audioBuffer = new Queue<short>();
        public Queue<short> AudioBuffer => _audioBuffer;

        private int _dspCycles;

        public SDsp()
        {
            for (int i = 0; i < 8; i++) _voices[i] = new DspVoice();
        }

        // Called once by Spc700's constructor so voices can read BRR sample
        // data and the sample directory table directly out of SPC RAM.
        public void AttachMemory(byte[] ram)
        {
            foreach (var voice in _voices) voice.AttachMemory(ram);
        }

        public void Reset()
        {
            Array.Clear(_registers, 0, _registers.Length);
            _registerAddress = 0;
            _dspCycles = 0;
            _prevKon = 0;
            _prevKoff = 0;
            foreach (var voice in _voices) voice.Reset();
            AudioBuffer.Clear();
        }

        public void SetRegisterAddress(byte address)
        {
            _registerAddress = (byte)(address & 0x7F);
        }

        public byte GetRegisterAddress()
        {
            return _registerAddress;
        }

        public byte ReadRegister()
        {
            // ENDX - see Venus_APU.md §3.2.
            if (_registerAddress == 0x7C)
            {
                byte endx = 0;
                for (int v = 0; v < 8; v++)
                {
                    if (_voices[v].Ended) endx |= (byte)(1 << v);
                }
                return endx;
            }
            return _registers[_registerAddress];
        }

        public void WriteRegister(byte data)
        {
            _registers[_registerAddress] = data;

            // Any write to ENDX clears all its bits - see Venus_APU.md §3.2.
            if (_registerAddress == 0x7C)
            {
                foreach (var voice in _voices) ClearEnded(voice);
                return;
            }

            int voiceIdx = _registerAddress >> 4;
            int voiceReg = _registerAddress & 0x0F;
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

        private void GenerateSample()
        {
            ProcessKeyEvents();

            short leftSample = 0;
            short rightSample = 0;

            if (AudioSettings.AudioEnabled && !AudioSettings.Muted)
            {
                int mixL = 0;
                int mixR = 0;

                foreach (var voice in _voices)
                {
                    short s = voice.GetNextSample();
                    mixL += (s * (sbyte)voice.VolL) >> 7;
                    mixR += (s * (sbyte)voice.VolR) >> 7;
                }

                sbyte mvolL = (sbyte)_registers[0x0C];
                sbyte mvolR = (sbyte)_registers[0x1C];
                mixL = (mixL * mvolL) >> 7;
                mixR = (mixR * mvolR) >> 7;

                mixL = (int)(mixL * AudioSettings.MasterVolume);
                mixR = (int)(mixR * AudioSettings.MasterVolume);

                leftSample = (short)Math.Clamp(mixL, short.MinValue, short.MaxValue);
                rightSample = (short)Math.Clamp(mixR, short.MinValue, short.MaxValue);
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

                if (konBit) _voices[v].KeyOn(dirTableAddr);
                // KeyOff wins if both are newly set - see Venus_APU.md §3.3.
                if (koffBit) _voices[v].KeyOff();
            }

            _prevKon = kon;
            _prevKoff = koff;
        }
    }
}
