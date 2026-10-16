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

        // The folder asked for each call, as the cheat command read the player's setting each time, where no fixed folder is given.
        public Func<string>? CheatDatabaseFolder { get; init; }
        public Action<string>? SaveState { get; init; }
        public Action<string>? LoadState { get; init; }
        public Func<string>? DefaultStatePath { get; init; }
    }
}
