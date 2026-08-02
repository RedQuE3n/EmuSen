namespace EmuSen.Cores.Nintendo.Venus.Memory
{
    // A minimal hook for observing bus writes - see Venus_Memory.md §6.
    public interface IWriteObserver
    {
        void OnWrite(string spaceName, int address, byte value);

        // A write a cartridge coprocessor made itself, so the event can be
        // labelled with that chip's PC rather than the S-CPU's, which is
        // meaningless for it - see Venus_SA1.md §11.4. Defaulted so an
        // observer that does not care stays unchanged.
        void OnCoprocessorWrite(string spaceName, int address, byte value) => OnWrite(spaceName, address, value);
    }
}
