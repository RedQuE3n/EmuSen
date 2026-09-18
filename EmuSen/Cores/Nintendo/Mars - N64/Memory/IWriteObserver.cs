namespace EmuSen.Cores.Nintendo.Mars.Memory
{
    // How the bus reports a processor store to whatever is watching - the seam `watch` and `bp w` hang on; see Mars_Debug.md §3.
    public interface IWriteObserver
    {
        void OnWrite(string spaceName, int address, byte value);

        // False when OnWrite would do nothing, which lets the bus skip the report - see Mars_Performance.md §16.
        bool Listening { get; }
    }
}
