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

        // A $4017 write restarts the sequencer 3-4 cycles later, not at once - see Moon_APU.md §2.2.
        private int _writeDelayCounter;
        private int _pendingFrameValue = -1;

        // The mode only takes effect once that delay expires, unlike the inhibit bit.
        private bool _stepMode;

        // One tick blocks the next two cycles from producing another - see Moon_APU.md §2.2.
        private int _blockFrameCounterTick;

        // Parity decides the write delay, so the sequencer needs its own cycle count.
        private long _cycleCount;

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
        public bool FiveStepMode => _stepMode;

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

        // A debugging aid, not hardware, so it is never part of a save state - see Moon_APU.md §5.
        [SkipInState] private readonly bool[] _channelMuted = new bool[ChannelCount];

        public bool IsChannelMuted(int index) =>
            (uint)index < (uint)_channelMuted.Length && _channelMuted[index];

        public void SetChannelMuted(int index, bool muted)
        {
            if ((uint)index < (uint)_channelMuted.Length) _channelMuted[index] = muted;
        }

        public void Reset()
        {
            Array.Clear(Registers);

            // Only a cold start writes $4017 with $00; RESET rewrites whatever was there - see Moon_APU.md §2.2.
            FrameCounter = 0;
            _stepMode = false;
            _cycleCount = 0;
            SoftReset();
            Dmc.Reset();
            _buffer.Clear();
        }

        // RESET silences every channel and rewrites $4017 with the mode it already had - see Moon_APU.md §2.2.
        public void SoftReset()
        {
            WriteEnable(0);
            FrameIrqPending = false;
            _frameCycle = 0;
            _frameStep = 0;
            _apuCycle = false;
            _blockFrameCounterTick = 0;
            _pendingFrameValue = _stepMode ? 0x80 : 0x00;
            _writeDelayCounter = 3;
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
                    if (IrqInhibited) FrameIrqPending = false;

                    // A write between two APU cycles takes 4 CPU cycles to land, one during them 3.
                    _pendingFrameValue = value;
                    _writeDelayCounter = (_cycleCount & 1) != 0 ? 4 : 3;
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
                _cycleCount++;
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
            // Muting drops a channel's contribution to the DAC, which is not the same as disabling it.
            int pulseSum = (_channelMuted[0] ? 0 : Pulse1.Output) + (_channelMuted[1] ? 0 : Pulse2.Output);
            double pulseOut = pulseSum == 0 ? 0.0 : 95.88 / ((8128.0 / pulseSum) + 100.0);

            double tnd = ((_channelMuted[2] ? 0 : Triangle.Output) / 8227.0)
                + ((_channelMuted[3] ? 0 : Noise.Output) / 12241.0)
                + ((_channelMuted[4] ? 0 : Dmc.Output) / 22638.0);

            double tndOut = tnd == 0.0 ? 0.0 : 159.79 / ((1.0 / tnd) + 100.0);

            return pulseOut + tndOut;
        }

        // Six entries, not four: the sequence runs two cycles past its last clock - see Moon_APU.md §2.1.
        private const int FrameNone = 0;
        private const int FrameQuarter = 1;
        private const int FrameHalf = 2;
        private const int StepCount = 6;

        private static readonly int[] FourStepCycles = { 7457, 14913, 22371, 29828, 29829, 29830 };
        private static readonly int[] FiveStepCycles = { 7457, 14913, 22371, 29829, 37281, 37282 };
        private static readonly int[] FrameTypes = { FrameQuarter, FrameHalf, FrameQuarter, FrameNone, FrameHalf, FrameNone };

        // Modelled on Mesen's ApuFrameCounter, which is the reference for every edge case here - see §2.1.
        private void StepFrameCounter()
        {
            _frameCycle++;

            int[] steps = _stepMode ? FiveStepCycles : FourStepCycles;

            if (_frameCycle >= steps[_frameStep])
            {
                // Four-step mode holds the IRQ across all three of the sequence's last cycles.
                if (!_stepMode && _frameStep >= 3 && !IrqInhibited) FrameIrqPending = true;

                int type = FrameTypes[_frameStep];
                if (type != FrameNone && _blockFrameCounterTick == 0)
                {
                    ClockQuarterFrame();
                    if (type == FrameHalf) ClockHalfFrame();
                    _blockFrameCounterTick = 2;
                }

                _frameStep++;
                if (_frameStep == StepCount)
                {
                    _frameStep = 0;
                    _frameCycle = 0;
                }
            }

            if (_pendingFrameValue >= 0 && --_writeDelayCounter == 0)
            {
                _stepMode = (_pendingFrameValue & 0x80) != 0;
                _pendingFrameValue = -1;
                _frameStep = 0;
                _frameCycle = 0;

                // Selecting five-step mode clocks both units once, immediately.
                if (_stepMode && _blockFrameCounterTick == 0)
                {
                    ClockQuarterFrame();
                    ClockHalfFrame();
                    _blockFrameCounterTick = 2;
                }
            }

            if (_blockFrameCounterTick > 0) _blockFrameCounterTick--;
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
