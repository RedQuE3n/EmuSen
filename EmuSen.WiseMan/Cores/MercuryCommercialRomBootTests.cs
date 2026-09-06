using EmuSen.Cores.Nintendo.Mercury;
using static EmuSen.WiseMan.Cores.MercuryCommercialRom;

namespace EmuSen.WiseMan.Cores
{
    // One theory per class, so xUnit can run the seven cartridges' worth of emulation
    // in each of these concurrently with the other six - see §3.56.
    public class MercuryCommercialRomBootTests
    {
        public static TheoryData<string> Cartridges => Roms;

        [Theory]
        [MemberData(nameof(Cartridges))]
        public void A_real_cartridge_boots_and_reaches_a_title_screen(string path)
        {
            if (path.Length == 0) return;

            var core = Load(path);
            RunTo(core, BootFrames);

            Assert.Equal(BootFrames, core.TotalFrames);

            // A frame of one flat colour means the PPU never drew anything the game asked for.
            byte[] frame = core.GetFrameBufferRgba();
            var shades = new HashSet<byte>();
            for (int i = 0; i < frame.Length; i += 4) shades.Add(frame[i]);

            Assert.True(shades.Count > 1, $"{Path.GetFileName(path)} rendered a single flat colour after {BootFrames} frames.");
        }
    }
}
