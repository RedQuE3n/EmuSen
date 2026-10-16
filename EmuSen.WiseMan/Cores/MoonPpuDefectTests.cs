using System;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.Pharaoh;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // The PPU and NMI defects stage 2c fixed, held on MoonRT - see Moon_Native.md §3.10.
    public class MoonPpuDefectTests
    {
        private readonly ITestOutputHelper _output;

        public MoonPpuDefectTests(ITestOutputHelper output) => _output = output;

        // P1: OAMADDR is held at 0 through a rendered line's sprite fetches, so a DMA the next frame starts at slot 0.
        [Fact]
        public void Rendering_returns_oamaddr_to_zero()
        {
            CoreOptions.BatteryRamDisabled = true;
            string path = SyntheticNesRom.WriteTemp(SyntheticNesRom.Build(mapper: 0));
            using var core = new MoonRtCore();
            core.LoadRom(path);
            File.Delete(path);
            core.WriteSpace("CPUBUS", 0x2003, 0x05);
            core.WriteSpace("CPUBUS", 0x2001, 0x18);
            core.RunFrame();
            core.RunFrame();
            Assert.Equal(0, core.Layout!.Int(core.Machine.Save(), "Ppu.OamAddress"));
        }

        // P3-P6 and C1: blargg's NMI timing, suppression, NMI on and off, the odd frame's skip, and NMI against BRK and IRQ.
        [Fact]
        public void The_nmi_and_vblank_timing_roms_pass()
        {
            if (MoonGoldenTests.CorpusRoot is not { } corpus)
            {
                _output.WriteLine($"{MoonGoldenTests.CorpusVariable} unset, not run");
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
    }
}
