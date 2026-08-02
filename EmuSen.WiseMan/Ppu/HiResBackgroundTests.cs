using EmuSen.Cores.Nintendo.Venus;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Ppu
{
    // Mode 5/6 true hi-res BG fetch - see Venus_PPU.md §8. A tilemap cell
    // covers 16 output dots supplied by a pair of 8x8 tiles (N, N+1), and
    // each output dot maps 1:1 onto a 512-dot BG column. Secret of Mana's
    // file-select menu is the game this was built against: it draws its
    // window frames and text entirely in Mode 5, and the pre-fix renderer
    // sampled 8 consecutive columns of a single tile per 8 output pixels,
    // which dropped half of every glyph.
    public class HiResBackgroundTests
    {
        private const int Red = 248;

        // Mode 5, BG1 on both screens. Tile 2 (the cell's left half) lights
        // only its leftmost column; tile 3 (the right half) is solid.
        private static VenusCore BuildMode5Core()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            var ppu = core.Bus!.Ppu;

            ppu.Inidisp = 0x0F;
            ppu.Bgmode = 0x05;    // Mode 5, BG1 tile size 8x8 (bit 4 clear)
            ppu.BgSc[0] = 0x00;   // BG1 tilemap at VRAM 0, 32x32
            ppu.Bg12Nba = 0x01;   // BG1 character data at VRAM 0x2000
            ppu.Tm = 0x01;        // BG1 on the main screen
            ppu.Ts = 0x01;        // BG1 on the sub screen too

            ppu.Cgram[2] = 0x1F;  // palette 0 colour 1 = red

            // Cell 0 -> tile 2; cell 1 is left as tile 0 (blank).
            ppu.Vram[0] = 0x02;
            ppu.Vram[1] = 0x00;

            // 4bpp stride 32: tile 2 at 0x2040, tile 3 at 0x2060.
            for (int row = 0; row < 8; row++)
            {
                ppu.Vram[0x2040 + row * 2] = 0x80; // leftmost column only
                ppu.Vram[0x2060 + row * 2] = 0xFF; // solid
            }

            return core;
        }

        private static byte[] RenderLine0(VenusCore core)
        {
            core.Renderer!.RenderScanline(core.Bus!, 0);
            return core.Renderer.GetFrameBufferRgba();
        }

        private static bool IsRed(byte[] rgba, int dot) =>
            rgba[dot * 4] == Red && rgba[dot * 4 + 1] == 0 && rgba[dot * 4 + 2] == 0;

        [Fact]
        public void Mode5_outputs_a_512_dot_frame()
        {
            var core = BuildMode5Core();
            RenderLine0(core);

            Assert.Equal(512, core.Renderer!.FrameWidth);
        }

        [Fact]
        public void Each_output_dot_maps_to_its_own_bg_column()
        {
            // Tile 2 contributes dots 0-7 and lit only its column 0, so
            // exactly one dot is lit in that half - not four, which is what
            // sampling every other column of the tile would produce.
            byte[] rgba = RenderLine0(BuildMode5Core());

            Assert.True(IsRed(rgba, 0));
            for (int dot = 1; dot < 8; dot++) Assert.False(IsRed(rgba, dot));
        }

        [Fact]
        public void The_paired_tile_supplies_the_second_half_of_the_cell()
        {
            // Tile 3 is solid, so dots 8-15 are all lit. The pre-fix code
            // gave the whole sub screen tile N+1 and the whole main screen
            // tile N, which lit every odd dot across the entire cell.
            byte[] rgba = RenderLine0(BuildMode5Core());

            for (int dot = 8; dot < 16; dot++) Assert.True(IsRed(rgba, dot));
        }

        [Fact]
        public void A_cell_covers_exactly_sixteen_dots()
        {
            // Cell 1 is tile 0 (blank), so nothing past dot 15 is lit.
            byte[] rgba = RenderLine0(BuildMode5Core());

            for (int dot = 16; dot < 32; dot++) Assert.False(IsRed(rgba, dot));
        }

        [Fact]
        public void Horizontal_scroll_is_expressed_in_hi_res_dots()
        {
            // BGnHOFS counts 256-space pixels, so one unit shifts the 512-dot
            // BG by two dots - the lit column moves from dot 0 to dot 14
            // (wrapping within the 16-dot cell is not involved; scrolling by
            // 1 pulls the cell left by 2 dots).
            var core = BuildMode5Core();
            core.Bus!.Ppu.BgScrollX[0] = 1;

            byte[] rgba = RenderLine0(core);

            Assert.False(IsRed(rgba, 0));
            for (int dot = 6; dot < 14; dot++) Assert.True(IsRed(rgba, dot));
        }

        [Fact]
        public void Mode1_is_unaffected_by_the_hi_res_path()
        {
            // The same tile data in Mode 1 must still render 8 output pixels
            // from tile 2 alone, at standard 256-wide resolution.
            var core = BuildMode5Core();
            core.Bus!.Ppu.Bgmode = 0x01;

            byte[] rgba = RenderLine0(core);

            Assert.Equal(256, core.Renderer!.FrameWidth);
            Assert.True(IsRed(rgba, 0));
            for (int dot = 1; dot < 8; dot++) Assert.False(IsRed(rgba, dot));
        }
    }
}
