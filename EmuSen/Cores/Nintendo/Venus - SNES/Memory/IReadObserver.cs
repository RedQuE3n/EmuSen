namespace EmuSen.Cores.Nintendo.Venus.Memory
{
    // Mirror of IWriteObserver for reads - same reasoning for living here
    // rather than in Debug/: a minimal hook MemoryBus calls with no idea
    // what's listening, so real emulation state and debug-toolchain
    // plumbing stay separate. Deliberately a SEPARATE interface from
    // IWriteObserver rather than adding OnRead to it - that would be a
    // breaking change to an existing contract for no real benefit; a
    // future implementer that only cares about writes (or only reads)
    // can implement just the one interface it needs. SnesDebugTarget
    // implements both today.
    //
    // Called far more often than IWriteObserver.OnWrite in practice - a
    // read happens on every instruction fetch and every operand read
    // that touches a watched space, not just the writes a game actually
    // makes - so implementations need to stay cheap when nothing is
    // watching (see WatchRegistry.RecordRead's own comment on this).
    public interface IReadObserver
    {
        void OnRead(string spaceName, int address, byte value);
    }
}
