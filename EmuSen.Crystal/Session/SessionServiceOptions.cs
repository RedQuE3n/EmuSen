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

        // The id the session is listed and answered by; a host with one session keeps 1.
        public uint Id { get; init; } = SessionService.SessionId;

        // Held around every change, so a change from another thread lands between two frames - see EmuSen_Backend.md §14.3.
        public IChangeGate? Gate { get; init; }

        // Run control as a runtime answers it, given the provider's own answer to call; null leaves it to the provider.
        public Func<uint, uint, long, Func<long>, long>? RunControl { get; init; }
    }
}
