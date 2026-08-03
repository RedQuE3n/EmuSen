namespace EmuSen.Cores.Nintendo.Venus.Memory
{
    // Mirror of IWriteObserver for reads - see Venus_Memory.md §6.
    public interface IReadObserver
    {
        void OnRead(string spaceName, int address, byte value);

        // Mirror of IWriteObserver.OnCoprocessorWrite - see Venus_SuperFX.md §8.4.
        void OnCoprocessorRead(string spaceName, int address, byte value) => OnRead(spaceName, address, value);
    }
}
