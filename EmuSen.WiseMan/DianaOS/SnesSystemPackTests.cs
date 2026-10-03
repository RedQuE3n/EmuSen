using System;
using System.Linq;
using EmuSen.Cores;
using EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Sys.Systems;
using EmuSen.DianaOS.DianaOS.Sys.Systems.Snes;

namespace EmuSen.WiseMan.DianaOS
{
    // The SNES's system pack: its entry, and its two cheat formats as fullsnes describes them - see EmuSen_CoreAPI.md §20.
    public class SnesSystemPackTests
    {
        // Worked by hand from fullsnes's digit table and bit order: one address bit, then the value byte, each in isolation.
        [Theory]
        [InlineData("DDDD-DDDD", 0x000000, 0x00)]
        [InlineData("DD6D-DDDD", 0x008000, 0x00)]
        [InlineData("DDDD-4DDD", 0x800000, 0x00)]
        [InlineData("C2DD-DDDD", 0x000000, 0xAD)]
        [InlineData("dd6ddddd", 0x008000, 0x00)]
        public void A_game_genie_code_deciphers_by_the_documented_table_and_bit_order(string code, int address, int value)
        {
            Assert.Equal((address, (byte)value), SnesCheatFormats.DecodeGameGenie(code));
        }

        // The shuffle is a permutation: each code bit lands on its own address bit, and encoding inverts decoding.
        [Fact]
        public void The_address_shuffle_is_a_permutation_and_encoding_inverts_it()
        {
            var seen = new System.Collections.Generic.HashSet<int>();
            for (int bit = 0; bit < 24; bit++)
            {
                string code = SnesCheatFormats.EncodeGameGenie(1 << bit, 0);
                Assert.True(seen.Add(SnesCheatFormats.DecodeGameGenie(code).Address), code);
            }
            var random = new Random(20261003);
            for (int i = 0; i < 1000; i++)
            {
                int address = random.Next(0x1000000);
                byte value = (byte)random.Next(256);
                Assert.Equal((address, value), SnesCheatFormats.DecodeGameGenie(SnesCheatFormats.EncodeGameGenie(address, value)));
            }
            Assert.Equal(SnesCheatFormats.GenieDigits.Length, SnesCheatFormats.GenieDigits.Distinct().Count());
            Assert.Equal(SnesCheatFormats.AddressOrder.Order(), SnesCheatFormats.GenieOrder.Order());
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
            Assert.True(CheatCommand.PrefersExplicitCodec(gg, par, "C2B5-ED0D"));
            Assert.False(CheatCommand.PrefersExplicitCodec(gg, par, "7E0DBE09"));
            Assert.Throws<FormatException>(() => SnesSystem.GameGenie().Decode("12345"));
        }

        [Fact]
        public void The_pack_is_found_by_system_id_and_the_snes_row_reads_its_facts()
        {
            var pack = SystemPacks.For("snes")!;
            Assert.Same(SnesSystem.Entry, pack.Entry);
            Assert.Null(SystemPacks.For("test"));
            Assert.False(pack.Entry.SelfDelimitingImages, "a SNES image does not record its own length, so C4 may not require a truncated one refused");
            var row = CoreCatalog.ByAnyName("SNES")!;
            Assert.Equal(pack.Entry.CheatSystems, row.CheatSystems);
            Assert.Equal(pack.Entry.Extensions, row.Extensions);
            Assert.Equal((pack.Entry.Console, pack.Entry.Manufacturer, pack.Entry.ReleaseYear, pack.Entry.CoverAspect), (row.ConsoleName, row.Manufacturer, row.ReleaseYear, row.CoverAspect));
            byte[] headed = new byte[1024 + 512];
            Assert.Equal(1024, row.OpenVgdbBytes!(headed).Length);
            Assert.Equal(("snes", "Super Nintendo"), (CoreCatalog.ShelfByName(row.DisplayName)!.EsdeSystem, CoreCatalog.ShelfByName(row.DisplayName)!.EsdeFullName));
        }
    }
}
