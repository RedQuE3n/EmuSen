using System;
using System.Collections.Generic;
using EmuSen.Common;

namespace EmuSen.Cores.Nintendo.Moon.Apu
{
    // The 2A03 sound hardware: five channels, the frame sequencer, and the non-linear mixer - see Moon_APU.md.
    public sealed class Apu
    {
        public const int ChannelCount = 5;

        // NTSC CPU clock, which every timer below is expressed against - see Moon_APU.md §1.
        public const double CpuClockHz = 1789773.0;

        // Chosen so the loudest game measured peaks near -2 dBFS - see Moon_APU.md §4.1.
        private const double OutputGain = 80000.0;

        // $4000-$4013, kept verbatim so the debug target can report what a game wrote.
        public readonly byte[] Registers = new byte[0x14];

        public byte FrameCounter;
        public bool FrameIrqPending;

        public readonly PulseChannel Pulse1 = new(onesComplement: true);
        public readonly PulseChannel Pulse2 = new(onesComplement: false);
        public readonly TriangleChannel Triangle = new();
        public readonly NoiseChannel Noise = new();
        public readonly DmcChannel Dmc = new();

        // Sequencer position in CPU cycles, not steps - see Moon_APU.md §2.
        private int _frameCycle;
        private int _frameStep;

        // The pulse and noise timers run at half the CPU rate, so they tick on alternate cycles.
        private bool _apuCycle;

        // Box-filter accumulator: every output sample is the mean of the cycles it covers - see Moon_APU.md §4.
        [SkipInState] private double _sampleAccumulator;
        [SkipInState] private int _sampleCount;
        [SkipInState] private double _cyclesPerSample = CpuClockHz / 44100.0;
        [SkipInState] private double _cycleFraction;

        // Drained by MoonCore.DequeueAudioSamples; never part of a save state.
        [SkipInState] private readonly Queue<short> _buffer = new();

        [SkipInState] public int MaxBufferedSamples = 128000;

        // The console's own output filters; without them the unipolar DAC leaves a DC step - see Moon_APU.md §4.1.
        [SkipInState] private double _hp90;
        [SkipInState] private double _hp90Prev;
        [SkipInState] private double _hp440;
        [SkipInState] private double _hp440Prev;
        [SkipInState] private double _lp14k;
        [SkipInState] private double _highPass90Alpha = 1.0;
        [SkipInState] private double _highPass440Alpha = 1.0;
        [SkipInState] private double _lowPassAlpha;

        public int BufferedSamples => _buffer.Count;

        public bool IrqInhibited => (FrameCounter & 0x40) != 0;
        public bool FiveStepMode => (FrameCounter & 0x80) != 0;

        // Everything the CPU should see as one IRQ line.
        public bool IrqAsserted => FrameIrqPending || Dmc.IrqPending;

        public void SetSampleRate(int sampleRate)
        {
            int rate = Math.Max(1, sampleRate);
            _cyclesPerSample = CpuClockHz / rate;

            double dt = 1.0 / rate;
            _highPass90Alpha = HighPassAlpha(90.0, dt);
            _highPass440Alpha = HighPassAlpha(440.0, dt);
            _lowPassAlpha = LowPassAlpha(14000.0, dt);
        }

        private static double HighPassAlpha(double hz, double dt)
        {
            double rc = 1.0 / (2.0 * Math.PI * hz);
            return rc / (rc + dt);
        }

        private static double LowPassAlpha(double hz, double dt)
        {
            double rc = 1.0 / (2.0 * Math.PI * hz);
            return dt / (rc + dt);
        }

        // Two high-passes then a low-pass, matching the console's own output stage.
        private double Filter(double sample)
        {
            _hp90 = _highPass90Alpha * (_hp90 + sample - _hp90Prev);
            _hp90Prev = sample;

            _hp440 = _highPass440Alpha * (_hp440 + _hp90 - _hp440Prev);
            _hp440Prev = _hp90;

            _lp14k += _lowPassAlpha * (_hp440 - _lp14k);
            return _lp14k;
        }

        // $4015's bit order, for the debug target.
        public int[] LengthCounters => new[]
        {
            Pulse1.LengthCounter, Pulse2.LengthCounter, Triangle.LengthCounter,
            Noise.LengthCounter, Dmc.Active ? 1 : 0,
        };

        public void Reset()
        {
            Array.Clear(Registers);
            FrameCounter = 0;
            FrameIrqPending = false;
            _frameCycle = 0;
            _frameStep = 0;
            _apuCycle = false;
            Dmc.Reset();
            _buffer.Clear();
        }

        public void WriteRegister(int address, byte value)
        {
            if (address >= 0x4000 && address <= 0x4013) Registers[address - 0x4000] = value;

            switch (address)
            {
                case 0x4000: WritePulseControl(Pulse1, value); break;
                case 0x4001: WriteSweep(Pulse1, value); break;
                case 0x4002: Pulse1.TimerPeriod = (Pulse1.TimerPeriod & 0x700) | value; break;
                case 0x4003: WritePulseHigh(Pulse1, value); break;

                case 0x4004: WritePulseControl(Pulse2, value); break;
                case 0x4005: WriteSweep(Pulse2, value); break;
                case 0x4006: Pulse2.TimerPeriod = (Pulse2.TimerPeriod & 0x700) | value; break;
                case 0x4007: WritePulseHigh(Pulse2, value); break;

                case 0x4008:
                    Triangle.ControlFlag = (value & 0x80) != 0;
                    Triangle.LinearReloadValue = value & 0x7F;
                    break;
                case 0x400A: Triangle.TimerPeriod = (Triangle.TimerPeriod & 0x700) | value; break;
                case 0x400B:
                    Triangle.TimerPeriod = (Triangle.TimerPeriod & 0xFF) | ((value & 0x07) << 8);
                    if (Triangle.Enabled) Triangle.LengthCounter = ApuTables.LengthCounter[value >> 3];
                    Triangle.ReloadLinear();
                    break;

                case 0x400C:
                    Noise.LengthHalted = (value & 0x20) != 0;
                    Noise.Envelope.Loop = Noise.LengthHalted;
                    Noise.Envelope.ConstantVolume = (value & 0x10) != 0;
                    Noise.Envelope.Volume = value & 0x0F;
                    break;
                case 0x400E:
                    Noise.ShortMode = (value & 0x80) != 0;
                    Noise.PeriodIndex = value & 0x0F;
                    break;
                case 0x400F:
                    if (Noise.Enabled) Noise.LengthCounter = ApuTables.LengthCounter[value >> 3];
                    Noise.Envelope.Restart();
                    break;

                case 0x4010:
                    Dmc.IrqEnabled = (value & 0x80) != 0;
                    Dmc.Loop = (value & 0x40) != 0;
                    Dmc.RateIndex = value & 0x0F;
                    if (!Dmc.IrqEnabled) Dmc.IrqPending = false;
                    break;
                case 0x4011: Dmc.OutputLevel = value & 0x7F; break;
                case 0x4012: Dmc.SampleAddress = 0xC000 + (value * 64); break;
                case 0x4013: Dmc.SampleLength = (value * 16) + 1; break;

                case 0x4015: WriteEnable(value); break;

                case 0x4017:
                    FrameCounter = value;
                    _frameCycle = 0;
                    _frameStep = 0;
                    if (IrqInhibited) FrameIrqPending = false;

                    // Setting the five-step bit clocks everything once immediately.
                    if (FiveStepMode) { ClockQuarterFrame(); ClockHalfFrame(); }
                    break;
            }
        }

        private static void WritePulseControl(PulseChannel pulse, byte value)
        {
            pulse.Duty = (value >> 6) & 0x03;
            pulse.LengthHalted = (value & 0x20) != 0;
            pulse.Envelope.Loop = pulse.LengthHalted;
            pulse.Envelope.ConstantVolume = (value & 0x10) != 0;
            pulse.Envelope.Volume = value & 0x0F;
        }

        private static void WriteSweep(PulseChannel pulse, byte value)
        {
            pulse.SweepEnabled = (value & 0x80) != 0;
            pulse.SweepPeriod = (value >> 4) & 0x07;
            pulse.SweepNegate = (value & 0x08) != 0;
            pulse.SweepShift = value & 0x07;
            pulse.ReloadSweep();
        }

        private static void WritePulseHigh(PulseChannel pulse, byte value)
        {
            pulse.TimerPeriod = (pulse.TimerPeriod & 0xFF) | ((value & 0x07) << 8);
            if (pulse.Enabled) pulse.LengthCounter = ApuTables.LengthCounter[value >> 3];
            pulse.RestartSequencer();
        }

        private void WriteEnable(byte value)
        {
            Pulse1.Enabled = (value & 0x01) != 0;
            Pulse2.Enabled = (value & 0x02) != 0;
            Triangle.Enabled = (value & 0x04) != 0;
            Noise.Enabled = (value & 0x08) != 0;
            Dmc.Enabled = (value & 0x10) != 0;
        }

        // Reading the status register acknowledges the frame IRQ - see Moon_APU.md §2.
        public byte ReadStatus()
        {
            byte value = 0;
            if (Pulse1.LengthCounter > 0) value |= 0x01;
            if (Pulse2.LengthCounter > 0) value |= 0x02;
            if (Triangle.LengthCounter > 0) value |= 0x04;
            if (Noise.LengthCounter > 0) value |= 0x08;
            if (Dmc.Active) value |= 0x10;
            if (FrameIrqPending) value |= 0x40;
            if (Dmc.IrqPending) value |= 0x80;

            FrameIrqPending = false;
            return value;
        }

        // Advances every timer by <cpuCycles> and emits however many output samples that covers.
        public void Step(int cpuCycles)
        {
            for (int i = 0; i < cpuCycles; i++)
            {
                Triangle.StepTimer();
                Dmc.StepTimer();

                _apuCycle = !_apuCycle;
                if (_apuCycle)
                {
                    Pulse1.StepTimer();
                    Pulse2.StepTimer();
                    Noise.StepTimer();
                }

                StepFrameCounter();

                _sampleAccumulator += Mix();
                _sampleCount++;

                _cycleFraction += 1.0;
                if (_cycleFraction < _cyclesPerSample) continue;

                _cycleFraction -= _cyclesPerSample;
                EmitSample();
            }
        }

        private void EmitSample()
        {
            double mean = _sampleCount > 0 ? _sampleAccumulator / _sampleCount : 0.0;
            _sampleAccumulator = 0;
            _sampleCount = 0;

            short sample = (short)Math.Clamp(Filter(mean) * OutputGain, short.MinValue, short.MaxValue);

            // Oldest pair goes first, so a frontend that stops draining loses history, not the present.
            if (_buffer.Count + 2 > MaxBufferedSamples)
            {
                _buffer.Dequeue();
                _buffer.Dequeue();
            }

            // The 2A03 is mono, so both output channels carry the same value.
            _buffer.Enqueue(sample);
            _buffer.Enqueue(sample);
        }

        // The two halves of the DAC are non-linear and summed separately - see Moon_APU.md §4.
        private double Mix()
        {
            int pulseSum = Pulse1.Output + Pulse2.Output;
            double pulseOut = pulseSum == 0 ? 0.0 : 95.88 / ((8128.0 / pulseSum) + 100.0);

            double tnd = (Triangle.Output / 8227.0) + (Noise.Output / 12241.0) + (Dmc.Output / 22638.0);
            double tndOut = tnd == 0.0 ? 0.0 : 159.79 / ((1.0 / tnd) + 100.0);

            return pulseOut + tndOut;
        }

        private static readonly int[] FourStepCycles = { 7457, 14913, 22371, 29829 };
        private static readonly int[] FiveStepCycles = { 7457, 14913, 22371, 29829, 37281 };

        // Four or five steps across one sequencer period - see Moon_APU.md §2.
        private void StepFrameCounter()
        {
            _frameCycle++;

            int[] steps = FiveStepMode ? FiveStepCycles : FourStepCycles;
            if (_frameStep >= steps.Length || _frameCycle < steps[_frameStep]) return;

            bool half = FiveStepMode ? _frameStep is 1 or 4 : _frameStep is 1 or 3;
            bool quarter = !FiveStepMode || _frameStep != 3;

            if (quarter) ClockQuarterFrame();
            if (half) ClockHalfFrame();

            if (!FiveStepMode && _frameStep == 3 && !IrqInhibited) FrameIrqPending = true;

            _frameStep++;
            if (_frameStep < steps.Length) return;

            _frameStep = 0;
            _frameCycle = 0;
        }

        private void ClockQuarterFrame()
        {
            Pulse1.Envelope.Clock();
            Pulse2.Envelope.Clock();
            Noise.Envelope.Clock();
            Triangle.ClockLinear();
        }

        private void ClockHalfFrame()
        {
            Pulse1.ClockLength();
            Pulse2.ClockLength();
            Triangle.ClockLength();
            Noise.ClockLength();
            Pulse1.ClockSweep();
            Pulse2.ClockSweep();
        }

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
