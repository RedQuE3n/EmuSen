namespace EmuSen.Crystal
{
    // Repeated rate conversion that carries its remainder, so it cannot drift - see EmuSen_Crystal_Scheduler.md §5.
    public struct ClockAccumulator
    {
        // Master ticks earned but not yet worth a whole device tick.
        private long _remainder;

        public long Remainder => _remainder;

        // Whole device ticks earned by advancing masterDelta, remainder carried to the next call.
        public long Advance(long masterDelta, ClockRatio ratio)
        {
            if (masterDelta < 0) throw new ArgumentOutOfRangeException(nameof(masterDelta), "A clock cannot run backwards.");

            long scaled = masterDelta * ratio.Numerator + _remainder;
            _remainder = scaled % ratio.Denominator;
            return scaled / ratio.Denominator;
        }

        public void Reset() => _remainder = 0;
    }
}
