namespace EmuSen.Cauldron
{
    // One kind of live core information, published as an immutable snapshot - see EmuSen_Debugging_Tools_Reference_v5.md §3.66.2.
    public interface IRealtimeProvider<T>
    {
        // Never blocks, never touches live core state, safe from any thread - see §3.66.2.
        T Current { get; }

        // Emulation thread only: the one member here that touches live core state - see §3.66.2.
        void Refresh();
    }
}
