namespace EmuSen.Cauldron
{
    // One kind of live core information, published as an immutable snapshot - see EmuSen_Cauldron.md §2.
    public interface IRealtimeProvider<T>
    {
        // Never blocks, never touches live core state, safe from any thread - see §2.
        T Current { get; }

        // Emulation thread only: the one member here that touches live core state - see §2.
        void Refresh();
    }
}
