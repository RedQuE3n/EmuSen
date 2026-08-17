namespace EmuSen.Cores
{
    // The two hooks a debug target installs on a core's bus - see EmuSen_Multicore.md §11.

    // Mirror of IWriteObserver/IReadObserver for frame boundaries - see Venus_Memory.md §6.
    public interface IFrameObserver
    {
        void OnFrame(long frameCount);
    }

    // It can change what the CPU sees, unlike the observers - see Venus_Memory.md §6.
    public interface IRomReadPatcher
    {
        bool TryPatch(uint address, byte originalValue, out byte patchedValue);
    }
}
