namespace EmuSen.Crystal
{
    // A chip that consumes time and can be run forward to a point - see EmuSen_Crystal_Scheduler.md §2.
    public interface IClockedDevice
    {
        // Where this device has reached on the master timeline; may lag Scheduler.Now.
        long Clock { get; }

        // Run forward until Clock is at or past masterClock; must never run backwards.
        void SyncTo(long masterClock);
    }
}
