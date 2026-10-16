using EmuSen.WiseMan.Fixtures;
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
            using var core = Load(path);
            core.SkipRendering = skipRendering;
            for (int i = 0; i < BootFrames; i++) core.RunFrame();

            byte[] saved = core.State();
            var layout = MercuryLayouts.For(core.Rom, saved);
            var state = new List<byte>();
            foreach (string array in new[] { "Bus.Vram", "Bus.Wram", "Bus.Oam", "Bus.HighRam" }) state.AddRange(layout.Bytes(saved, array).ToArray());

            // The window's own line counter is render-derived and deliberately kept live - see Mercury_Ppu.md §4.2.
            foreach (string field in new[] { "Bus.Ppu.WindowLine", "Bus.Ppu.Ly", "Bus.Ppu.Lcdc", "Bus.InterruptFlags" }) state.Add((byte)layout.Get(saved, field));

            return FrameHash.Compute(state.ToArray());
        }
    }
}
