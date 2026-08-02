using EmuSen.Cores.Nintendo.Venus;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Ppu
{
    // The BG row shown on display row N is N+1+VOFS, not N+VOFS - the first
    // visible scanline is hardware line 1, not 0 (see Venus_PPU.md §2.1).
    // Donkey Kong Country 2's Pirate Panic is the scene that exposed it: it
    // parks BG2 at VOFS $3FF (one less than the map height, i.e. -1) so the
    // hardware +1 lands it on BG row 0. Without the +1 the top display row
    // wrapped to the map's last row instead, drawing one line of unrelated
    // tilemap across the top of the screen.
    public class BackgroundVerticalOffsetTests
    {
        private const int Red = 248;

        // Mode 1, BG1 8x8 4bpp, 32x32 tilemap at VRAM 0, characters at $2000.
        private static VenusCore BuildCore()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            var ppu = core.Bus!.Ppu;

            ppu.Inidisp = 0x0F;
            ppu.Bgmode = 0x01;
            ppu.BgSc[0] = 0x00;
            ppu.Bg12Nba = 0x01;
            ppu.Tm = 0x01;

            ppu.Cgram[2] = 0x1F; // palette 0 colour 1 = red
            ppu.Cgram[3] = 0x00;

            return core;
        }

        // 4bpp stride 32, plane 0 at byte row*2 - a lit row is colour 1 across.
        private static void LightTileRow(VenusCore core, int tile, int row) =>
            core.Bus!.Ppu.Vram[0x2000 + tile * 32 + row * 2] = 0xFF;

        // 32x32 tilemap: entry index is ty*32 + tx, two bytes each.
        private static void PlaceTile(VenusCore core, int tx, int ty, int tile)
        {
            int addr = (ty * 32 + tx) * 2;
            core.Bus!.Ppu.Vram[addr] = (byte)tile;
            core.Bus.Ppu.Vram[addr + 1] = 0x00;
        }

        private static bool DisplayRow0IsRed(VenusCore core)
        {
            core.Renderer!.RenderScanline(core.Bus!, 0);
            byte[] rgba = core.Renderer.GetFrameBufferRgba();
            return rgba[0] == Red && rgba[1] == 0 && rgba[2] == 0;
        }

        [Fact]
        public void Display_row_0_shows_the_bg_row_one_past_vofs()
        {
            var core = BuildCore();
            PlaceTile(core, 0, 0, 1);
            LightTileRow(core, 1, 3); // only row 3 of the tile is lit

            core.Bus!.Ppu.BgScrollY[0] = 2; // 0 + 1 + 2 = BG row 3

            Assert.True(DisplayRow0IsRed(core));
        }

        [Fact]
        public void A_vofs_landing_on_the_lit_row_without_the_offset_does_not_show_it()
        {
            var core = BuildCore();
            PlaceTile(core, 0, 0, 1);
            LightTileRow(core, 1, 3);

            core.Bus!.Ppu.BgScrollY[0] = 3; // the pre-fix formula's answer - now BG row 4

            Assert.False(DisplayRow0IsRed(core));
        }

        // The DKC2 case: VOFS = mapH-1 is how a game asks for BG row 0 at the
        // top of the screen, since hardware adds the 1 back.
        [Fact]
        public void A_vofs_of_minus_one_wraps_forward_to_bg_row_0()
        {
            var core = BuildCore();
            PlaceTile(core, 0, 0, 1);      // BG row 0-7
            PlaceTile(core, 0, 31, 2);     // BG row 248-255, the map's last tile row
            LightTileRow(core, 1, 0);      // tile 1 lights its row 0
            LightTileRow(core, 2, 3);      // tile 2 lights row 3, NOT the row 7 that BG row 255 would hit

            core.Bus!.Ppu.BgScrollY[0] = 255; // 0 + 1 + 255 = 256 -> wraps to BG row 0

            Assert.True(DisplayRow0IsRed(core));
        }
    }
}
