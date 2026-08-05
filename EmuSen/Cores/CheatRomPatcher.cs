using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.Cores
{
    // Points the bus's cartridge-read intercept at a CheatRegistry. Both are
    // already core-agnostic; this is only the wire between them, so a core
    // that owns its own registry needs no debug target for Game Genie-style
    // patches to work - see EmuSen_Cheats.md §3.
    public sealed class CheatRomPatcher : IRomReadPatcher
    {
        private readonly CheatRegistry _cheats;

        public CheatRomPatcher(CheatRegistry cheats) => _cheats = cheats;

        // Runs on every cartridge-routed read, so the no-patches case must not walk the list.
        public bool TryPatch(uint address, byte originalValue, out byte patchedValue) =>
            _cheats.TryPatchRom(address, originalValue, out patchedValue);
    }
}
