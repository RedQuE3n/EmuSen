namespace EmuSen.Crystal
{
    // Exact rate conversion between a device's clock and the master clock - see EmuSen_Crystal_Scheduler.md §5.
    public readonly struct ClockRatio
    {
        // device ticks = master ticks * Numerator / Denominator.
        public readonly long Numerator;
        public readonly long Denominator;

        public ClockRatio(long numerator, long denominator)
        {
            if (numerator <= 0) throw new ArgumentOutOfRangeException(nameof(numerator), "A clock ratio must be positive.");
            if (denominator <= 0) throw new ArgumentOutOfRangeException(nameof(denominator), "A clock ratio must be positive.");

            long g = Gcd(numerator, denominator);
            Numerator = numerator / g;
            Denominator = denominator / g;
        }

        // Named for the two rates rather than the direction, which is the part that gets inverted by mistake.
        public static ClockRatio FromHz(long deviceHz, long masterHz) => new(deviceHz, masterHz);

        public bool IsIdentity => Numerator == Denominator;

        // Truncating; use ClockAccumulator when converting repeatedly, or the remainder is lost each time.
        public long ToDevice(long masterTicks) => masterTicks * Numerator / Denominator;

        public long ToMaster(long deviceTicks) => deviceTicks * Denominator / Numerator;

        // The smallest master span that yields at least deviceTicks, so a deadline never lands short.
        public long MasterTicksFor(long deviceTicks) => (deviceTicks * Denominator + Numerator - 1) / Numerator;

        public override string ToString() => $"{Numerator}/{Denominator}";

        private static long Gcd(long a, long b)
        {
            while (b != 0) (a, b) = (b, a % b);
            return a;
        }
    }
}
