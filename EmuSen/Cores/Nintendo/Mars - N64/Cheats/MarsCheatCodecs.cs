using System.Collections.Generic;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.Cores.Nintendo.Mars.Cheats
{
    // The plug letting CheatCommand and Mistress use the decoder without DianaOS referencing this project.
    public sealed class N64GameSharkCheatCodec : ICheatCodeCodec
    {
        public string Name => "GameShark";
        public CheatCodeKind Kind => CheatCodeKind.RamPoke;
        public string? SpaceName => MarsCore.SpaceRdram;

        public bool CanDecode(string code) => N64GameSharkCodec.CanDecode(code);
        public (int Address, byte Value) Decode(string code) => N64GameSharkCodec.Decode(code);
        public IReadOnlyList<CheatWrite>? DecodeWrites(string code) => N64GameSharkCodec.DecodeWrites(code);
    }
}
