namespace EmuSen.Cores.Nintendo.Venus.Memory
{
    // Mirror of IWriteObserver/IReadObserver for frame boundaries - see
    // Venus_Memory.md §6.
    public interface IFrameObserver
    {
        void OnFrame(long frameCount);
    }
}
