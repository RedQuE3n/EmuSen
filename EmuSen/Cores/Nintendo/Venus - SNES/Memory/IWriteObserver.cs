namespace EmuSen.Cores.Nintendo.Venus.Memory
{
    // A minimal hook for observing bus writes - see Venus_Memory.md §6.
    public interface IWriteObserver
    {
        void OnWrite(string spaceName, int address, byte value);
    }
}
