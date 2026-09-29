using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Cheats;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.Mercury.Cheats;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.Moon.Cheats;
using EmuSen.Cores.Nintendo.Venus.Cheats;
using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.WiseMan.Cores
{
    // Seven adapter classes became one DelegateCheatCodec; each format keeps the name, kind and space its class had - see EmuSen_Settings_Reference.md §4.85.7.
    public class CheatCodecAdapterTests
    {
        public static TheoryData<string, string, CheatCodeKind, string?> Formats => new()
        {
            { "NES raw", "NES address:value", CheatCodeKind.RamPoke, MoonCore.SpaceCpuBus },
            { "NES Game Genie", "Game Genie", CheatCodeKind.RomPatch, null },
            { "GB GameShark", "GameShark", CheatCodeKind.RamPoke, MercuryCore.SpaceCpuBus },
            { "GB Game Genie", "Game Genie", CheatCodeKind.RomPatch, null },
            { "N64 GameShark", "GameShark", CheatCodeKind.RamPoke, MarsCore.SpaceRdram },
            { "SNES Action Replay", "Pro Action Replay/Game Wizard", CheatCodeKind.RamPoke, "CpuBus" },
            { "SNES Game Genie", "Game Genie", CheatCodeKind.RomPatch, null },
        };

        private static ICheatCodeCodec Codec(string format) => format switch
        {
            "NES raw" => MoonCheatCodecs.Raw(),
            "NES Game Genie" => MoonCheatCodecs.GameGenie(),
            "GB GameShark" => MercuryCheatCodecs.GameShark(),
            "GB Game Genie" => MercuryCheatCodecs.GameGenie(),
            "N64 GameShark" => MarsCheatCodecs.GameShark(),
            "SNES Action Replay" => VenusCheatCodecs.ActionReplay(),
            _ => VenusCheatCodecs.GameGenie(),
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
            Assert.Equal(NesGameGenieCodec.DecodeCompare("SXIOPOVK"), MoonCheatCodecs.GameGenie().DecodeCompare("SXIOPOVK"));
            Assert.NotNull(MoonCheatCodecs.GameGenie().DecodeCompare("SXIOPOVK"));
            Assert.Null(VenusCheatCodecs.GameGenie().DecodeCompare("DD62-6DAD"));
            Assert.Equal(3, MarsCheatCodecs.GameShark().DecodeWrites("8110FB40 FFFF+8110FB42 F801+80001003 0000")!.Count);
            Assert.Null(MoonCheatCodecs.Raw().DecodeWrites("0x0042:99"));
        }
    }
}
