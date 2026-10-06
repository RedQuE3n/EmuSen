using EmuSen.Cores.Native;
using EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Sys.Systems;
using EmuSen.DianaOS.DianaOS.Sys.Systems.Genesis;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.WiseMan.DianaOS
{
    // The Genesis pack's Game Genie and Action Replay codecs, held to their documents' worked examples - see EmuSen_Cheats.md §9.
    public class GenesisCheatFormatsTests
    {
        private static ICheatCodeCodec Poke => SystemPacks.For("md")!.AutoDetectCodec!();
        private static ICheatCodeCodec Patch => SystemPacks.For("md")!.ExplicitCodec!();

        [Fact]
        public void The_game_genie_method_documents_worked_example_decodes_to_its_address_and_word()
        {
            Assert.Equal((0x009C76, (ushort)0x5478), GenesisCheatFormats.DecodeGameGenie("SCRA-BJX0"));
            Assert.Equal((0x009C76, (ushort)0x5478), GenesisCheatFormats.DecodeGameGenie("scra-bjx0"));
            Assert.Equal("SCRA-BJX0", GenesisCheatFormats.EncodeGameGenie(0x009C76, 0x5478));
        }

        [Fact]
        public void Every_address_and_word_survives_encoding_and_decoding()
        {
            var random = new Random(1988);
            for (int i = 0; i < 2000; i++)
            {
                int address = random.Next(0x100_0000);
                ushort value = (ushort)random.Next(0x1_0000);
                string code = GenesisCheatFormats.EncodeGameGenie(address, value);
                Assert.True(GenesisCheatFormats.IsGameGenie(code), code);
                Assert.Equal((address, value), GenesisCheatFormats.DecodeGameGenie(code));
            }
        }

        [Theory]
        [InlineData("SCRA-BJIO")]
        [InlineData("SCRABJX0")]
        [InlineData("SCRA-BJX")]
        public void A_code_outside_the_genies_form_is_not_one(string code) => Assert.False(GenesisCheatFormats.IsGameGenie(code));

        [Fact]
        public void A_game_genie_code_is_the_word_its_address_falls_in()
        {
            var writes = Patch.DecodeWrites("SCRA-BJX0")!;
            var w = Assert.Single(writes);
            Assert.Equal((0x009C76, 0x5478u, 2, true), (w.Address, w.Value, w.Width, w.BigEndian));
            string odd = GenesisCheatFormats.EncodeGameGenie(0x009C77, 0x1234);
            Assert.Equal(0x009C76, Assert.Single(Patch.DecodeWrites(odd)!).Address);
        }

        [Theory]
        [InlineData("FFFFE0:0001", 0xFFE0, 0x0001u, 2)]
        [InlineData("FFFFE 00001", 0xFFE0, 0x0001u, 2)]
        [InlineData("FFFFE-00001", 0xFFE0, 0x0001u, 2)]
        [InlineData("FFFFE00001", 0xFFE0, 0x0001u, 2)]
        [InlineData("FF0213:50", 0x0213, 0x50u, 1)]
        [InlineData("FFD5E3FF", 0xD5E3, 0xFFu, 1)]
        [InlineData("E0FFE0:0001", 0xFFE0, 0x0001u, 2)]
        [InlineData("FFBF02:000004FF", 0xBF02, 0x04FFu, 4)]
        public void An_action_replay_code_pokes_the_main_ram(string code, int address, uint value, int width)
        {
            Assert.True(Poke.CanDecode(code), code);
            Assert.False(CheatCommand.PrefersExplicitCodec(Patch, Poke, code), code);
            var w = Assert.Single(Poke.DecodeWrites(code)!);
            Assert.Equal((GenesisSystems.Wram, address, value, width, true), (w.Space, w.Address, w.Value, w.EffectiveWidth, w.BigEndian));
        }

        [Fact]
        public void An_action_replay_code_for_the_cartridge_is_a_rom_patch()
        {
            const string code = "002D50:6004";
            Assert.False(Poke.CanDecode(code));
            Assert.True(CheatCommand.PrefersExplicitCodec(Patch, Poke, code));
            var w = Assert.Single(Patch.DecodeWrites(code)!);
            Assert.Equal((0x2D50, 0x6004u, 2), (w.Address, w.Value, w.EffectiveWidth));
            Assert.False(Poke.CanDecode("B00080:3200") || Patch.CanDecode("B00080:3200"));
            Assert.False(Poke.CanDecode("FF7E26:??"));
        }

        [Fact]
        public void Codes_joined_with_a_plus_are_one_cheat()
        {
            Assert.Equal(2, Poke.DecodeWrites("FFE4A2:0020+FFE4A4:10")!.Count);
            Assert.True(CheatCommand.PrefersExplicitCodec(Patch, Poke, "SCRA-BJX0+ATGA-AA56"));
            Assert.Equal(2, Patch.DecodeWrites("SCRA-BJX0+ATGA-AA56")!.Count);
            Assert.False(Patch.CanDecode("SCRA-BJX0+FFE4A2:0020"));
        }

        [Fact]
        public void A_game_genie_code_reaches_the_core_as_two_patched_bytes()
        {
            var registry = new CheatRegistry();
            registry.AddCheat(CheatKind.RomPatch, Patch.DecodeWrites("SCRA-BJX0")!, null, "rings");
            Assert.Equal(new uint[] { 0x9C76, 0x54, uint.MaxValue, 0x9C77, 0x78, uint.MaxValue }, NativeRtCore<NativeMachine>.RomPatchTriples(registry, 0, 0x3F_FFFF));
        }

        [Fact]
        public void A_cht_file_brings_its_game_genie_codes_as_rom_patches_and_its_pokes_as_pokes()
        {
            const string text = "cheats = 3\n\ncheat0_desc = \"Rings\"\ncheat0_code = \"SCRA-BJX0\"\ncheat0_enable = false\n\n" +
                "cheat1_desc = \"Lives\"\ncheat1_code = \"FFFE12:0009\"\ncheat1_enable = false\n\n" +
                "cheat2_desc = \"Asked\"\ncheat2_code = \"FF7E26:??\"\ncheat2_enable = false\n";
            var parsed = ChtFile.Parse(text, Poke, GenesisSystems.Wram, Patch);
            Assert.Equal(1, parsed.Skipped);
            Assert.Equal(new[] { CheatKind.RomPatch, CheatKind.RamPoke }, parsed.Cheats.Select(c => c.Kind));
            var registry = new CheatRegistry();
            Assert.Equal(2, CheatImport.FromChtText(registry, text, Poke, patchCodec: Patch).Loaded);
            Assert.Empty(ChtFile.Parse(text, Poke, GenesisSystems.Wram).Cheats.Where(c => c.Kind == CheatKind.RomPatch));
        }

        [Fact]
        public void The_attachments_packs_share_the_consoles_formats()
        {
            foreach (string id in new[] { "md", "mcd", "32x" })
            {
                var pack = SystemPacks.For(id)!;
                Assert.Equal(("Action Replay", CheatCodeKind.RamPoke, GenesisSystems.Wram), (pack.AutoDetectCodec!().Name, pack.AutoDetectCodec!().Kind, pack.AutoDetectCodec!().SpaceName));
                Assert.Equal(("Game Genie", CheatCodeKind.RomPatch), (pack.ExplicitCodec!().Name, pack.ExplicitCodec!().Kind));
            }
        }
    }
}
