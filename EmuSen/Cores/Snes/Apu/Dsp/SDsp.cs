using System;
using System.Collections.Generic;
using EmuSen.Audio;

namespace EmuSen.Apu
{
    // The S-DSP: register file, the 8 voices (DspVoice.cs), and final stereo
    // mixing. BRR decoding and per-voice envelope logic live in DspVoice -
    // this class owns register decode (turning raw register bytes into the
    // fields DspVoice actually uses), KON/KOFF edge detection, ENDX, and the
    // MVOL-scaled mix down to the output buffer.
    //
    // Not implemented (see DspVoice's header comment for the full list):
    // echo, noise, pitch modulation, Gaussian interpolation.
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
            // ENDX ($7C) reads the live per-voice end flags; everything else
            // just reads back whatever was last written.
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

            // Any write to ENDX clears all its bits, regardless of the value
            // written - documented hardware behavior, not a typo.
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
                    // 0x8 (ENVX) and 0x9 (OUTX) are read-only from the game's
                    // perspective on real hardware; writes are accepted (it's
                    // just RAM) but ignored here since nothing reads them back
                    // for voice logic.
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
                // Still advance voice playback state even when muted/disabled,
                // so audio doesn't "jump ahead" the moment it's re-enabled.
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

        // KON/KOFF are edge-triggered here (fires on a bit newly set since
        // last sample) rather than level-triggered, so a game holding a KON
        // bit set across multiple writes doesn't re-key the voice every
        // sample. Real hardware processes these every 2nd internal pass
        // (~every 64 SPC clocks per fullsnes); gating on the same per-sample
        // cadence GenerateSample already runs at is close enough for this
        // pass without modeling that separately.
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
                // If both KON and KOFF are newly set for the same voice in
                // the same sample, KeyOff wins (matches documented hardware
                // behavior: key-on immediately followed by key-off silences
                // the channel).
                if (koffBit) _voices[v].KeyOff();
            }

            _prevKon = kon;
            _prevKoff = koff;
        }
    }
}
