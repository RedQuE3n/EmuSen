using EmuSen.Cores.Nintendo.Venus;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Ppu
{
    // Mode 0 gives each BG its own 32-colour CGRAM block - see Venus_PPU.md §14
    // for the Yoshi's Island intro text this was found on.
    public class Mode0PaletteBlockTests
    {
        // One 2bpp layer showing tile 1 with the given tilemap palette field,
        // every other layer off. bgIndex picks which of BG1-BG4 carries it.
        private static VenusCore BuildMode0Core(int bgIndex, int tilemapPalette)
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            var ppu = core.Bus!.Ppu;

            ppu.Inidisp = 0x0F;
            ppu.Bgmode = 0x00;
            ppu.Tm = (byte)(1 << bgIndex);
            ppu.Ts = 0x00;

            // Tilemap at VRAM 0x0000, character data at VRAM 0x2000, for
            // whichever layer is under test - the other three stay at 0 and
            // are not enabled in TM, so they cannot contribute a pixel.
            ppu.BgSc[bgIndex] = 0x00;
            if (bgIndex < 2) ppu.Bg12Nba = (byte)(0x01 << (bgIndex * 4));
            else ppu.Bg34Nba = (byte)(0x01 << ((bgIndex - 2) * 4));

            int entry = 0x0001 | (tilemapPalette << 10);
            ppu.Vram[0] = (byte)(entry & 0xFF);
            ppu.Vram[1] = (byte)(entry >> 8);

            // Tile 1, every row solid colour index 1 (bitplane 0 only). 2bpp
            // is a 16-byte stride, so tile 1 sits at base + 0x10, not + 0x20.
            for (int row = 0; row < 8; row++) ppu.Vram[0x2010 + row * 2] = 0xFF;

            return core;
        }

        private static void PaintCgram(VenusCore core, int entryIndex, ushort bgr555)
        {
            core.Bus!.Ppu.Cgram[entryIndex * 2] = (byte)(bgr555 & 0xFF);
            core.Bus.Ppu.Cgram[entryIndex * 2 + 1] = (byte)(bgr555 >> 8);
        }

        private static (byte R, byte G, byte B) FirstPixel(VenusCore core)
        {
            core.Renderer!.RenderScanline(core.Bus!, 0);
            byte[] rgba = core.Renderer.GetFrameBufferRgba();
            return (rgba[0], rgba[1], rgba[2]);
        }

        // BG1 0, BG2 32, BG3 64, BG4 96 - Mesen's RenderMode0 basePaletteOffset.
        [Theory]
        [InlineData(0, 0)]
        [InlineData(1, 32)]
        [InlineData(2, 64)]
        [InlineData(3, 96)]
        public void Each_layer_reads_its_own_32_colour_block(int bgIndex, int expectedBase)
        {
            const int Palette = 6;
            var core = BuildMode0Core(bgIndex, Palette);

            // Red only in this layer's block, green in BG1's - so a layer that
            // wrongly ignores its base lands on the green and is caught.
            PaintCgram(core, expectedBase + Palette * 4 + 1, 0x001F);
            if (expectedBase != 0) PaintCgram(core, Palette * 4 + 1, 0x03E0);

            Assert.Equal(((byte)248, (byte)0, (byte)0), FirstPixel(core));
        }

        // Outside Mode 0 the palette field indexes the bottom of CGRAM for
        // every layer, so the base must not leak into the other modes.
        [Fact]
        public void Mode_1_bg3_still_reads_the_bottom_of_cgram()
        {
            const int Palette = 6;
            var core = BuildMode0Core(bgIndex: 2, tilemapPalette: Palette);
            core.Bus!.Ppu.Bgmode = 0x01;

            PaintCgram(core, Palette * 4 + 1, 0x001F);
            PaintCgram(core, 64 + Palette * 4 + 1, 0x03E0);

            Assert.Equal(((byte)248, (byte)0, (byte)0), FirstPixel(core));
        }
    }
}
