using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
namespace EmuSen.DianaOS.DianaOS.Lib
{
    // What kind of cheat a successfully decoded code becomes - RamPoke
    // targets SpaceName at Address every frame, RomPatch is an
    // unconditional ROM-read intercept (see IDebugTarget.Cheats /
    // CheatKind for the registry-side counterpart these map onto).
    public enum CheatCodeKind
    {
        RamPoke,
        RomPatch,
    }

    // A single cheat-code format decoder ("Pro Action Replay/Game
    // Wizard", "Game Genie", or any future format) a core plugs into
    // CheatCommand at construction (see DianaOSInterpreter.CreateDefault's
    // cheatCodec parameters). Exists so CheatCommand itself can stay
    // compiled with zero references to any specific core's code-format
    // decoders (see CheatCommand's own header comment) while still
    // offering the same decode-on-demand `add`/`gg` commands - this is
    // exactly the "future auto-detecting cheat add that tries each known
    // codec in turn" ActionReplayCodec.CanDecode/GameGenieCodec.CanDecode
    // were already written for.
    public interface ICheatCodeCodec
    {
        // Human-readable name for `add`'s own "(detected X format)" message.
        string Name { get; }

        // True if <code> looks like this codec's format at all.
        bool CanDecode(string code);

        CheatCodeKind Kind { get; }

        // The memory space a decoded RamPoke address targets (e.g.
        // "CpuBus"). Unused (and may be null) when Kind is RomPatch.
        string? SpaceName { get; }

        (int Address, byte Value) Decode(string code);
    }
}
