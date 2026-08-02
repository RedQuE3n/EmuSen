using EmuSen.Cores.Nintendo.Venus.Memory.Mappers;
using EmuSen.WiseMan.Fixtures;
using Gsu = EmuSen.Cores.Nintendo.Venus.Coprocessors.SuperFx.SuperFx;

namespace EmuSen.WiseMan.Coprocessors
{
    // Detection, the S-CPU's view of a SuperFX cartridge, and the register
    // window - see Venus_SuperFX.md §1 and §3.
    public class SuperFxMemoryMapTests
    {
        private static Gsu Chip() => new(new byte[0x200000], new byte[0x20000]);

        private static (CartridgeRegion Region, int Offset) Map(Gsu gsu, byte bank, ushort offset)
        {
            var r = gsu.ResolveScpu(bank, offset);
            return (r.Region, r.Offset);
        }

        // --- Detection ---

        [Fact]
        public void Cartridge_type_1x_builds_the_gsu()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildSuperFx());
            Assert.NotNull(core.Cart!.SuperFx);
            Assert.Null(core.Cart.Sa1);
            Assert.Equal("SuperFX", core.Cart.MapperName);
        }

        [Fact]
        public void An_ordinary_lorom_gets_no_gsu()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.Build());
            Assert.Null(core.Cart!.SuperFx);
            Assert.Equal("LoROM", core.Cart.MapperName);
        }

        // The GSU can address 128KB, more than the header's expansion-RAM byte
        // usually admits to - see Venus_SuperFX.md §1.
        [Fact]
        public void Game_pak_ram_is_sized_for_the_whole_chip()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildSuperFx());
            Assert.Equal(0x20000, core.Cart!.SramSize);
        }

        // --- S-CPU address map ---

        [Fact]
        public void Low_banks_expose_rom_lorom_style()
        {
            Assert.Equal((CartridgeRegion.Rom, 0), Map(Chip(), 0x00, 0x8000));
            Assert.Equal((CartridgeRegion.Rom, 0x8000), Map(Chip(), 0x01, 0x8000));
        }

        [Fact]
        public void Banks_40_to_5f_expose_the_same_rom_linearly()
        {
            Assert.Equal((CartridgeRegion.Rom, 0), Map(Chip(), 0x40, 0x0000));
            Assert.Equal((CartridgeRegion.Rom, 0x10000), Map(Chip(), 0x41, 0x0000));
        }

        [Fact]
        public void Banks_c0_to_df_mirror_the_linear_rom_view()
        {
            var gsu = Chip();
            Assert.Equal(Map(gsu, 0x40, 0x1234), Map(gsu, 0xC0, 0x1234));
        }

        [Fact]
        public void Banks_80_to_bf_mirror_the_lorom_view()
        {
            var gsu = Chip();
            Assert.Equal(Map(gsu, 0x00, 0x8000), Map(gsu, 0x80, 0x8000));
        }

        [Fact]
        public void Banks_60_and_up_are_flat_game_pak_ram()
        {
            Assert.Equal((CartridgeRegion.Sram, 0), Map(Chip(), 0x60, 0x0000));
            Assert.Equal((CartridgeRegion.Sram, 0x10000), Map(Chip(), 0x61, 0x0000));
        }

        // The $6000-$7FFF window is the FIRST 8KB of Game Pak RAM mirrored into
        // every low bank - it does NOT advance per bank. This test previously
        // asserted the opposite, which is what let the bug through: Yoshi's
        // Island reads a jump-table index through $0F:6F0C, and a bank-indexed
        // window handed it a byte from 120KB further into RAM. See §5.1 and
        // §10.1 - it crashed the S-CPU at frame 1759.
        [Fact]
        public void The_scpu_ram_window_mirrors_the_first_8kb_into_every_bank()
        {
            Assert.Equal((CartridgeRegion.Sram, 0), Map(Chip(), 0x00, 0x6000));
            Assert.Equal((CartridgeRegion.Sram, 0), Map(Chip(), 0x01, 0x6000));
            Assert.Equal((CartridgeRegion.Sram, 0), Map(Chip(), 0x3F, 0x6000));

            // The offset within the window still tracks the address.
            Assert.Equal((CartridgeRegion.Sram, 0x0F0C), Map(Chip(), 0x0F, 0x6F0C));
            Assert.Equal((CartridgeRegion.Sram, 0x1FFF), Map(Chip(), 0x20, 0x7FFF));
        }

        [Fact]
        public void The_register_window_covers_3000_to_32ff()
        {
            Assert.Equal(CartridgeRegion.CoprocessorRegister, Map(Chip(), 0x00, 0x3000).Region);
            Assert.Equal(CartridgeRegion.CoprocessorRegister, Map(Chip(), 0x00, 0x32FF).Region);
            Assert.Equal(CartridgeRegion.Unmapped, Map(Chip(), 0x00, 0x3300).Region);
        }

        [Fact]
        public void Ppu_and_cpu_register_windows_are_left_to_the_bus()
        {
            Assert.Equal(CartridgeRegion.Unmapped, Map(Chip(), 0x00, 0x2100).Region);
            Assert.Equal(CartridgeRegion.Unmapped, Map(Chip(), 0x00, 0x4200).Region);
        }

        // --- Register file ---

        [Fact]
        public void The_register_file_reads_back_what_was_written()
        {
            var gsu = Chip();
            gsu.WriteRegister(0x3004, 0x34); // R2 low
            gsu.WriteRegister(0x3005, 0x12); // R2 high
            Assert.Equal(0x1234, gsu.R[2]);
            Assert.Equal(0x34, gsu.ReadRegister(0x3004));
            Assert.Equal(0x12, gsu.ReadRegister(0x3005));
        }

        // Writing R15's high byte is the launch trigger, so it must come last.
        [Fact]
        public void Writing_r15_high_starts_the_gsu()
        {
            var gsu = Chip();
            Assert.False(gsu.Running);
            gsu.WriteRegister(0x301E, 0x00);
            Assert.False(gsu.Running);
            gsu.WriteRegister(0x301F, 0x00);
            Assert.True(gsu.Running);
        }

        [Fact]
        public void The_scpu_cannot_disturb_registers_while_the_gsu_runs()
        {
            var gsu = Chip();
            gsu.WriteRegister(0x301F, 0x00); // start
            gsu.WriteRegister(0x3004, 0x99);
            Assert.Equal(0, gsu.R[2]);
        }

        [Fact]
        public void The_cache_is_writable_from_the_scpu()
        {
            var gsu = Chip();
            gsu.WriteRegister(0x3100, 0xAB);
            Assert.Equal(0xAB, gsu.ReadRegister(0x3100));
            Assert.Equal(0xAB, gsu.Cache[0]);
        }

        [Fact]
        public void The_version_register_reports_a_gsu_two()
        {
            Assert.Equal(0x04, Chip().ReadRegister(0x303B));
        }
    }
}
