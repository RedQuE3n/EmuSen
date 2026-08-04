namespace EmuSen.Cores.Nintendo.Mercury.Memory
{
    // How the bus reports a write to whatever is watching - the seam `watch` hangs on.
    public interface IWriteObserver
    {
        void OnWrite(string spaceName, int address, byte value);
    }
}
