namespace EmuSen.Cores
{
    // The two hooks a debug target installs on a core's bus. Neither says
    // anything about a console, so both are shared rather than one pair per
    // core - see EmuSen_Multicore.md §11.

    // Mirror of IWriteObserver/IReadObserver for frame boundaries - see
    // Venus_Memory.md §6.
    public interface IFrameObserver
    {
        void OnFrame(long frameCount);
    }

    // The cartridge-read intercept era devices like Game Genie use - see
    // Venus_Memory.md §6 and Moon_Cheats.md §3. Unlike the observers (which
    // only ever observe), this one can change what the CPU sees: it is
    // called for every cartridge-routed read, which is exactly the set of
    // addresses a real Game Genie sitting in the edge connector can see.
    // originalValue is whatever the cartridge itself returned, needed for
    // compare-gated patches. Returns true (and sets patchedValue) if a
    // patch should override that byte for this read.
    public interface IRomReadPatcher
    {
        bool TryPatch(uint address, byte originalValue, out byte patchedValue);
    }
}
