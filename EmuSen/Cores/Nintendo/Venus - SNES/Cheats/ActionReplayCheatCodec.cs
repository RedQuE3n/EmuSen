using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.Cores.Nintendo.Venus.Cheats
{
    // Adapts the static ActionReplayCodec decoder to DianaOS's ICheatCodeCodec contract, so CheatCommand.
    public sealed class ActionReplayCheatCodec : ICheatCodeCodec
    {
        public string Name => "Pro Action Replay/Game Wizard";
        public CheatCodeKind Kind => CheatCodeKind.RamPoke;

        // Decoded codes target the CPU bus, not a raw WRAM offset, so mirroring resolves as on hardware.
        public string? SpaceName => "CpuBus";

        public bool CanDecode(string code) => ActionReplayCodec.CanDecode(code);
        public (int Address, byte Value) Decode(string code) => ActionReplayCodec.Decode(code);
    }
}
