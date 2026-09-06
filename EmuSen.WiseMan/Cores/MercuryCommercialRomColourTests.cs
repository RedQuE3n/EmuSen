using System.IO;
using static EmuSen.WiseMan.Cores.MercuryCommercialRom;

namespace EmuSen.WiseMan.Cores
{
    // What the $0143 flag is worth on screen, measured per cartridge - see Mercury_RealCartridges.md §6.2.
    public class MercuryCommercialRomColourTests
    {
        public static TheoryData<string> Cartridges => Roms;

        [Theory]
        [MemberData(nameof(Cartridges))]
        public void The_header_flag_decides_the_mode_and_the_mode_reaches_the_screen(string path)
        {
            if (path.Length == 0) return;

            bool colour = HeaderFlag(path) is 0x80 or 0xC0;
            string name = Path.GetFileName(path);

            var core = Load(path);
            RunTo(core, BootFrames);

            Assert.Equal(colour ? "GBC" : "GB", core.CoreName);

            int coloured = ColouredPixels(core.GetFrameBufferRgba());
            if (!colour)
            {
                // The DMG palette is four neutral greys, so every channel matches - see Mercury_Ppu.md §6.
                Assert.True(coloured == 0, $"{name} declares no colour support but drew {coloured} coloured pixels.");
                return;
            }

            // A colour cartridge can still be on a monochrome logo at BootFrames, so give it a title screen too.
            if (coloured == 0)
            {
                RunTo(core, PlayFrames - BootFrames);
                coloured = ColouredPixels(core.GetFrameBufferRgba());
            }

            Assert.True(coloured > 0, $"{name} is a colour cartridge but drew only greys through {PlayFrames} frames.");
        }

        // The file's own byte, not Cartridge's reading of it: the claim is that the header decides.
        private static byte HeaderFlag(string path)
        {
            using var file = File.OpenRead(path);
            file.Seek(0x0143, SeekOrigin.Begin);
            return (byte)file.ReadByte();
        }

        private static int ColouredPixels(byte[] rgba)
        {
            int coloured = 0;
            for (int i = 0; i + 3 < rgba.Length; i += 4)
            {
                if (rgba[i] != rgba[i + 1] || rgba[i + 1] != rgba[i + 2]) coloured++;
            }

            return coloured;
        }
    }
}
