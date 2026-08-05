using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.Cores.Nintendo.Moon.Cheats
{
    // The plugs that let CheatCommand use the two decoders without DianaOS
    // referencing this project - the Venus pair's counterpart.

    // "Game Genie", not "NES Game Genie", so `add`'s "(detected X format)"
    // message reads the way it always has for the SNES device.
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
