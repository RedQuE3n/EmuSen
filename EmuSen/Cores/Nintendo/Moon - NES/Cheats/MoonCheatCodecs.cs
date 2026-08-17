using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.Cores.Nintendo.Moon.Cheats
{
    // The plugs letting CheatCommand use both decoders without DianaOS referencing this project.

    // "Game Genie", not "NES Game Genie", so the detected-format message reads as it always has.
    public sealed class NesGameGenieCheatCodec : ICheatCodeCodec
    {
        public string Name => "Game Genie";
        public CheatCodeKind Kind => CheatCodeKind.RomPatch;
        public string? SpaceName => null; // ROM patches aren't RAM-space addressed

        public bool CanDecode(string code) => NesGameGenieCodec.CanDecode(code);
        public (int Address, byte Value) Decode(string code) => NesGameGenieCodec.Decode(code);
        public byte? DecodeCompare(string code) => NesGameGenieCodec.DecodeCompare(code);
    }

    public sealed class NesRawCheatCodec : ICheatCodeCodec
    {
        public string Name => "NES address:value";
        public CheatCodeKind Kind => CheatCodeKind.RamPoke;
        public string? SpaceName => "CPUBUS";

        public bool CanDecode(string code) => NesRawCodec.CanDecode(code);
        public (int Address, byte Value) Decode(string code) => NesRawCodec.Decode(code);
    }
}
