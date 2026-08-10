using EmuSen.Common.Imaging;
using static EmuSen.WiseMan.Cores.MercuryCommercialRom;

namespace EmuSen.WiseMan.Cores
{
    public class MercuryCommercialRomRenderSkipTests
    {
        public static TheoryData<string> Cartridges => Roms;

        // Fast-forward drops pixel writes and nothing else; the machine must not notice - see ICore.SkipRendering.
        [Theory]
        [MemberData(nameof(Cartridges))]
        public void Skipping_the_renderer_does_not_change_the_machine(string path)
        {
            if (path.Length == 0) return;

            Assert.Equal(RunAndHashMemory(path, skipRendering: false), RunAndHashMemory(path, skipRendering: true));
        }

        private static ulong RunAndHashMemory(string path, bool skipRendering)
        {
            var core = Load(path);
            core.SkipRendering = skipRendering;
            for (int i = 0; i < BootFrames; i++) core.RunFrame();

            var bus = core.Bus!;
            var state = new List<byte>(bus.Vram.Length + bus.Wram.Length + bus.Oam.Length + bus.HighRam.Length + 4);
            state.AddRange(bus.Vram);
            state.AddRange(bus.Wram);
            state.AddRange(bus.Oam);
            state.AddRange(bus.HighRam);

            // The window's own line counter is render-derived and deliberately kept live - see Mercury_Ppu.md §4.2.
            state.Add((byte)bus.Ppu.WindowLine);
            state.Add(bus.Ppu.Ly);
            state.Add(bus.Ppu.Lcdc);
            state.Add(bus.InterruptFlags);

            return FrameHash.Compute(state.ToArray());
        }
    }
}
