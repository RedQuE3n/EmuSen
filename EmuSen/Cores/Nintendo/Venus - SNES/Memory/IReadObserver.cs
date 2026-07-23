namespace EmuSen.Cores.Nintendo.Venus.Memory
{
    // Mirror of IWriteObserver for reads - see Venus_Memory.md §6.
    public interface IReadObserver
    {
        void OnRead(string spaceName, int address, byte value);
    }
}
