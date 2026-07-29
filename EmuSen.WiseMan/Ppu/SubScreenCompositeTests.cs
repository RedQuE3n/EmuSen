using EmuSen.Cores.Nintendo.Venus;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Ppu
{
    // Renderer.Scanline.cs's CompositeScreen, driven once with TM and once
    // with TS - see Venus_PPU.md §4.1 for the ALTTP black-doorframe bug
    // that came from the sub screen having its own, shorter layer stack.
    public class SubScreenCompositeTests
    {
        private const int Red = 248;

        // Mode 1, BG1 sub-screen-only, color math adding the sub screen
        // onto a black backdrop - the exact ALTTP throne-room setup.
        private static VenusCore BuildSubScreenOnlyCore(bool highPriorityTile)
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            var ppu = core.Bus!.Ppu;

            ppu.Inidisp = 0x0F;
            ppu.Bgmode = 0x01;
            ppu.BgSc[0] = 0x00;   // BG1 tilemap at VRAM 0, 32x32
            ppu.Bg12Nba = 0x01;   // BG1 character data at VRAM 0x2000
            ppu.Tm = 0x00;
            ppu.Ts = 0x01;
            ppu.Cgwsel = 0x02;    // sub screen is the blend operand
            ppu.Cgadsub = 0x20;   // backdrop participates, add, no halve

            // Backdrop black, BG1 palette 0 colour 1 red.
            ppu.Cgram[2] = 0x1F;

            int entry = 0x0001 | (highPriorityTile ? 0x2000 : 0x0000);
            ppu.Vram[0] = (byte)(entry & 0xFF);
            ppu.Vram[1] = (byte)(entry >> 8);

            // Tile 1, every row solid colour index 1 (plane 0 only).
            for (int row = 0; row < 8; row++) ppu.Vram[0x2020 + row * 2] = 0xFF;

            return core;
        }

        private static (byte R, byte G, byte B) FirstPixel(VenusCore core)
        {
            core.Renderer!.RenderScanline(core.Bus!, 0);
            byte[] rgba = core.Renderer.GetFrameBufferRgba();
            return (rgba[0], rgba[1], rgba[2]);
        }

        [Fact]
        public void High_priority_bg1_tile_reaches_the_sub_screen()
        {
            Assert.Equal((Red, (byte)0, (byte)0), FirstPixel(BuildSubScreenOnlyCore(highPriorityTile: true)));
        }

        [Fact]
        public void Low_priority_bg1_tile_reaches_the_sub_screen()
        {
            Assert.Equal((Red, (byte)0, (byte)0), FirstPixel(BuildSubScreenOnlyCore(highPriorityTile: false)));
        }

        // Sub-screen OBJ used to be drawn after every BG regardless of
        // priority, so a priority-0 sprite covered a high-priority BG1 tile.
        [Fact]
        public void Sub_screen_obj_priority_still_loses_to_high_priority_bg1()
        {
            var core = BuildSubScreenOnlyCore(highPriorityTile: true);
            var ppu = core.Bus!.Ppu;

            ppu.Ts = 0x11;        // BG1 + OBJ on the sub screen
            ppu.Obsel = 0x03;     // OBJ character data at VRAM 0xC000

            ppu.Oam[0] = 0;       // x
            ppu.Oam[1] = 0;       // y
            ppu.Oam[2] = 0;       // tile 0
            ppu.Oam[3] = 0x00;    // priority 0, OBJ palette 0
            for (int row = 0; row < 8; row++) ppu.Vram[0xC000 + row * 2] = 0xFF;
            ppu.Cgram[(128 + 1) * 2 + 1] = 0x7C; // OBJ palette 0 colour 1 blue

            Assert.Equal((Red, (byte)0, (byte)0), FirstPixel(core));
        }
    }
}
