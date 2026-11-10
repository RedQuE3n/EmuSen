using System;
using System.Linq;
using EmuSen.WiseMan.Bindings.Cheats;
using EmuSen.WiseMan.Bindings.Systems;
using EmuSen.WiseMan.Bindings.Systems.Snes;
using EmuSen.Mistress.Library;

namespace EmuSen.WiseMan.DianaSh
{
    // The SNES's system pack: its entry, and its two cheat formats as fullsnes describes them - see EmuSen_CoreAPI.md §20.
    public class SnesSystemPackTests
    {

        // `cheat add`'s rule, as the cheat module and Mistress's list apply it: the codec that alone claims the code, else the Game Genie's look - see EmuSen_Cheats.md.
        private static bool PrefersExplicit(ICheatCodeCodec explicitCodec, ICheatCodeCodec autoCodec, string code) =>
            explicitCodec.CanDecode(code) != autoCodec.CanDecode(code) ? explicitCodec.CanDecode(code) : EmuSen.Mistress.CheatList.LooksLikeGameGenieFormat(code);
        // Worked by hand from fullsnes's digit table and bit order: one address bit, then the value byte, each in isolation.
        [Theory]
        [InlineData("DDDD-DDDD", 0x000000, 0x00)]
        [InlineData("DD6D-DDDD", 0x008000, 0x00)]
        [InlineData("DDDD-4DDD", 0x800000, 0x00)]
        [InlineData("C2DD-DDDD", 0x000000, 0xAD)]
        [InlineData("dd6ddddd", 0x008000, 0x00)]
        public void A_game_genie_code_deciphers_by_the_documented_table_and_bit_order(string code, int address, int value)
        {
            Assert.Equal((address, (byte)value), SnesSystem.GameGenie().Decode(code));
        }

        // The shuffle is a permutation: each code bit lands on its own address bit, and the library's encoding inverts decoding.
        [Fact]
        public void The_address_shuffle_is_a_permutation_and_encoding_inverts_it()
        {
            var seen = new System.Collections.Generic.HashSet<int>();
            for (int bit = 0; bit < 24; bit++)
            {
                string code = SystemCode.CheatEncode("snes.gg", 1 << bit, 0);
                Assert.True(seen.Add(SnesSystem.GameGenie().Decode(code).Address), code);
            }
            var random = new Random(20261003);
            for (int i = 0; i < 1000; i++)
            {
                int address = random.Next(0x1000000);
                byte value = (byte)random.Next(256);
                Assert.Equal((address, value), SnesSystem.GameGenie().Decode(SystemCode.CheatEncode("snes.gg", address, value)));
            }
        }

        [Fact]
        public void An_action_replay_code_pokes_wram_at_7e_and_7f_and_the_cpu_bus_elsewhere()
        {
            var par = SnesSystem.ProActionReplay();
            Assert.Equal((CheatCodeKind.RamPoke, "CpuBus"), (par.Kind, par.SpaceName));
            var wram = par.DecodeWrites("7E0DBE09")!.Single();
            Assert.Equal(("WRAM", 0x0DBE, 9u), (wram.Space, wram.Address, wram.Value));
            var high = par.DecodeWrites("7FFFFF01")!.Single();
            Assert.Equal(("WRAM", 0x1FFFF), (high.Space, high.Address));
            var bus = par.DecodeWrites("00808001")!.Single();
            Assert.Equal(("CpuBus", 0x008080, 1u), (bus.Space, bus.Address, bus.Value));
            Assert.Empty(par.DecodeWrites("7E000000")!);
            Assert.Equal((0x7E0DBE, (byte)9), par.Decode("7E0DBE09"));
        }

        // The libretro database joins a cheat's codes with '+'; the pack read only the first form until 2026-10-16 - see VenusRT_Native.md §68.4.
        [Fact]
        public void Action_replay_codes_joined_with_a_plus_are_one_cheat()
        {
            var writes = SnesSystem.ProActionReplay().DecodeWrites("7E0DBF63+7E000000+00808001")!;
            Assert.Equal(new[] { ("WRAM", 0x0DBF, 0x63u), ("CpuBus", 0x008080, 1u) }, writes.Select(w => (w.Space, w.Address, w.Value)));
            Assert.Throws<FormatException>(() => SnesSystem.ProActionReplay().DecodeWrites("7E0DBF63+7E00"));
        }

        [Theory]
        [InlineData("7E0DBE09", true, false)]
        [InlineData("C2B5-ED0D", false, true)]
        [InlineData("FE001234", false, false)]
        [InlineData("FF00FF00", false, false)]
        [InlineData("DEADC0DE", false, false)]
        [InlineData("C0DE0300", false, false)]
        [InlineData("7E0DBE0", false, false)]
        [InlineData("XYZW-1234", false, false)]
        public void Each_format_claims_only_its_own_printed_form(string code, bool actionReplay, bool gameGenie)
        {
            Assert.Equal(actionReplay, SnesSystem.ProActionReplay().CanDecode(code));
            Assert.Equal(gameGenie, SnesSystem.GameGenie().CanDecode(code));
        }

        [Fact]
        public void The_cheat_command_routes_a_dashed_code_to_the_game_genie_and_a_bare_one_to_the_action_replay()
        {
            var (par, gg) = (SnesSystem.ProActionReplay(), SnesSystem.GameGenie());
            Assert.True(PrefersExplicit(gg, par, "C2B5-ED0D"));
            Assert.False(PrefersExplicit(gg, par, "7E0DBE09"));
            Assert.Throws<FormatException>(() => SnesSystem.GameGenie().Decode("12345"));
        }

        [Fact]
        public void The_pack_is_found_by_system_id_and_the_snes_row_reads_its_facts()
        {
            var pack = SystemPacks.For("snes")!;
            Assert.Same(SnesSystem.Entry, pack.Entry);
            Assert.Null(SystemPacks.For("test"));
            Assert.False(pack.Entry.SelfDelimitingImages, "a SNES image does not record its own length, so C4 may not require a truncated one refused");
            var row = RuntimeCatalog.ByAnyName("SNES")!;
            var facts = RuntimeCatalog.SystemOf("SNES")!;
            Assert.Equal(pack.Entry.CheatSystems, row.CheatSystems);
            Assert.Equal(pack.Entry.Extensions, row.Extensions);
            Assert.Equal((pack.Entry.Console, pack.Entry.Manufacturer, pack.Entry.ReleaseYear, pack.Entry.CoverAspect), (row.Console, facts.Manufacturer, facts.ReleaseYear, row.CoverAspect));
            LibraryShelf shelf = RuntimeCatalog.Shelves.First(s => s.Core.Console == "SNES");
            Assert.Equal(("snes", "Nintendo SNES (Super Nintendo)"), (shelf.EsdeSystem, shelf.EsdeFullName));
        }
    }
}
