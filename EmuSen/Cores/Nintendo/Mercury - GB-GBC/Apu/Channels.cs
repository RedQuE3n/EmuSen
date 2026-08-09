namespace EmuSen.Cores.Nintendo.Mercury.Audio
{
    // The volume ramp three of the four channels share - see Mercury_Apu.md §3.1.
    public sealed class Envelope
    {
        public int InitialVolume;
        public bool Increasing;
        public int Period;

        public int Volume;
        public int Timer;
        public bool Finished;

        public void Load(byte nrx2)
        {
            InitialVolume = nrx2 >> 4;
            Increasing = (nrx2 & 0x08) != 0;
            Period = nrx2 & 0x07;
        }

        public void Trigger()
        {
            Volume = InitialVolume;
            Timer = Period == 0 ? 8 : Period;
            Finished = false;
        }

        // A period of zero is off, not "step every tick" - see Mercury_Apu.md §3.1.
        public void Clock()
        {
            if (Period == 0 || Finished) return;
            if (--Timer > 0) return;

            Timer = Period;
            int next = Volume + (Increasing ? 1 : -1);

            if (next is < 0 or > 15)
            {
                Finished = true;
                return;
            }

            Volume = next;
        }
    }

    // Counts down to silence at 256 Hz; 64 steps on three channels, 256 on the wave - see Mercury_Apu.md §3.2.
    public sealed class LengthCounter
    {
        public int Maximum;
        public int Counter;
        public bool Enabled;

        public LengthCounter(int maximum) => Maximum = maximum;

        public void Load(int value) => Counter = Maximum - value;

        // A trigger with the counter already expired reloads it full, which is how a game retriggers.
        public void Trigger()
        {
            if (Counter == 0) Counter = Maximum;
        }

        // True on the tick it reaches zero, which is the tick the channel goes quiet.
        public bool Clock()
        {
            if (!Enabled || Counter == 0) return false;
            return --Counter == 0;
        }
    }

    // Channels 1 and 2; only channel 1 has the sweep unit - see Mercury_Apu.md §3.3.
    public sealed class PulseChannel
    {
        // 12.5%, 25%, 50% and 75% duty, as the eight-step patterns hardware walks.
        public static readonly byte[][] DutyPatterns =
        {
            new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 },
            new byte[] { 1, 0, 0, 0, 0, 0, 0, 1 },
            new byte[] { 1, 0, 0, 0, 0, 1, 1, 1 },
            new byte[] { 0, 1, 1, 1, 1, 1, 1, 0 },
        };

        public readonly bool HasSweep;

        public readonly Envelope Envelope = new();
        public readonly LengthCounter Length = new(64);

        public int Duty;
        public int Frequency;
        public int Timer;
        public int Step;

        public bool Enabled;
        public bool DacEnabled;

        public int SweepPeriod;
        public int SweepShift;
        public bool SweepNegate;

        public int SweepTimer;
        public int SweepShadow;
        public bool SweepRunning;

        public PulseChannel(bool hasSweep) => HasSweep = hasSweep;

        public int Output =>
            Enabled && DacEnabled && DutyPatterns[Duty][Step] != 0 ? Envelope.Volume : 0;

        public void StepTimer()
        {
            if (--Timer > 0) return;

            Timer = (2048 - Frequency) * 4;
            Step = (Step + 1) & 0x07;
        }

        public void Trigger()
        {
            Enabled = DacEnabled;
            Timer = (2048 - Frequency) * 4;

            Envelope.Trigger();
            Length.Trigger();

            if (!HasSweep) return;

            SweepShadow = Frequency;
            SweepTimer = SweepPeriod == 0 ? 8 : SweepPeriod;
            SweepRunning = SweepPeriod > 0 || SweepShift > 0;

            // The overflow check runs at trigger time even though no sweep step has happened yet.
            if (SweepShift > 0) ComputeSweep(apply: false);
        }

        public void ClockSweep()
        {
            if (!HasSweep || !SweepRunning) return;
            if (--SweepTimer > 0) return;

            SweepTimer = SweepPeriod == 0 ? 8 : SweepPeriod;

            // A zero period keeps the timer running at 8 but performs no step.
            if (SweepPeriod == 0) return;

            ComputeSweep(apply: true);
        }

        // Overflowing 11 bits disables the channel outright; it does not wrap or clamp.
        private void ComputeSweep(bool apply)
        {
            int delta = SweepShadow >> SweepShift;
            int next = SweepNegate ? SweepShadow - delta : SweepShadow + delta;

            if (next > 2047)
            {
                Enabled = false;
                return;
            }

            if (!apply || SweepShift == 0) return;

            SweepShadow = next;
            Frequency = next;

            // Hardware checks a second time with the new frequency, and that one can also disable.
            int recheck = SweepNegate
                ? SweepShadow - (SweepShadow >> SweepShift)
                : SweepShadow + (SweepShadow >> SweepShift);

            if (recheck > 2047) Enabled = false;
        }
    }

    // Channel 3: 32 four-bit samples a game writes itself - see Mercury_Apu.md §3.4.
    public sealed class WaveChannel
    {
        public readonly byte[] Ram = new byte[16];
        public readonly LengthCounter Length = new(256);

        public int Frequency;
        public int Timer;
        public int Position;

        // 0 mutes, then 100%, 50% and 25%; there is no finer control than a shift.
        public int VolumeCode;

        public bool Enabled;
        public bool DacEnabled;

        public int Sample => (Position & 1) == 0 ? Ram[Position >> 1] >> 4 : Ram[Position >> 1] & 0x0F;

        public int Output
        {
            get
            {
                if (!Enabled || !DacEnabled || VolumeCode == 0) return 0;
                return Sample >> (VolumeCode - 1);
            }
        }

        public void StepTimer()
        {
            if (--Timer > 0) return;

            Timer = (2048 - Frequency) * 2;
            Position = (Position + 1) & 0x1F;
        }

        public void Trigger()
        {
            Enabled = DacEnabled;
            Timer = (2048 - Frequency) * 2;
            Position = 0;
            Length.Trigger();
        }
    }

    // Channel 4: a shift register, not a tone - see Mercury_Apu.md §3.5.
    public sealed class NoiseChannel
    {
        // The eight base periods the shift then multiplies; index 0 is half of index 1, not zero.
        public static readonly int[] Divisors = { 8, 16, 32, 48, 64, 80, 96, 112 };

        public readonly Envelope Envelope = new();
        public readonly LengthCounter Length = new(64);

        public int ClockShift;
        public int DivisorCode;
        public bool ShortMode;

        public int Lfsr = 0x7FFF;
        public int Timer;

        public bool Enabled;
        public bool DacEnabled;

        // Bit 0 low is the loud half, which is why the register starts all ones.
        public int Output =>
            Enabled && DacEnabled && (Lfsr & 0x01) == 0 ? Envelope.Volume : 0;

        public void StepTimer()
        {
            if (--Timer > 0) return;

            Timer = Divisors[DivisorCode] << ClockShift;
            StepLfsr();
        }

        public void Trigger()
        {
            Enabled = DacEnabled;
            Timer = Divisors[DivisorCode] << ClockShift;
            Lfsr = 0x7FFF;

            Envelope.Trigger();
            Length.Trigger();
        }

        // Short mode feeds the same bit into position 6 as well, shortening the period to 127.
        private void StepLfsr()
        {
            int feedback = (Lfsr ^ (Lfsr >> 1)) & 0x01;
            Lfsr = (Lfsr >> 1) | (feedback << 14);

            if (ShortMode) Lfsr = (Lfsr & ~0x40) | (feedback << 6);
        }
    }
}
