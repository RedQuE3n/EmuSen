using System;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.Moon.Validation;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // The PPU and NMI defects stage 2c fixed, each failing on the unmodified core - see Moon_Native.md §3.10.
    public class MoonPpuDefectTests
    {
        private readonly ITestOutputHelper _output;

        public MoonPpuDefectTests(ITestOutputHelper output) => _output = output;

        private static MoonCore Load()
        {
            CoreOptions.BatteryRamDisabled = true;
            string path = SyntheticNesRom.WriteTemp(SyntheticNesRom.Build(mapper: 0));
            try
            {
                var core = new MoonCore();
                core.LoadRom(path);
                return core;
            }
            finally
            {
                File.Delete(path);
            }
        }

        // P1: OAMADDR is held at 0 through a rendered line's sprite fetches, so a DMA the next frame starts at slot 0.
        [Fact]
        public void Rendering_returns_oamaddr_to_zero()
        {
            var core = Load();
            core.Bus!.Write(0x2003, 0x05);
            core.Bus.Write(0x2001, 0x18);
            core.RunFrame();
            core.RunFrame();
            Assert.Equal(0, core.Ppu!.OamAddress);
        }

        // P2: dots 1-64 of a rendered line clear secondary OAM, so $2004 reads $FF there.
        [Fact]
        public void A_read_of_2004_while_secondary_oam_clears_is_ff()
        {
            var core = Load();
            var ppu = core.Ppu!;
            ppu.Mask = 0x18;
            ppu.OamAddress = 0;
            ppu.Oam[0] = 0x12;
            ppu.Scanline = 10;
            ppu.Cycle = 30;
            Assert.Equal(0xFF, ppu.ReadRegister(4));
            ppu.Cycle = 100;
            Assert.Equal(0x12, ppu.ReadRegister(4));
        }

        // P3-P6 and C1: blargg's NMI timing, suppression, NMI on and off, the odd frame's skip, and NMI against BRK and IRQ.
        [Fact]
        public void The_nmi_and_vblank_timing_roms_pass()
        {
            string? corpus = Environment.GetEnvironmentVariable(MoonRtCorpusTests.CorpusVariable);
            if (corpus is null || !Directory.Exists(corpus))
            {
                _output.WriteLine($"{MoonRtCorpusTests.CorpusVariable} unset, not run");
                return;
            }
            CoreOptions.BatteryRamDisabled = true;
            string[] roms =
            {
                "ppu_vbl_nmi/rom_singles/05-nmi_timing.nes", "ppu_vbl_nmi/rom_singles/06-suppression.nes",
                "ppu_vbl_nmi/rom_singles/07-nmi_on_timing.nes", "ppu_vbl_nmi/rom_singles/08-nmi_off_timing.nes",
                "ppu_vbl_nmi/rom_singles/10-even_odd_timing.nes",
                "cpu_interrupts_v2/rom_singles/2-nmi_and_brk.nes", "cpu_interrupts_v2/rom_singles/3-nmi_and_irq.nes",
            };
            var failed = roms.Select(r => (Rom: r, Result: NesTestRomRunner.Run(Path.Combine(corpus, r), 2400))).Where(x => x.Result.Outcome != NesTestRomOutcome.Passed).ToList();
            foreach (var (rom, result) in failed) _output.WriteLine($"{rom}: {result.Outcome} {result.Code}");
            Assert.Empty(failed);
        }

        // The picture before the palette: every pixel's colour number and PPUMASK's emphasis, for a filter that draws the PPU's own signal - see EmuSen_CRT.md §22.
        [Fact]
        public void The_raw_picture_carries_each_pixel_s_colour_number_and_the_emphasis_bits()
        {
            var core = Load();
            core.Bus!.Write(0x2001, 0xA8);
            core.Bus.Write(0x2006, 0x3F);
            core.Bus.Write(0x2006, 0x00);
            core.Bus.Write(0x2007, 0x16);
            core.RunFrame();
            core.RunFrame();
            byte[] raw = core.RawPicture!;
            byte[] rgb = core.GetFrameBufferRgba();
            Assert.Equal(rgb.Length, raw.Length);
            int centre = (120 * 256 + 128) * 4;
            Assert.Equal((0x16, 0xA8 >> 5, 255), (raw[centre], raw[centre + 1], raw[centre + 3]));
            Assert.Equal((rgb[centre], rgb[centre + 1], rgb[centre + 2]), (EmuSen.Cores.Nintendo.Moon.Video.Ppu.NesPalette[0x16 * 3], EmuSen.Cores.Nintendo.Moon.Video.Ppu.NesPalette[0x16 * 3 + 1], EmuSen.Cores.Nintendo.Moon.Video.Ppu.NesPalette[0x16 * 3 + 2]));
        }
    }
}
