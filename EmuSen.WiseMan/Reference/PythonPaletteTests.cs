using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Text.RegularExpressions;
using EmuSen.Galaxia;

namespace EmuSen.WiseMan.Reference
{
    // The one duplicated constant the Python port could not avoid - see
    // EmuSen_Stack.md §3 and analysis/screen.py's own header.
    public class PythonPaletteTests
    {
        private static string ScreenPy => Path.Combine(
            ConfigRoot.Directory, "EmuSen.WiseMan", "Reference", "analysis", "screen.py");

        private static byte[] PythonPalette()
        {
            string source = File.ReadAllText(ScreenPy);
            int open = source.IndexOf("NES_PALETTE = bytes((", StringComparison.Ordinal);
            Assert.True(open >= 0, "screen.py no longer declares NES_PALETTE the way this test reads it");

            int close = source.IndexOf("))", open, StringComparison.Ordinal);
            string body = source[open..close];

            var bytes = new List<byte>();
            foreach (Match number in Regex.Matches(body, @"\b\d+\b"))
            {
                bytes.Add(byte.Parse(number.Value, CultureInfo.InvariantCulture));
            }
            return bytes.ToArray();
        }

        [Fact]
        public void The_python_palette_is_the_cores_palette()
        {
            Assert.Equal(EmuSen.Cores.Nintendo.Moon.Video.Ppu.NesPalette, PythonPalette());
        }

        [Fact]
        public void It_is_still_sixty_four_colours()
        {
            Assert.Equal(64 * 3, PythonPalette().Length);
        }
    }
}
