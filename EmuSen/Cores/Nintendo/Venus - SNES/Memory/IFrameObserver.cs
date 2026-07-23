namespace EmuSen.Cores.Nintendo.Venus.Memory
{
    // A minimal hook for observing frame boundaries, same reasoning as
    // IWriteObserver/IReadObserver for why this lives here rather than
    // under Debug/: MemoryBus (which already owns FrameCount - see that
    // field's own comment) calls this with no idea what's listening,
    // keeping real emulation state and debug-toolchain plumbing separate.
    // Called once per rendered frame, not once per memory access, so
    // implementations can afford to do real work here (unlike OnRead,
    // which has to stay cheap - see IReadObserver's comment).
    public interface IFrameObserver
    {
        void OnFrame(long frameCount);
    }
}
