using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.Mercury.Memory;
using EmuSen.WiseMan.Fixtures;
using GbPpu = EmuSen.Cores.Nintendo.Mercury.Video.Ppu;

namespace EmuSen.WiseMan.Cores
{
    // The Game Boy Color extensions on the same core - see Mercury_Cgb.md.
    public class MercuryCgbTests : IDisposable
    {
        private const int LineCycles = GbPpu.CyclesPerScanline;
        private const int DrawingEnd = GbPpu.OamScanCycles + GbPpu.BaseDrawingCycles;

        private readonly List<string> _temporaryFiles = new();

        public void Dispose()
        {
            foreach (string path in _temporaryFiles)
            {
                try { File.Delete(path); } catch (IOException) { }
            }
        }

        private MercuryCore Load(byte cgbFlag = 0xC0, params (int Offset, byte[] Bytes)[] patches)
        {
            string path = SyntheticGbRom.WriteTemp(SyntheticGbRom.Build(cgbFlag: cgbFlag, patches: patches));
            _temporaryFiles.Add(path);

            var core = new MercuryCore();
            core.LoadRom(path);
            return core;
        }

        private static (byte R, byte G, byte B) Pixel(MercuryCore core, int x, int y)
        {
            byte[] frame = core.GetFrameBufferRgba();
            int offset = (((y * GbPpu.ScreenWidth) + x) * 4);
            return (frame[offset], frame[offset + 1], frame[offset + 2]);
        }

        private static void FillTile(MercuryCore core, int bank, int tile, int color)
        {
            int origin = (bank * GbPpu.VramBankStride) + (tile * 16);

            for (int row = 0; row < 8; row++)
            {
                core.Bus!.Vram[origin + (row * 2)] = (byte)((color & 0x01) != 0 ? 0xFF : 0x00);
                core.Bus.Vram[origin + (row * 2) + 1] = (byte)((color & 0x02) != 0 ? 0xFF : 0x00);
            }
        }

        // Pushes one RGB555 entry into a palette through the auto-incrementing port.
        private static void SetPaletteEntry(MercuryCore core, ushort indexPort, int palette, int color, int rgb555)
        {
            core.Bus!.Write(indexPort, (byte)(0x80 | ((palette * 8) + (color * 2))));
            core.Bus.Write((ushort)(indexPort + 1), (byte)(rgb555 & 0xFF));
            core.Bus.Write((ushort)(indexPort + 1), (byte)(rgb555 >> 8));
        }

        private static void PutSprite(MercuryCore core, int slot, int y, int x, int tile, byte attributes)
        {
            core.Bus!.Oam[slot * 4] = (byte)y;
            core.Bus.Oam[(slot * 4) + 1] = (byte)x;
            core.Bus.Oam[(slot * 4) + 2] = (byte)tile;
            core.Bus.Oam[(slot * 4) + 3] = attributes;
        }

        private static void StepInstructions(MercuryCore core, int count)
        {
            for (int i = 0; i < count; i++)
            {
                int cycles = core.Cpu!.Step(core.Bus!.InterruptEnable, core.Bus.InterruptFlags, out int serviced);
                if (serviced >= 0) core.Bus.InterruptFlags &= (byte)~(1 << serviced);
                core.Bus.Tick(cycles);
            }
        }

        [Fact]
        public void A_colour_cartridge_gets_the_colour_name_and_the_extra_memory()
        {
            var core = Load();

            Assert.Equal("GBC", core.CoreName);
            Assert.Equal(0x4000, core.Bus!.Vram.Length);
            Assert.Equal(0x8000, core.Bus.Wram.Length);
        }

        [Fact]
        public void A_monochrome_cartridge_keeps_the_smaller_memory_and_the_old_name()
        {
            var core = Load(cgbFlag: 0x00);

            Assert.Equal("GB", core.CoreName);
            Assert.Equal(0x2000, core.Bus!.Vram.Length);
            Assert.Equal(0x2000, core.Bus.Wram.Length);
        }

        // A is the flag an enhanced cart branches on; get it wrong and the game picks the DMG path.
        [Fact]
        public void The_cpu_boots_with_the_register_file_a_colour_boot_rom_leaves()
        {
            var colour = Load();
            Assert.Equal(0x11, colour.Cpu!.A);
            Assert.Equal(0xFF56, colour.Cpu.DE);

            var monochrome = Load(cgbFlag: 0x00);
            Assert.Equal(0x01, monochrome.Cpu!.A);
        }

        // A CGB-enhanced cart runs in colour mode, the same as it would on the real console.
        [Fact]
        public void The_enhanced_flag_is_enough_to_enter_colour_mode()
        {
            var core = Load(cgbFlag: 0x80);
            Assert.Equal("GBC", core.CoreName);
        }

        [Fact]
        public void The_colour_registers_do_not_exist_on_a_monochrome_cartridge()
        {
            var core = Load(cgbFlag: 0x00);

            core.Bus!.Write(0xFF4F, 0x01);
            core.Bus.Write(0xFF70, 0x05);

            Assert.Equal(0, core.Bus.VramBank);
            Assert.Equal(1, core.Bus.WramBank);
        }

        [Fact]
        public void Vbk_selects_which_vram_bank_the_cpu_reaches()
        {
            var core = Load();

            core.Bus!.Write(0xFF4F, 0x00);
            core.Bus.Write(0x8000, 0x11);

            core.Bus.Write(0xFF4F, 0x01);
            core.Bus.Write(0x8000, 0x22);

            Assert.Equal(0x22, core.Bus.Read(0x8000));
            Assert.Equal(0x11, core.Bus.Vram[0x0000]);
            Assert.Equal(0x22, core.Bus.Vram[GbPpu.VramBankStride]);
            Assert.Equal(0xFF, core.Bus.Read(0xFF4F));
        }

        // Only the top half of WRAM is banked; $C000 is always bank 0.
        [Fact]
        public void Svbk_banks_the_upper_wram_window_and_treats_zero_as_one()
        {
            var core = Load();

            core.Bus!.Write(0xFF70, 0x01);
            core.Bus.Write(0xD000, 0xAA);
            core.Bus.Write(0xC000, 0x55);

            core.Bus.Write(0xFF70, 0x03);
            core.Bus.Write(0xD000, 0xBB);

            Assert.Equal(0x55, core.Bus.Read(0xC000));
            Assert.Equal(0xBB, core.Bus.Read(0xD000));

            core.Bus.Write(0xFF70, 0x00);
            Assert.Equal(1, core.Bus.WramBank);
            Assert.Equal(0xAA, core.Bus.Read(0xD000));
        }

        [Fact]
        public void The_palette_index_auto_increments_only_when_its_top_bit_is_set()
        {
            var core = Load();

            core.Bus!.Write(0xFF68, 0x80);
            core.Bus.Write(0xFF69, 0x01);
            core.Bus.Write(0xFF69, 0x02);

            Assert.Equal(0x01, core.Bus.Ppu.BgPaletteRam[0]);
            Assert.Equal(0x02, core.Bus.Ppu.BgPaletteRam[1]);

            core.Bus.Write(0xFF68, 0x00);
            core.Bus.Write(0xFF69, 0x09);
            core.Bus.Write(0xFF69, 0x0A);

            Assert.Equal(0x0A, core.Bus.Ppu.BgPaletteRam[0]);
            Assert.Equal(0x02, core.Bus.Ppu.BgPaletteRam[1]);
        }

        [Fact]
        public void A_background_tile_prints_through_its_colour_palette()
        {
            var core = Load();

            FillTile(core, bank: 0, tile: 1, color: 1);
            core.Bus!.Vram[0x1800] = 1;

            // Full red in RGB555, which expands to $FF rather than $F8.
            SetPaletteEntry(core, 0xFF68, palette: 0, color: 1, rgb555: 0x001F);

            core.Bus.Tick(LineCycles);
            Assert.Equal((0xFF, 0x00, 0x00), Pixel(core, 0, 0));
        }

        [Fact]
        public void The_map_attribute_selects_the_palette_the_bank_and_the_flips()
        {
            var core = Load();

            FillTile(core, bank: 1, tile: 1, color: 1);
            core.Bus!.Vram[0x1800] = 1;

            // Palette 3, tile data from bank 1.
            core.Bus.Vram[0x1800 + GbPpu.VramBankStride] = 0x0B;
            SetPaletteEntry(core, 0xFF68, palette: 3, color: 1, rgb555: 0x03E0);

            core.Bus.Tick(LineCycles);
            Assert.Equal((0x00, 0xFF, 0x00), Pixel(core, 0, 0));
        }

        [Fact]
        public void An_attribute_flip_mirrors_the_tile_in_both_axes()
        {
            var core = Load();

            // One opaque pixel, at column 0 of row 0.
            core.Bus!.Vram[16] = 0x80;
            core.Bus.Vram[0x1800] = 1;
            core.Bus.Vram[0x1801] = 1;

            core.Bus.Vram[0x1800 + GbPpu.VramBankStride] = 0x20;   // x flip
            core.Bus.Vram[0x1801 + GbPpu.VramBankStride] = 0x40;   // y flip

            SetPaletteEntry(core, 0xFF68, palette: 0, color: 0, rgb555: 0x0000);
            SetPaletteEntry(core, 0xFF68, palette: 0, color: 1, rgb555: 0x001F);

            core.Bus.Tick(LineCycles * 8);

            Assert.Equal((0xFF, 0x00, 0x00), Pixel(core, 7, 0));
            Assert.Equal((0x00, 0x00, 0x00), Pixel(core, 0, 0));
            Assert.Equal((0xFF, 0x00, 0x00), Pixel(core, 8, 7));
        }

        // The attribute's own priority bit hides a sprite that did not ask to be behind anything.
        [Fact]
        public void A_background_attribute_can_claim_priority_a_sprite_never_yielded()
        {
            var core = Load();
            core.Bus!.Write(0xFF40, 0x93);

            FillTile(core, bank: 0, tile: 1, color: 1);
            FillTile(core, bank: 0, tile: 2, color: 2);
            core.Bus.Vram[0x1800] = 1;
            core.Bus.Vram[0x1800 + GbPpu.VramBankStride] = 0x80;

            SetPaletteEntry(core, 0xFF68, palette: 0, color: 1, rgb555: 0x001F);
            SetPaletteEntry(core, 0xFF6A, palette: 0, color: 2, rgb555: 0x03E0);
            PutSprite(core, slot: 0, y: 16, x: 8, tile: 2, attributes: 0x00);

            core.Bus.Tick(LineCycles);
            Assert.Equal((0xFF, 0x00, 0x00), Pixel(core, 0, 0));
        }

        // LCDC bit 0 stops blanking the background and starts waiving its priority instead.
        [Fact]
        public void Clearing_lcdc_bit_zero_waives_background_priority_rather_than_blanking()
        {
            var core = Load();

            FillTile(core, bank: 0, tile: 1, color: 1);
            FillTile(core, bank: 0, tile: 2, color: 2);
            core.Bus!.Vram[0x1800] = 1;
            core.Bus.Vram[0x1801] = 1;
            core.Bus.Vram[0x1800 + GbPpu.VramBankStride] = 0x80;
            core.Bus.Vram[0x1801 + GbPpu.VramBankStride] = 0x80;

            SetPaletteEntry(core, 0xFF68, palette: 0, color: 1, rgb555: 0x001F);
            SetPaletteEntry(core, 0xFF6A, palette: 0, color: 2, rgb555: 0x03E0);
            PutSprite(core, slot: 0, y: 16, x: 8, tile: 2, attributes: 0x00);

            core.Bus.Write(0xFF40, 0x92);   // sprites on, BG priority waived
            core.Bus.Tick(LineCycles);

            // The sprite takes the pixels it covers, and the background still draws the ones it does not.
            Assert.Equal((0x00, 0xFF, 0x00), Pixel(core, 0, 0));
            Assert.Equal((0xFF, 0x00, 0x00), Pixel(core, 8, 0));
        }

        [Fact]
        public void A_sprite_takes_its_palette_and_its_tile_bank_from_the_oam_attribute()
        {
            var core = Load();
            core.Bus!.Write(0xFF40, 0x93);

            FillTile(core, bank: 1, tile: 1, color: 3);
            SetPaletteEntry(core, 0xFF6A, palette: 5, color: 3, rgb555: 0x7C00);
            PutSprite(core, slot: 0, y: 16, x: 8, tile: 1, attributes: 0x0D);

            core.Bus.Tick(LineCycles);
            Assert.Equal((0x00, 0x00, 0xFF), Pixel(core, 0, 0));
        }

        // On a CGB the OAM index decides overlap outright; X is not consulted at all.
        [Fact]
        public void Sprite_overlap_is_resolved_by_oam_index_not_by_x()
        {
            var core = Load();
            core.Bus!.Write(0xFF40, 0x93);

            FillTile(core, bank: 0, tile: 1, color: 1);
            FillTile(core, bank: 0, tile: 2, color: 2);

            SetPaletteEntry(core, 0xFF6A, palette: 0, color: 1, rgb555: 0x001F);
            SetPaletteEntry(core, 0xFF6A, palette: 0, color: 2, rgb555: 0x03E0);

            PutSprite(core, slot: 0, y: 16, x: 8 + 20, tile: 1, attributes: 0x00);
            PutSprite(core, slot: 1, y: 16, x: 8 + 16, tile: 2, attributes: 0x00);

            core.Bus.Tick(LineCycles);

            // The DMG answer here would be green: sprite 1 is further left.
            Assert.Equal((0xFF, 0x00, 0x00), Pixel(core, 20, 0));
            Assert.Equal((0x00, 0xFF, 0x00), Pixel(core, 16, 0));
        }

        [Fact]
        public void A_general_purpose_hdma_copies_at_once_and_reports_itself_idle()
        {
            var core = Load();

            for (int i = 0; i < 32; i++) core.Bus!.Write((ushort)(0xC000 + i), (byte)(i ^ 0x3C));

            core.Bus!.Write(0xFF51, 0xC0);
            core.Bus.Write(0xFF52, 0x00);
            core.Bus.Write(0xFF53, 0x00);
            core.Bus.Write(0xFF54, 0x10);   // destination $8010
            core.Bus.Write(0xFF55, 0x01);   // two blocks, immediate

            for (int i = 0; i < 32; i++) Assert.Equal((byte)(i ^ 0x3C), core.Bus.Vram[0x0010 + i]);
            Assert.Equal(0xFF, core.Bus.Read(0xFF55));
        }

        [Fact]
        public void An_hblank_hdma_moves_one_block_per_line_and_counts_down()
        {
            var core = Load();

            for (int i = 0; i < 48; i++) core.Bus!.Write((ushort)(0xC000 + i), (byte)(i + 1));

            core.Bus!.Write(0xFF51, 0xC0);
            core.Bus.Write(0xFF52, 0x00);
            core.Bus.Write(0xFF53, 0x00);
            core.Bus.Write(0xFF54, 0x00);
            core.Bus.Write(0xFF55, 0x82);   // three blocks, hblank-driven

            Assert.Equal(0, core.Bus.Vram[0]);
            Assert.Equal(0x02, core.Bus.Read(0xFF55));

            core.Bus.Tick(DrawingEnd);
            Assert.Equal(0x01, core.Bus.Vram[0]);
            Assert.Equal(0, core.Bus.Vram[16]);
            Assert.Equal(0x01, core.Bus.Read(0xFF55));

            core.Bus.Tick(LineCycles);
            Assert.Equal(0x11, core.Bus.Vram[16]);

            core.Bus.Tick(LineCycles);
            Assert.Equal(0x21, core.Bus.Vram[32]);
            Assert.Equal(0xFF, core.Bus.Read(0xFF55));
        }

        [Fact]
        public void Clearing_bit_seven_cancels_a_running_hblank_hdma()
        {
            var core = Load();

            core.Bus!.Write(0xFF51, 0xC0);
            core.Bus.Write(0xFF55, 0x82);

            core.Bus.Write(0xFF55, 0x00);

            Assert.Equal(0xFF, core.Bus.Read(0xFF55));
            Assert.Equal(0, core.Bus.HdmaBlocksLeft);
        }

        [Fact]
        public void Stop_performs_the_speed_switch_key1_armed()
        {
            // LD A,$01 / LDH ($FF4D),A / STOP
            var core = Load(0xC0, (0, new byte[] { 0x3E, 0x01, 0xE0, 0x4D, 0x10, 0x00 }));

            // Reset lands at $0100, so the header's NOP and JP run before the patch does.
            StepInstructions(core, 4);
            Assert.True(core.Bus!.SpeedSwitchArmed);
            Assert.Equal(0x01, core.Bus.Read(0xFF4D) & 0x01);

            StepInstructions(core, 1);
            Assert.True(core.Bus.DoubleSpeed);
            Assert.False(core.Bus.SpeedSwitchArmed);
            Assert.Equal(0x80, core.Bus.Read(0xFF4D) & 0x80);
        }

        // The CPU doubles and the LCD does not, which is the entire reason the mode exists.
        [Fact]
        public void Double_speed_halves_the_lcd_clock_relative_to_the_cpu()
        {
            var core = Load();
            core.Bus!.Tick(LineCycles);
            Assert.Equal(1, core.Bus.Ppu.Ly);

            core.Bus.Write(0xFF4D, 0x01);
            core.Bus.Stop();
            Assert.True(core.Bus.DoubleSpeed);

            core.Bus.Tick(LineCycles * 2);
            Assert.Equal(2, core.Bus.Ppu.Ly);
        }

        [Fact]
        public void A_save_state_round_trips_the_colour_state()
        {
            var core = Load();

            core.Bus!.Write(0xFF4F, 0x01);
            core.Bus.Write(0xFF70, 0x06);
            core.Bus.Write(0xFF4D, 0x01);
            core.Bus.Stop();
            SetPaletteEntry(core, 0xFF68, palette: 2, color: 1, rgb555: 0x1234);

            using var stream = new MemoryStream();
            core.SaveState(stream);

            core.Bus.Write(0xFF4F, 0x00);
            core.Bus.Write(0xFF70, 0x01);
            core.Bus.Stop();
            SetPaletteEntry(core, 0xFF68, palette: 2, color: 1, rgb555: 0x0000);

            stream.Position = 0;
            core.LoadState(stream);

            Assert.Equal(1, core.Bus.VramBank);
            Assert.Equal(6, core.Bus.WramBank);
            Assert.True(core.Bus.DoubleSpeed);
            Assert.Equal(0x34, core.Bus.Ppu.BgPaletteRam[(2 * 8) + 2]);
            Assert.Equal(0x12, core.Bus.Ppu.BgPaletteRam[(2 * 8) + 3]);
        }
    }
}
