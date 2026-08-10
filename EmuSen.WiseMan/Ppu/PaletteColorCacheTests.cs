using EmuSen.Cores.Nintendo.Venus;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Ppu
{
    // The renderer caches CGRAM as converted colours; these pin its invalidation - see Venus_PPU.md §7.2.
    [Collection(TestCollections.ProcessGlobals)]
    public class PaletteColorCacheTests
    {
        private const ushort Cgadd = 0x2121, Cgdata = 0x2122;

        private static VenusCore BuildCore()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            core.Bus!.Ppu.Inidisp = 0x0F;
            return core;
        }

        private static void WriteBackdropViaRegisters(VenusCore core, ushort color)
        {
            core.Bus!.Write8(Cgadd, 0x00);
            core.Bus.Write8(Cgdata, (byte)color);
            core.Bus.Write8(Cgdata, (byte)(color >> 8));
        }

        private static (byte R, byte G, byte B) TopLeft(VenusCore core)
        {
            byte[] rgba = core.GetFrameBufferRgba();
            return (rgba[0], rgba[1], rgba[2]);
        }

        [Fact]
        public void A_cgram_register_write_marks_the_palette_changed()
        {
            var core = BuildCore();
            core.Bus!.Ppu.CgramChanged = false;

            WriteBackdropViaRegisters(core, 0x001F);

            Assert.True(core.Bus.Ppu.CgramChanged);
        }

        [Fact]
        public void Rendering_a_frame_consumes_the_changed_flag()
        {
            var core = BuildCore();
            WriteBackdropViaRegisters(core, 0x001F);

            core.RunFrame();

            Assert.False(core.Bus!.Ppu.CgramChanged);
        }

        [Fact]
        public void A_register_write_reaches_the_backdrop()
        {
            var core = BuildCore();
            WriteBackdropViaRegisters(core, 0x001F);

            core.RunFrame();

            Assert.Equal(((byte)248, (byte)0, (byte)0), TopLeft(core));
        }

        // Tests, debug tools and state restore all poke Cgram directly, past the flag - see §7.2.
        [Fact]
        public void A_direct_cgram_poke_still_reaches_the_backdrop()
        {
            var core = BuildCore();
            core.Bus!.Ppu.Cgram[0] = 0x00;
            core.Bus.Ppu.Cgram[1] = 0x7C;
            core.Bus.Ppu.CgramChanged = false;

            core.RunFrame();

            Assert.Equal(((byte)0, (byte)0, (byte)248), TopLeft(core));
        }

        // A second frame with no write at all must not lose the cached colour.
        [Fact]
        public void The_backdrop_survives_a_frame_with_no_palette_write()
        {
            var core = BuildCore();
            WriteBackdropViaRegisters(core, 0x03E0);
            core.RunFrame();
            var first = TopLeft(core);

            core.RunFrame();

            Assert.Equal(first, TopLeft(core));
            Assert.Equal(((byte)0, (byte)248, (byte)0), first);
        }

        // Brightness scales the cached colour, so a change must rebuild it too.
        [Fact]
        public void Halving_brightness_darkens_the_cached_backdrop()
        {
            var core = BuildCore();
            WriteBackdropViaRegisters(core, 0x001F);
            core.RunFrame();
            byte full = TopLeft(core).R;

            core.Bus!.Ppu.Inidisp = 0x07;
            core.RunFrame();
            byte dim = TopLeft(core).R;

            Assert.True(dim < full, $"expected dimmer than {full}, got {dim}");
        }
    }
}
