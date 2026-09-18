namespace EmuSen.Cores.Nintendo.Mars.Memory
{
    // How the bus reports a processor store to whatever is watching - the seam `watch` and `bp w` hang on; see Mars_Debug.md §3.
    public interface IWriteObserver
    {
        void OnWrite(string spaceName, int address, byte value);
    }
}
