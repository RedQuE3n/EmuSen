namespace EmuSen.Cores.Nintendo.Moon.Apu
{
    // Volume envelope shared by both pulses and the noise channel - see Moon_APU.md §3.1.
    public sealed class Envelope
    {
        public bool Loop;
        public bool ConstantVolume;
        public int Volume;

        private bool _start;
        private int _divider;
        private int _decay;

        public void Restart() => _start = true;

        public int Output => ConstantVolume ? Volume : _decay;

        // Clocked on every quarter frame.
        public void Clock()
        {
            if (_start)
            {
                _start = false;
                _decay = 15;
                _divider = Volume;
                return;
            }

            if (_divider > 0)
            {
                _divider--;
                return;
            }

            _divider = Volume;
            if (_decay > 0) _decay--;
            else if (Loop) _decay = 15;
        }
    }

    // One of the two square channels, with its own sweep unit - see Moon_APU.md §3.2.
    public sealed class PulseChannel
    {
        public readonly Envelope Envelope = new();

        public int Duty;
        public bool LengthHalted;
        public int LengthCounter;
        public int TimerPeriod;

        public bool SweepEnabled;
        public int SweepPeriod;
        public bool SweepNegate;
        public int SweepShift;

        // Pulse 1 negates with an extra -1, which is why the two channels are not interchangeable.
        private readonly bool _onesComplement;

        private bool _sweepReload;
        private int _sweepDivider;
        private int _timer;
        private int _sequenceStep;
        private bool _enabled;

        public PulseChannel(bool onesComplement) => _onesComplement = onesComplement;

        public bool Enabled
        {
            get => _enabled;
            set
            {
                _enabled = value;
                if (!value) LengthCounter = 0;
            }
        }

        public void RestartSequencer()
        {
            _sequenceStep = 0;
            Envelope.Restart();
        }

        public void ReloadSweep() => _sweepReload = true;

        // A period under 8, or a sweep target past 11 bits, silences the channel outright.
        private int SweepTarget
        {
            get
            {
                int change = TimerPeriod >> SweepShift;
                if (!SweepNegate) return TimerPeriod + change;
                return TimerPeriod - change - (_onesComplement ? 1 : 0);
            }
        }

        public bool Muted => TimerPeriod < 8 || SweepTarget > 0x7FF;

        public int Output => LengthCounter == 0 || Muted
            ? 0
            : ApuTables.PulseDuty[Duty][_sequenceStep] * Envelope.Output;

        public void StepTimer()
        {
            if (_timer > 0)
            {
                _timer--;
                return;
            }

            _timer = TimerPeriod;
            _sequenceStep = (_sequenceStep + 1) & 0x07;
        }

        public void ClockLength()
        {
            if (!LengthHalted && LengthCounter > 0) LengthCounter--;
        }

        public void ClockSweep()
        {
            if (_sweepDivider == 0 && SweepEnabled && SweepShift > 0 && !Muted)
            {
                TimerPeriod = SweepTarget;
            }

            if (_sweepDivider == 0 || _sweepReload)
            {
                _sweepDivider = SweepPeriod;
                _sweepReload = false;
            }
            else
            {
                _sweepDivider--;
            }
        }
    }

    // The only channel with no volume control at all - see Moon_APU.md §3.3.
    public sealed class TriangleChannel
    {
        public bool ControlFlag;
        public int LinearReloadValue;
        public int LengthCounter;
        public int TimerPeriod;

        private bool _linearReload;
        private int _linearCounter;
        private int _timer;
        private int _sequenceStep;
        private bool _enabled;

        public bool Enabled
        {
            get => _enabled;
            set
            {
                _enabled = value;
                if (!value) LengthCounter = 0;
            }
        }

        public void ReloadLinear() => _linearReload = true;

        // Silenced by period rather than by output, or it would emit a DC pop instead of nothing.
        public int Output => TimerPeriod < 2 ? 0 : ApuTables.TriangleSequence[_sequenceStep];

        // Clocked at the full CPU rate, unlike every other channel.
        public void StepTimer()
        {
            if (LengthCounter == 0 || _linearCounter == 0) return;

            if (_timer > 0)
            {
                _timer--;
                return;
            }

            _timer = TimerPeriod;
            _sequenceStep = (_sequenceStep + 1) & 0x1F;
        }

        public void ClockLinear()
        {
            if (_linearReload) _linearCounter = LinearReloadValue;
            else if (_linearCounter > 0) _linearCounter--;

            if (!ControlFlag) _linearReload = false;
        }

        public void ClockLength()
        {
            if (!ControlFlag && LengthCounter > 0) LengthCounter--;
        }
    }

    // A 15-bit LFSR; the mode bit changes which tap it feeds back from - see Moon_APU.md §3.4.
    public sealed class NoiseChannel
    {
        public readonly Envelope Envelope = new();

        public bool LengthHalted;
        public int LengthCounter;
        public bool ShortMode;
        public int PeriodIndex;

        private int _shift = 1;
        private int _timer;
        private bool _enabled;

        public bool Enabled
        {
            get => _enabled;
            set
            {
                _enabled = value;
                if (!value) LengthCounter = 0;
            }
        }

        public int Output => LengthCounter == 0 || (_shift & 0x01) != 0 ? 0 : Envelope.Output;

        public void StepTimer()
        {
            if (_timer > 0)
            {
                _timer--;
                return;
            }

            _timer = ApuTables.NoisePeriod[PeriodIndex];

            int tap = ShortMode ? (_shift >> 6) & 0x01 : (_shift >> 1) & 0x01;
            int feedback = (_shift & 0x01) ^ tap;
            _shift = (_shift >> 1) | (feedback << 14);
        }

        public void ClockLength()
        {
            if (!LengthHalted && LengthCounter > 0) LengthCounter--;
        }
    }
}
