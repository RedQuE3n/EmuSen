using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.Mercury.Memory;
using EmuSen.Cores.Nintendo.Mercury.Video;
using GbPpu = EmuSen.Cores.Nintendo.Mercury.Video.Ppu;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Mercury's LCD controller - see Mercury_Ppu.md.
    public class MercuryPpuTests : IDisposable
    {
        private const int LineCycles = GbPpu.CyclesPerScanline;

        // Where mode 3 ends with SCX zero, which is when the line is composited.
        private const int DrawingEnd = GbPpu.OamScanCycles + GbPpu.BaseDrawingCycles;

        private const byte White = 0xFF;
        private const byte LightGrey = 0xAA;
        private const byte DarkGrey = 0x55;
        private const byte Black = 0x00;

        // Identity: colour n maps to shade n.
        private const byte IdentityPalette = 0xE4;

        private readonly List<string> _temporaryFiles = new();

        public void Dispose()
        {
            foreach (string path in _temporaryFiles)
            {
                try { File.Delete(path); } catch (IOException) { }
            }
        }

        private MercuryCore Load()
        {
            string path = SyntheticGbRom.WriteTemp(SyntheticGbRom.Build());
            _temporaryFiles.Add(path);

            var core = new MercuryCore();
            core.LoadRom(path);
            return core;
        }

        private static byte Pixel(MercuryCore core, int x, int y) =>
            core.GetFrameBufferRgba()[(((y * GbPpu.ScreenWidth) + x) * 4)];

        // Every row of one tile set to a single colour, written at the $8000-relative slot.
        private static void FillTile(MercuryCore core, int tile, int color)
        {
            for (int row = 0; row < 8; row++)
            {
                core.Bus!.Vram[(tile * 16) + (row * 2)] = (byte)((color & 0x01) != 0 ? 0xFF : 0x00);
                core.Bus.Vram[(tile * 16) + (row * 2) + 1] = (byte)((color & 0x02) != 0 ? 0xFF : 0x00);
            }
        }

        private static void PutSprite(MercuryCore core, int slot, int y, int x, int tile, byte attributes)
        {
            core.Bus!.Oam[slot * 4] = (byte)y;
            core.Bus.Oam[(slot * 4) + 1] = (byte)x;
            core.Bus.Oam[(slot * 4) + 2] = (byte)tile;
            core.Bus.Oam[(slot * 4) + 3] = attributes;
        }

        [Fact]
        public void A_scanline_walks_oam_scan_then_drawing_then_hblank()
        {
            var core = Load();

            Assert.Equal(PpuMode.OamScan, core.Bus!.Ppu.Mode);

            core.Bus.Tick(GbPpu.OamScanCycles);
            Assert.Equal(PpuMode.Drawing, core.Bus.Ppu.Mode);

            core.Bus.Tick(DrawingEnd - GbPpu.OamScanCycles);
            Assert.Equal(PpuMode.HBlank, core.Bus.Ppu.Mode);

            core.Bus.Tick(LineCycles - DrawingEnd);
            Assert.Equal(1, core.Bus.Ppu.Ly);
            Assert.Equal(PpuMode.OamScan, core.Bus.Ppu.Mode);
        }

        // SCX's low three bits are the fine scroll, and hardware pays for them out of mode 3.
        [Fact]
        public void Fine_scroll_lengthens_mode_three_and_shortens_hblank()
        {
            var core = Load();
            core.Bus!.Write(0xFF43, 0x05);

            core.Bus.Tick(DrawingEnd);
            Assert.Equal(PpuMode.Drawing, core.Bus.Ppu.Mode);

            core.Bus.Tick(5);
            Assert.Equal(PpuMode.HBlank, core.Bus.Ppu.Mode);
        }

        [Fact]
        public void Vblank_is_raised_when_ly_reaches_144()
        {
            var core = Load();

            core.Bus!.Tick(LineCycles * 143);
            Assert.Equal(143, core.Bus.Ppu.Ly);
            Assert.Equal(0, core.Bus.InterruptFlags & (1 << (int)Interrupt.VBlank));

            core.Bus.Tick(LineCycles);
            Assert.Equal(144, core.Bus.Ppu.Ly);
            Assert.Equal(PpuMode.VBlank, core.Bus.Ppu.Mode);
            Assert.NotEqual(0, core.Bus.InterruptFlags & (1 << (int)Interrupt.VBlank));
        }

        [Fact]
        public void A_frame_is_154_lines_and_wraps_ly_to_zero()
        {
            var core = Load();

            core.Bus!.Tick(LineCycles * GbPpu.TotalScanlines);

            Assert.Equal(0, core.Bus.Ppu.Ly);
            Assert.Equal(1, core.Bus.Ppu.FrameCount);
            Assert.True(core.Bus.Ppu.FrameComplete);
        }

        [Fact]
        public void Stat_reports_the_mode_the_lyc_match_and_its_unwired_top_bit()
        {
            var core = Load();
            core.Bus!.Write(0xFF45, 0x00);

            byte stat = core.Bus.Read(0xFF41);
            Assert.Equal(0x80, stat & 0x80);
            Assert.Equal(0x04, stat & 0x04);
            Assert.Equal((int)PpuMode.OamScan, stat & 0x03);

            core.Bus.Tick(LineCycles);
            Assert.Equal(0x00, core.Bus.Read(0xFF41) & 0x04);
        }

        // Enabling a second source while the line is already high must not request again.
        [Fact]
        public void The_stat_interrupt_only_fires_on_a_rising_edge()
        {
            var core = Load();

            core.Bus!.Write(0xFF45, 0x00);   // LYC = 0, which LY already matches
            core.Bus.Write(0xFF41, 0x48);    // LYC and hblank sources both enabled
            core.Bus.InterruptFlags = 0;

            core.Bus.Tick(DrawingEnd);
            Assert.Equal(PpuMode.HBlank, core.Bus.Ppu.Mode);
            Assert.Equal(0, core.Bus.InterruptFlags & (1 << (int)Interrupt.LcdStat));
        }

        [Fact]
        public void The_stat_interrupt_fires_when_hblank_is_the_only_source_held_high()
        {
            var core = Load();

            core.Bus!.Write(0xFF45, 0x05);   // LYC far from LY, so nothing else holds the line
            core.Bus.Write(0xFF41, 0x48);
            core.Bus.InterruptFlags = 0;

            core.Bus.Tick(DrawingEnd);
            Assert.NotEqual(0, core.Bus.InterruptFlags & (1 << (int)Interrupt.LcdStat));
        }

        [Fact]
        public void A_lyc_match_requests_stat_when_that_source_is_enabled()
        {
            var core = Load();

            core.Bus!.Write(0xFF45, 0x03);
            core.Bus.Write(0xFF41, 0x40);
            core.Bus.InterruptFlags = 0;

            core.Bus.Tick(LineCycles * 3);
            Assert.Equal(3, core.Bus.Ppu.Ly);
            Assert.NotEqual(0, core.Bus.InterruptFlags & (1 << (int)Interrupt.LcdStat));
        }

        [Fact]
        public void The_background_draws_a_tile_through_bgp()
        {
            var core = Load();
            core.Bus!.Write(0xFF47, IdentityPalette);

            FillTile(core, tile: 1, color: 1);
            core.Bus.Vram[0x1800] = 1;

            core.Bus.Tick(LineCycles);

            Assert.Equal(LightGrey, Pixel(core, 0, 0));
            Assert.Equal(LightGrey, Pixel(core, 7, 0));
            Assert.Equal(White, Pixel(core, 8, 0));
        }

        [Fact]
        public void Bgp_remaps_the_shade_a_colour_index_prints_as()
        {
            var core = Load();
            core.Bus!.Write(0xFF47, 0x1B);   // 00 01 10 11: the identity mapping reversed

            FillTile(core, tile: 1, color: 1);
            core.Bus.Vram[0x1800] = 1;

            core.Bus.Tick(LineCycles);

            Assert.Equal(DarkGrey, Pixel(core, 0, 0));
            Assert.Equal(Black, Pixel(core, 8, 0));
        }

        [Fact]
        public void Scx_and_scy_scroll_the_map_under_the_window_of_the_screen()
        {
            var core = Load();
            core.Bus!.Write(0xFF47, IdentityPalette);

            FillTile(core, tile: 1, color: 3);
            core.Bus.Vram[0x1800 + 32] = 1;   // map row 1, column 0

            // Row 1 of the map is off screen until SCY brings it up to line 0.
            core.Bus.Tick(LineCycles);
            Assert.Equal(White, Pixel(core, 0, 0));

            core.Bus.Write(0xFF42, 8);
            core.Bus.Write(0xFF43, 4);
            core.Bus.Tick(LineCycles * (GbPpu.TotalScanlines - 1));
            core.Bus.Tick(LineCycles);

            // SCX 4 slides the tile four pixels left, so it occupies x 0-3 and nothing after.
            Assert.Equal(Black, Pixel(core, 3, 0));
            Assert.Equal(White, Pixel(core, 4, 0));
        }

        // LCDC bit 4 clear moves the tile base to $9000 and reads the index as signed.
        [Fact]
        public void Signed_tile_addressing_selects_from_the_middle_block()
        {
            var core = Load();
            core.Bus!.Write(0xFF47, IdentityPalette);
            core.Bus.Write(0xFF40, 0x81);   // LCD on, BG on, tile data at $8800

            for (int row = 0; row < 8; row++) core.Bus.Vram[0x1000 + (row * 2)] = 0xFF;
            core.Bus.Vram[0x1800] = 0;

            core.Bus.Tick(LineCycles);
            Assert.Equal(LightGrey, Pixel(core, 0, 0));
        }

        [Fact]
        public void Clearing_lcdc_bit_zero_blanks_the_background_to_white()
        {
            var core = Load();
            core.Bus!.Write(0xFF47, IdentityPalette);

            FillTile(core, tile: 1, color: 3);
            core.Bus.Vram[0x1800] = 1;

            core.Bus.Write(0xFF40, 0x90);   // same as boot, minus BG enable
            core.Bus.Tick(LineCycles);

            Assert.Equal(White, Pixel(core, 0, 0));
        }

        [Fact]
        public void The_window_draws_from_wx_minus_seven_over_the_background()
        {
            var core = Load();
            core.Bus!.Write(0xFF47, IdentityPalette);

            FillTile(core, tile: 1, color: 1);
            FillTile(core, tile: 2, color: 3);

            for (int i = 0; i < 32; i++) core.Bus.Vram[0x1800 + i] = 1;
            core.Bus.Vram[0x1C00] = 2;

            core.Bus.Write(0xFF4A, 0);      // WY
            core.Bus.Write(0xFF4B, 15);     // WX, so the window starts at x = 8
            core.Bus.Write(0xFF40, 0xF1);   // LCD, window map $9C00, window on, $8000 data, BG map $9800, BG on

            core.Bus.Tick(LineCycles);

            Assert.Equal(LightGrey, Pixel(core, 7, 0));
            Assert.Equal(Black, Pixel(core, 8, 0));
        }

        // The window has its own line counter, so a late WY still starts the window at its row 0.
        [Fact]
        public void The_window_line_counter_starts_at_zero_on_the_line_wy_selects()
        {
            var core = Load();
            core.Bus!.Write(0xFF47, IdentityPalette);

            FillTile(core, tile: 2, color: 3);
            core.Bus.Vram[0x1C00] = 2;      // window map row 0 only

            core.Bus.Write(0xFF4A, 4);      // WY
            core.Bus.Write(0xFF4B, 7);      // WX, so the window starts at x = 0
            core.Bus.Write(0xFF40, 0xF1);

            core.Bus.Tick(LineCycles * 5);

            Assert.Equal(White, Pixel(core, 0, 3));
            Assert.Equal(Black, Pixel(core, 0, 4));
            Assert.Equal(1, core.Bus.Ppu.WindowLine);
        }

        [Fact]
        public void A_sprite_draws_at_its_oam_position_through_its_own_palette()
        {
            var core = Load();
            core.Bus!.Write(0xFF48, IdentityPalette);   // OBP0
            core.Bus.Write(0xFF40, 0x93);               // sprites enabled

            FillTile(core, tile: 1, color: 2);
            PutSprite(core, slot: 0, y: 16, x: 8 + 24, tile: 1, attributes: 0x00);

            core.Bus.Tick(LineCycles);

            Assert.Equal(White, Pixel(core, 23, 0));
            Assert.Equal(DarkGrey, Pixel(core, 24, 0));
            Assert.Equal(DarkGrey, Pixel(core, 31, 0));
            Assert.Equal(White, Pixel(core, 32, 0));
        }

        [Fact]
        public void Sprite_colour_zero_is_transparent_rather_than_a_shade()
        {
            var core = Load();
            core.Bus!.Write(0xFF47, IdentityPalette);
            core.Bus.Write(0xFF48, 0x1B);               // OBP0 maps colour 0 to black, if it were ever drawn
            core.Bus.Write(0xFF40, 0x93);

            FillTile(core, tile: 1, color: 1);
            FillTile(core, tile: 2, color: 0);
            core.Bus.Vram[0x1800] = 1;
            PutSprite(core, slot: 0, y: 16, x: 8, tile: 2, attributes: 0x00);

            core.Bus.Tick(LineCycles);

            Assert.Equal(LightGrey, Pixel(core, 0, 0));
        }

        [Fact]
        public void The_x_flip_and_y_flip_attributes_mirror_the_tile()
        {
            var core = Load();
            core.Bus!.Write(0xFF48, IdentityPalette);
            core.Bus.Write(0xFF40, 0x93);

            // Column 0 opaque on row 0 only, so both flips are separately observable.
            core.Bus.Vram[16] = 0x80;

            PutSprite(core, slot: 0, y: 16, x: 8, tile: 1, attributes: 0x00);
            PutSprite(core, slot: 1, y: 16, x: 8 + 16, tile: 1, attributes: 0x20);
            PutSprite(core, slot: 2, y: 16, x: 8 + 32, tile: 1, attributes: 0x40);

            core.Bus.Tick(LineCycles);

            Assert.Equal(LightGrey, Pixel(core, 0, 0));
            Assert.Equal(LightGrey, Pixel(core, 16 + 7, 0));
            Assert.Equal(White, Pixel(core, 32, 0));

            // The y-flipped sprite's opaque row moved to the bottom of the eight.
            core.Bus.Tick(LineCycles * 7);
            Assert.Equal(LightGrey, Pixel(core, 32, 7));
        }

        [Fact]
        public void An_eight_by_sixteen_sprite_spans_two_tiles_and_ignores_the_tile_low_bit()
        {
            var core = Load();
            core.Bus!.Write(0xFF48, IdentityPalette);
            core.Bus.Write(0xFF40, 0x97);   // sprites enabled, 8x16

            FillTile(core, tile: 2, color: 1);
            FillTile(core, tile: 3, color: 3);
            PutSprite(core, slot: 0, y: 16, x: 8, tile: 3, attributes: 0x00);

            core.Bus.Tick(LineCycles * 16);

            Assert.Equal(LightGrey, Pixel(core, 0, 7));
            Assert.Equal(Black, Pixel(core, 0, 8));
            Assert.Equal(Black, Pixel(core, 0, 15));
        }

        // On a DMG the leftmost sprite wins the pixel outright; the loser is not drawn behind it.
        [Fact]
        public void The_lower_x_sprite_wins_an_overlapping_pixel()
        {
            var core = Load();
            core.Bus!.Write(0xFF48, IdentityPalette);
            core.Bus.Write(0xFF40, 0x93);

            FillTile(core, tile: 1, color: 3);
            FillTile(core, tile: 2, color: 1);

            PutSprite(core, slot: 0, y: 16, x: 8 + 20, tile: 1, attributes: 0x00);
            PutSprite(core, slot: 1, y: 16, x: 8 + 16, tile: 2, attributes: 0x00);

            core.Bus.Tick(LineCycles);

            Assert.Equal(LightGrey, Pixel(core, 20, 0));
            Assert.Equal(Black, Pixel(core, 24, 0));
        }

        [Fact]
        public void Equal_x_falls_back_to_the_lower_oam_index()
        {
            var core = Load();
            core.Bus!.Write(0xFF48, IdentityPalette);
            core.Bus.Write(0xFF40, 0x93);

            FillTile(core, tile: 1, color: 3);
            FillTile(core, tile: 2, color: 1);

            PutSprite(core, slot: 0, y: 16, x: 8, tile: 1, attributes: 0x00);
            PutSprite(core, slot: 1, y: 16, x: 8, tile: 2, attributes: 0x00);

            core.Bus.Tick(LineCycles);

            Assert.Equal(Black, Pixel(core, 0, 0));
        }

        // The cut is by OAM index, before any priority sort, so a low X cannot buy a slot.
        [Fact]
        public void Only_the_first_ten_sprites_on_a_line_are_drawn()
        {
            var core = Load();
            core.Bus!.Write(0xFF48, IdentityPalette);
            core.Bus.Write(0xFF40, 0x93);

            FillTile(core, tile: 1, color: 3);

            for (int i = 0; i < 10; i++) PutSprite(core, i, y: 16, x: 8 + 16 + (i * 8), tile: 1, attributes: 0x00);
            PutSprite(core, slot: 10, y: 16, x: 8, tile: 1, attributes: 0x00);

            core.Bus.Tick(LineCycles);

            Assert.Equal(White, Pixel(core, 0, 0));
            Assert.Equal(Black, Pixel(core, 16, 0));
        }

        [Fact]
        public void The_priority_attribute_puts_a_sprite_behind_opaque_background_only()
        {
            var core = Load();
            core.Bus!.Write(0xFF47, IdentityPalette);
            core.Bus.Write(0xFF48, IdentityPalette);
            core.Bus.Write(0xFF40, 0x93);

            FillTile(core, tile: 1, color: 1);
            FillTile(core, tile: 2, color: 0);
            FillTile(core, tile: 3, color: 3);

            core.Bus.Vram[0x1800] = 1;      // x 0-7 background colour 1
            core.Bus.Vram[0x1801] = 2;      // x 8-15 background colour 0
            PutSprite(core, slot: 0, y: 16, x: 8 + 4, tile: 3, attributes: 0x80);

            core.Bus.Tick(LineCycles);

            Assert.Equal(LightGrey, Pixel(core, 4, 0));
            Assert.Equal(Black, Pixel(core, 8, 0));
        }

        [Fact]
        public void Turning_the_lcd_off_parks_ly_and_whitens_the_panel()
        {
            var core = Load();
            core.Bus!.Write(0xFF47, IdentityPalette);

            FillTile(core, tile: 1, color: 3);
            core.Bus.Vram[0x1800] = 1;

            core.Bus.Tick(LineCycles * 3);
            Assert.Equal(Black, Pixel(core, 0, 0));

            core.Bus.Write(0xFF40, 0x11);
            Assert.Equal(0, core.Bus.Ppu.Ly);
            Assert.Equal(White, Pixel(core, 0, 0));

            // A disabled controller does not advance, whatever the clock does.
            core.Bus.Tick(LineCycles * 3);
            Assert.Equal(0, core.Bus.Ppu.Ly);
        }

        [Fact]
        public void Run_frame_ends_on_the_ppu_boundary_with_ly_back_at_zero()
        {
            var core = Load();
            core.RunFrame();

            Assert.Equal(1, core.TotalFrames);
            Assert.Equal(0, core.Bus!.Ppu.Ly);
            Assert.Equal(1, core.Bus.Ppu.FrameCount);
        }

        // A game that leaves the LCD off still has to get frames out of RunFrame.
        [Fact]
        public void Run_frame_still_returns_with_the_lcd_disabled()
        {
            var core = Load();
            core.Bus!.Write(0xFF40, 0x11);

            core.RunFrame();

            Assert.Equal(1, core.TotalFrames);
            Assert.Equal(0, core.Bus.Ppu.FrameCount);
        }

        [Fact]
        public void A_save_state_round_trips_the_ppu_position()
        {
            var core = Load();
            core.Bus!.Tick(LineCycles * 40);

            using var stream = new MemoryStream();
            core.SaveState(stream);
            byte lyAtSave = core.Bus.Ppu.Ly;

            core.Bus.Tick(LineCycles * 10);
            Assert.NotEqual(lyAtSave, core.Bus.Ppu.Ly);

            stream.Position = 0;
            core.LoadState(stream);

            Assert.Equal(lyAtSave, core.Bus.Ppu.Ly);
        }
    }
}
