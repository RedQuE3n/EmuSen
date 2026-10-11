using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Debugger;

namespace EmuSen.Crystal.Session
{
    // What a host hands the provider besides the target: today's CreateDefault parameters and the state command's delegates - see EmuSen_DianaSh.md §13.3 and §13.6.
    public sealed record SessionServiceOptions
    {
        public ICheatCodeCodec? AutoDetectCodec { get; init; }
        public ICheatCodeCodec? ExplicitCodec { get; init; }
        public ICpuTraceSwitch? Trace { get; init; }
        public Func<IReadOnlyCollection<string>>? CheatSystems { get; init; }
        public string? CheatDatabaseDirectory { get; init; }
        public Action<string>? SaveState { get; init; }
        public Action<string>? LoadState { get; init; }
        public Func<string>? DefaultStatePath { get; init; }
    }
}
