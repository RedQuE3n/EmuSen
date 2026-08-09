using System;
using System.Collections.Generic;
using EmuSen.Common;

namespace EmuSen.Cores.Nintendo.Mercury.Audio
{
    // The Game Boy's sound hardware: four channels into a two-channel mixer - see Mercury_Apu.md.
    public sealed class Apu
    {
        public const int ChannelCount = 4;

        public const int RegisterBase = 0xFF10;
        public const int RegisterCount = 0x17;

        public const int WaveRamBase = 0xFF30;

        // What a real console's output capacitor does to the DAC's standing offset - see Mercury_Apu.md §5.2.
        public const double HighPassSeed = 0.999958;

        // $FF10-$FF26 as written, for the read path's unreadable-bit masks and for the debug view.
        public readonly byte[] Registers = new byte[RegisterCount];

        public readonly PulseChannel Pulse1 = new(hasSweep: true);
        public readonly PulseChannel Pulse2 = new(hasSweep: false);
        public readonly WaveChannel Wave = new();
        public readonly NoiseChannel Noise = new();

        public bool PoweredOn;

        // NR50's two three-bit master volumes, and NR51's eight panning switches.
        public int LeftVolume;
        public int RightVolume;
        public byte Panning;

        public int SequencerStep;

        // The DIV bit the sequencer watches, seen on its previous tick - see Mercury_Apu.md §2.
        public bool LastSequencerBit;

        [SkipInState] private readonly Queue<short> _buffer = new();

        [SkipInState] public int MaxBufferedSamples = 128000;

        [SkipInState] private readonly bool[] _channelMuted = new bool[ChannelCount];

        [SkipInState] private double _cyclesPerSample = MercuryCore.CpuClockHz / 44100.0;
        [SkipInState] private double _cycleFraction;

        [SkipInState] private double _leftCapacitor;
        [SkipInState] private double _rightCapacitor;
        [SkipInState] private double _chargeFactor = Math.Pow(HighPassSeed, MercuryCore.CpuClockHz / 44100.0);

        public int BufferedSamples => _buffer.Count;

        public void SetSampleRate(int sampleRate)
        {
            _cyclesPerSample = MercuryCore.CpuClockHz / sampleRate;
            _chargeFactor = Math.Pow(HighPassSeed, _cyclesPerSample);
        }

        public bool IsChannelMuted(int index) =>
            (uint)index < ChannelCount && _channelMuted[index];

        public void SetChannelMuted(int index, bool muted)
        {
            if ((uint)index < ChannelCount) _channelMuted[index] = muted;
        }

        public void Reset()
        {
            Array.Clear(Registers);
            Array.Clear(Wave.Ram);

            PowerOff();

            // What the DMG boot ROM leaves behind: powered up, both volumes at 7, everything panned to both.
            PoweredOn = true;
            LeftVolume = 7;
            RightVolume = 7;
            Panning = 0xF3;

            Registers[0x24 - 0x10] = 0x77;
            Registers[0x25 - 0x10] = 0xF3;
            Registers[0x26 - 0x10] = 0xF1;

            SequencerStep = 0;
            LastSequencerBit = false;

            _cycleFraction = 0;
            _leftCapacitor = 0;
            _rightCapacitor = 0;
            _buffer.Clear();
        }

        // One T-cycle at the base clock. Double speed does not reach here - see Mercury_Apu.md §2.1.
        public void Tick()
        {
            if (PoweredOn)
            {
                Pulse1.StepTimer();
                Pulse2.StepTimer();
                Wave.StepTimer();
                Noise.StepTimer();
            }

            _cycleFraction += 1.0;
            if (_cycleFraction < _cyclesPerSample) return;

            _cycleFraction -= _cyclesPerSample;
            EmitSample();
        }

        // Driven by a falling edge of a DIV bit rather than its own divider - see Mercury_Apu.md §2.
        public void OnDivBit(bool bit)
        {
            if (LastSequencerBit && !bit) StepSequencer();
            LastSequencerBit = bit;
        }

        private void StepSequencer()
        {
            if (!PoweredOn) return;

            // Length on the even steps, sweep on two of them, the envelope only on the last.
            switch (SequencerStep)
            {
                case 0 or 4:
                    ClockLengths();
                    break;

                case 2 or 6:
                    ClockLengths();
                    Pulse1.ClockSweep();
                    break;

                case 7:
                    Pulse1.Envelope.Clock();
                    Pulse2.Envelope.Clock();
                    Noise.Envelope.Clock();
                    break;
            }

            SequencerStep = (SequencerStep + 1) & 0x07;
        }

        private void ClockLengths()
        {
            if (Pulse1.Length.Clock()) Pulse1.Enabled = false;
            if (Pulse2.Length.Clock()) Pulse2.Enabled = false;
            if (Wave.Length.Clock()) Wave.Enabled = false;
            if (Noise.Length.Clock()) Noise.Enabled = false;
        }

        public byte ReadRegister(int address)
        {
            if (address >= WaveRamBase) return Wave.Ram[address - WaveRamBase];

            int index = address - RegisterBase;
            if ((uint)index >= RegisterCount) return 0xFF;

            if (address == 0xFF26)
            {
                int status = (PoweredOn ? 0x80 : 0x00)
                    | (Pulse1.Enabled ? 0x01 : 0x00)
                    | (Pulse2.Enabled ? 0x02 : 0x00)
                    | (Wave.Enabled ? 0x04 : 0x00)
                    | (Noise.Enabled ? 0x08 : 0x00);

                return (byte)(status | 0x70);
            }

            return (byte)(Registers[index] | ReadMask(address));
        }

        // Every bit a game cannot read back reads as 1, which is what a detection routine expects.
        private static byte ReadMask(int address) => address switch
        {
            0xFF10 => 0x80,
            0xFF11 or 0xFF16 => 0x3F,
            0xFF13 or 0xFF18 or 0xFF1B or 0xFF1D or 0xFF20 => 0xFF,
            0xFF14 or 0xFF19 or 0xFF1E or 0xFF23 => 0xBF,
            0xFF1A => 0x7F,
            0xFF1C => 0x9F,
            0xFF15 or 0xFF1F => 0xFF,
            _ => 0x00,
        };

        public void WriteRegister(int address, byte value)
        {
            if (address >= WaveRamBase)
            {
                Wave.Ram[address - WaveRamBase] = value;
                return;
            }

            int index = address - RegisterBase;
            if ((uint)index >= RegisterCount) return;

            // With the power off every register but NR52 ignores its write - see Mercury_Apu.md §4.
            if (!PoweredOn && address != 0xFF26) return;

            Registers[index] = value;

            switch (address)
            {
                case 0xFF10: WriteSweep(value); return;
                case 0xFF11: WritePulseDutyLength(Pulse1, value); return;
                case 0xFF12: WriteEnvelope(Pulse1, value); return;
                case 0xFF13: Pulse1.Frequency = (Pulse1.Frequency & 0x0700) | value; return;
                case 0xFF14: WritePulseControl(Pulse1, value); return;

                case 0xFF16: WritePulseDutyLength(Pulse2, value); return;
                case 0xFF17: WriteEnvelope(Pulse2, value); return;
                case 0xFF18: Pulse2.Frequency = (Pulse2.Frequency & 0x0700) | value; return;
                case 0xFF19: WritePulseControl(Pulse2, value); return;

                case 0xFF1A: WriteWaveDac(value); return;
                case 0xFF1B: Wave.Length.Load(value); return;
                case 0xFF1C: Wave.VolumeCode = (value >> 5) & 0x03; return;
                case 0xFF1D: Wave.Frequency = (Wave.Frequency & 0x0700) | value; return;
                case 0xFF1E: WriteWaveControl(value); return;

                case 0xFF20: Noise.Length.Load(value & 0x3F); return;
                case 0xFF21: WriteNoiseEnvelope(value); return;
                case 0xFF22: WriteNoisePeriod(value); return;
                case 0xFF23: WriteNoiseControl(value); return;

                case 0xFF24:
                    LeftVolume = (value >> 4) & 0x07;
                    RightVolume = value & 0x07;
                    return;

                case 0xFF25:
                    Panning = value;
                    return;

                case 0xFF26:
                    WritePower(value);
                    return;
            }
        }

        private void WriteSweep(byte value)
        {
            Pulse1.SweepPeriod = (value >> 4) & 0x07;
            Pulse1.SweepNegate = (value & 0x08) != 0;
            Pulse1.SweepShift = value & 0x07;
        }

        private static void WritePulseDutyLength(PulseChannel pulse, byte value)
        {
            pulse.Duty = value >> 6;
            pulse.Length.Load(value & 0x3F);
        }

        // Clearing the top five bits of NRx2 kills the DAC, and a dead DAC disables the channel.
        private static void WriteEnvelope(PulseChannel pulse, byte value)
        {
            pulse.Envelope.Load(value);
            pulse.DacEnabled = (value & 0xF8) != 0;
            if (!pulse.DacEnabled) pulse.Enabled = false;
        }

        private static void WritePulseControl(PulseChannel pulse, byte value)
        {
            pulse.Frequency = (pulse.Frequency & 0x00FF) | ((value & 0x07) << 8);
            pulse.Length.Enabled = (value & 0x40) != 0;
            if ((value & 0x80) != 0) pulse.Trigger();
        }

        private void WriteWaveDac(byte value)
        {
            Wave.DacEnabled = (value & 0x80) != 0;
            if (!Wave.DacEnabled) Wave.Enabled = false;
        }

        private void WriteWaveControl(byte value)
        {
            Wave.Frequency = (Wave.Frequency & 0x00FF) | ((value & 0x07) << 8);
            Wave.Length.Enabled = (value & 0x40) != 0;
            if ((value & 0x80) != 0) Wave.Trigger();
        }

        private void WriteNoiseEnvelope(byte value)
        {
            Noise.Envelope.Load(value);
            Noise.DacEnabled = (value & 0xF8) != 0;
            if (!Noise.DacEnabled) Noise.Enabled = false;
        }

        private void WriteNoisePeriod(byte value)
        {
            Noise.ClockShift = value >> 4;
            Noise.ShortMode = (value & 0x08) != 0;
            Noise.DivisorCode = value & 0x07;
        }

        private void WriteNoiseControl(byte value)
        {
            Noise.Length.Enabled = (value & 0x40) != 0;
            if ((value & 0x80) != 0) Noise.Trigger();
        }

        private void WritePower(byte value)
        {
            bool on = (value & 0x80) != 0;
            if (on == PoweredOn) return;

            if (on)
            {
                PoweredOn = true;

                // A power-up restarts the sequencer, so the first length clock is a known distance away.
                SequencerStep = 0;
                return;
            }

            PowerOff();
        }

        // Wave RAM survives a power cycle; nothing else does - see Mercury_Apu.md §4.
        private void PowerOff()
        {
            PoweredOn = false;

            Array.Clear(Registers);
            LeftVolume = 0;
            RightVolume = 0;
            Panning = 0;

            ResetChannel(Pulse1);
            ResetChannel(Pulse2);

            Wave.Enabled = false;
            Wave.DacEnabled = false;
            Wave.Frequency = 0;
            Wave.VolumeCode = 0;
            Wave.Length.Enabled = false;

            Noise.Enabled = false;
            Noise.DacEnabled = false;
            Noise.Length.Enabled = false;
            Noise.ClockShift = 0;
            Noise.DivisorCode = 0;
            Noise.ShortMode = false;
        }

        private static void ResetChannel(PulseChannel pulse)
        {
            pulse.Enabled = false;
            pulse.DacEnabled = false;
            pulse.Frequency = 0;
            pulse.Duty = 0;
            pulse.Length.Enabled = false;
            pulse.SweepRunning = false;
        }

        private void EmitSample()
        {
            double left = 0;
            double right = 0;

            for (int channel = 0; channel < ChannelCount; channel++)
            {
                double value = DacOutput(channel);

                if ((Panning & (0x10 << channel)) != 0) left += value;
                if ((Panning & (0x01 << channel)) != 0) right += value;
            }

            left = left / ChannelCount * ((LeftVolume + 1) / 8.0);
            right = right / ChannelCount * ((RightVolume + 1) / 8.0);

            short leftSample = ToSample(HighPass(left, ref _leftCapacitor));
            short rightSample = ToSample(HighPass(right, ref _rightCapacitor));

            // Oldest pair goes first, so a frontend that stops draining loses history, not the present.
            if (_buffer.Count + 2 > MaxBufferedSamples)
            {
                _buffer.Dequeue();
                _buffer.Dequeue();
            }

            _buffer.Enqueue(leftSample);
            _buffer.Enqueue(rightSample);
        }

        // A silent channel with a live DAC still sits at the bottom of the swing, which is not zero - see Mercury_Apu.md §5.1.
        private double DacOutput(int channel)
        {
            if (!PoweredOn || _channelMuted[channel]) return 0.0;

            var (dacEnabled, output) = channel switch
            {
                0 => (Pulse1.DacEnabled, Pulse1.Output),
                1 => (Pulse2.DacEnabled, Pulse2.Output),
                2 => (Wave.DacEnabled, Wave.Output),
                _ => (Noise.DacEnabled, Noise.Output),
            };

            return dacEnabled ? (output / 7.5) - 1.0 : 0.0;
        }

        private double HighPass(double input, ref double capacitor)
        {
            double output = input - capacitor;
            capacitor = input - (output * _chargeFactor);
            return output;
        }

        private static short ToSample(double value) =>
            (short)Math.Clamp(value * short.MaxValue, short.MinValue, short.MaxValue);

        // Non-destructive snapshot for `audiodump`; the buffer keeps everything.
        public short[] Peek() => _buffer.ToArray();

        // Destructive drain, matching ICore.DequeueAudioSamples - see EmuSen_Audio_Sync.md §7.
        public short[] Drain(int maxFrames)
        {
            long wantedLong = Math.Min((long)maxFrames * 2, _buffer.Count);
            int wanted = (int)Math.Max(0, wantedLong);
            wanted -= wanted & 1;
            if (wanted <= 0) return Array.Empty<short>();

            var samples = new short[wanted];
            for (int i = 0; i < wanted; i++) samples[i] = _buffer.Dequeue();
            return samples;
        }
    }
}
