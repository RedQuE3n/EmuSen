using System;

namespace EmuSen.Mistress
{
    // A late frame's time is owed, up to a cap, so frames that cost more than their slot are made up by those that cost less - see EmuSen_Settings_Reference.md §4.28.
    public static class FramePacer
    {
        public const int DebtFrames = 3;

        // The tick to wait for, or, when it has passed, the tick moved up so no more than the cap is owed.
        public static TimeSpan Settle(TimeSpan nextTick, TimeSpan now, TimeSpan interval)
        {
            TimeSpan cap = interval * DebtFrames;
            return now - nextTick > cap ? now - cap : nextTick;
        }
    }
}
