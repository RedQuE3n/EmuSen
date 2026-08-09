using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.Cores.Nintendo.Mercury.Cheats
{
    // The plugs that let CheatCommand use the two decoders without DianaOS
    // referencing this project - the Venus and Moon pairs' counterpart.

    // "Game Genie", not "Game Boy Game Genie", so `add`'s detected-format message reads as it always has.
    public sealed class GbGameGenieCheatCodec : ICheatCodeCodec
    {
        public string Name => "Game Genie";
        public CheatCodeKind Kind => CheatCodeKind.RomPatch;
        public string? SpaceName => null;

        public bool CanDecode(string code) => GbGameGenieCodec.CanDecode(code);
        public (int Address, byte Value) Decode(string code) => GbGameGenieCodec.Decode(code);
        public byte? DecodeCompare(string code) => GbGameGenieCodec.DecodeCompare(code);
    }

    public sealed class GbGameSharkCheatCodec : ICheatCodeCodec
    {
        public string Name => "GameShark";
        public CheatCodeKind Kind => CheatCodeKind.RamPoke;
        public string? SpaceName => MercuryCore.SpaceCpuBus;

        public bool CanDecode(string code) => GbGameSharkCodec.CanDecode(code);
        public (int Address, byte Value) Decode(string code) => GbGameSharkCodec.Decode(code);
    }
}
