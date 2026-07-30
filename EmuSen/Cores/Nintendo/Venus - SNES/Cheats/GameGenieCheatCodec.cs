using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.Cores.Nintendo.Venus.Cheats
{
    // Adapts the static GameGenieCodec decoder to DianaOS's
    // ICheatCodeCodec contract, so CheatCommand can use it without
    // EmuSen.DianaOS itself referencing this project (see CheatCommand's
    // own header comment). The decode logic stays in GameGenieCodec
    // unchanged - this is purely the plug that lets a host wire it in.
    public sealed class GameGenieCheatCodec : ICheatCodeCodec
    {
        // "Game Genie", not "SNES Game Genie" - matches CheatCommand's
        // pre-extraction message text exactly (`add`'s own
        // "(detected X format)" wording never mentioned the console).
        public string Name => "Game Genie";
        public CheatCodeKind Kind => CheatCodeKind.RomPatch;
        public string? SpaceName => null; // ROM patches aren't RAM-space addressed

        public bool CanDecode(string code) => GameGenieCodec.CanDecode(code);
        public (int Address, byte Value) Decode(string code) => GameGenieCodec.Decode(code);
    }
}
