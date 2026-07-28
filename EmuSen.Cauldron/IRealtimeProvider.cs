namespace EmuSen.Cauldron
{
    // Supplies one kind of real-time information from a live core (CPU
    // registers, sprite/OAM state, a hardware-load timing breakdown, ...)
    // as an immutable snapshot a consumer can read without ever touching
    // the core directly. Deliberately knows nothing about DianaOS, cores,
    // or any specific console - a core populates one of these at its own
    // pace (see Refresh), and anything (DianaOS today, conceivably
    // something else later) can read Current without needing to know how
    // or how often that happens.
    //
    // T should be an immutable snapshot (a readonly struct, or a
    // reference to a list that's never mutated after being handed to
    // Refresh) - Current is published without a lock, so a consumer
    // reading a half-mutated T would be exactly the kind of race this
    // exists to avoid.
    public interface IRealtimeProvider<T>
    {
        // The last published snapshot. Reading this never blocks and
        // never touches live core state - safe to call from any thread,
        // at any time, including concurrently with Refresh() running on
        // another thread.
        T Current { get; }

        // Pulls a fresh snapshot from the live core and publishes it as
        // Current. Must only ever be called from the thread that actually
        // owns the core (the emulation thread) - this is the one member
        // on this interface that touches live, mutable core state, and
        // calling it from two threads at once (or concurrently with
        // whatever code elsewhere is mutating that same core state)
        // would defeat the whole point.
        void Refresh();
    }
}
