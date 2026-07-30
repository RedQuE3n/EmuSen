using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.Cores.Nintendo.Venus.Cheats
{
    // Adapts the static ActionReplayCodec decoder to DianaOS's
    // ICheatCodeCodec contract, so CheatCommand can use it without
    // EmuSen.DianaOS itself referencing this project (see CheatCommand's
    // own header comment). The decode logic stays in ActionReplayCodec
    // unchanged - this is purely the plug that lets a host wire it in.
    public sealed class ActionReplayCheatCodec : ICheatCodeCodec
    {
        public string Name => "Pro Action Replay/Game Wizard";
        public CheatCodeKind Kind => CheatCodeKind.RamPoke;

        // See ActionReplayCodec's own comment on why decoded codes target
        // the CPU-bus address space, not a raw WRAM array offset.
        public string? SpaceName => "CpuBus";

        public bool CanDecode(string code) => ActionReplayCodec.CanDecode(code);
        public (int Address, byte Value) Decode(string code) => ActionReplayCodec.Decode(code);
    }
}
