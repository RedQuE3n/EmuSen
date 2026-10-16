using System;
using System.Linq;
using EmuSen.Cores;
using EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Systems;
using EmuSen.Crystal.Systems.Snes;

namespace EmuSen.WiseMan.DianaSh
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

        // fullsnes's names for the B bus, the APU's four ports by their mirrors, and an address it leaves unnamed - see EmuSen_CoreAPI.md §31.2.
        [Theory]
        [InlineData(0x00, "INIDISP $2100")]
        [InlineData(0x32, "COLDATA $2132")]
        [InlineData(0x39, "RDVRAML $2139")]
        [InlineData(0x3F, "STAT78 $213F")]
        [InlineData(0x41, "APUI01 $2141")]
        [InlineData(0x7E, "APUI02 $217E")]
        [InlineData(0x83, "WMADDH $2183")]
        [InlineData(0x90, "$2190")]
        public void A_dma_destination_is_named_as_fullsnes_names_it(int register, string name)
        {
            Assert.Equal(name, SnesDebugViews.NameDmaDestination((byte)register));
            Assert.Equal(name, ((ISystemDebugViews)SnesSystem.Pack.DebugViews!).NameDmaDestination((byte)register));
        }

        // An HDMA channel enabled with the terminating 0 in its line counter has ended; a non-zero counter is still running.
        [Fact]
        public void An_hdma_channel_runs_while_its_line_counter_is_not_zero()
        {
            byte[] io = new byte[SnesDebugViews.IoSize];
            io[0x20C] = 0x03;
            io[0x30A] = 0x05;
            io[0x317] = 0x7F;
            io[0x315] = 0x34;
            io[0x316] = 0x12;
            var channels = SnesDebugViews.DmaChannels(DelegateDebugMemorySpace.Over("IO", io));
            Assert.Equal((true, true), (channels[0].HdmaEnabled, channels[0].HdmaActive));
            Assert.Equal((true, false), (channels[1].HdmaEnabled, channels[1].HdmaActive));
            Assert.Equal((0x1234, 0x7F1234), (channels[1].TransferSize, channels[1].IndirectAddress));
            Assert.Empty(SnesDebugViews.Instance.DmaChannels(new IDebugMemorySpace[] { DelegateDebugMemorySpace.Over("IO", new byte[0x10000]) }));
        }
    }
}
