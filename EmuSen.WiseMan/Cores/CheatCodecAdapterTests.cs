using EmuSen.Crystal.Systems.N64;
using EmuSen.Cores.Native;
using EmuSen.WiseMan.Fixtures;
using EmuSen.Crystal.Systems.GameBoy;
using EmuSen.Crystal.Systems.Nes;
using EmuSen.Crystal.Systems.Snes;
using EmuSen.Crystal.Cheats;
using EmuSen.Cores.Nintendo.MercuryRT;

namespace EmuSen.WiseMan.Cores
{
    // Seven adapter classes became one DelegateCheatCodec; each format keeps the name, kind and space its class had - see EmuSen_Settings_Reference.md §4.85.7.
    public class CheatCodecAdapterTests
    {
        public static TheoryData<string, string, CheatCodeKind, string?> Formats => new()
        {
            { "NES raw", "NES address:value", CheatCodeKind.RamPoke, NesSystem.CpuBus },
            { "NES Game Genie", "Game Genie", CheatCodeKind.RomPatch, null },
            { "GB GameShark", "GameShark", CheatCodeKind.RamPoke, MercuryRtCore.SpaceCpuBus },
            { "GB Game Genie", "Game Genie", CheatCodeKind.RomPatch, null },
            { "N64 GameShark", "GameShark", CheatCodeKind.RamPoke, MarsRt.Rdram },
            { "SNES Action Replay", "Pro Action Replay", CheatCodeKind.RamPoke, SnesSystem.CpuBus },
            { "SNES Game Genie", "Game Genie", CheatCodeKind.RomPatch, null },
        };

        private static ICheatCodeCodec Codec(string format) => format switch
        {
            "NES raw" => NesSystem.Raw(),
            "NES Game Genie" => NesSystem.GameGenie(),
            "GB GameShark" => GameBoyCheatCodecs.GameShark(),
            "GB Game Genie" => GameBoyCheatCodecs.GameGenie(),
            "N64 GameShark" => N64System.GameShark(),
            "SNES Action Replay" => SnesSystem.ProActionReplay(),
            _ => SnesSystem.GameGenie(),
        };

        [Theory]
        [MemberData(nameof(Formats))]
        public void Each_format_keeps_its_name_kind_and_space(string format, string name, CheatCodeKind kind, string? space)
        {
            ICheatCodeCodec codec = Codec(format);
            Assert.Equal((name, kind, space), (codec.Name, codec.Kind, codec.SpaceName));
        }

        // The optional halves reach the static decoders only where a format has them.
        [Fact]
        public void Compares_and_wide_writes_are_forwarded_where_a_format_has_them()
        {
            Assert.Equal(NesGameGenieCodec.DecodeCompare("SXIOPOVK"), NesSystem.GameGenie().DecodeCompare("SXIOPOVK"));
            Assert.NotNull(NesSystem.GameGenie().DecodeCompare("SXIOPOVK"));
            Assert.Null(SnesSystem.GameGenie().DecodeCompare("DD62-6DAD"));
            Assert.Equal(3, N64System.GameShark().DecodeWrites("8110FB40 FFFF+8110FB42 F801+80001003 0000")!.Count);
            Assert.Null(NesSystem.Raw().DecodeWrites("0x0042:99"));
        }
    }
}
