namespace EmuSen.Cores.Nintendo.Venus.Memory
{
    // Mirror of IWriteObserver/IReadObserver/IFrameObserver for the
    // cartridge-read intercept mechanism era devices like Game Genie use -
    // see Venus_Memory.md §6. Unlike the other three (which only ever
    // observe), this one can actually change what the CPU sees: called for
    // every cartridge-routed read (ROM or SRAM - a real Game Genie device
    // can only ever see addresses that reach the cartridge edge connector
    // at all, same as this hook only fires from MemoryBus's cartridge
    // fallthrough, never for WRAM or a hardware register).
    // originalValue is whatever the cartridge itself returned, needed for
    // compare-gated patches (a code that only applies while the real byte
    // there matches an expected value). Returns true (and sets
    // patchedValue) if a patch should override that byte for this read.
    public interface IRomReadPatcher
    {
        bool TryPatch(uint address, byte originalValue, out byte patchedValue);
    }
}
