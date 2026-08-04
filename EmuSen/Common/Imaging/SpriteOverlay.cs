using EmuSen.Cauldron;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.Common.Imaging
{
    // Draws a rectangle outline (not filled - a filled box would hide the
    // very sprite pixels you're trying to locate) directly into an RGBA
    // buffer. Works from IDebugTarget.DebugSpriteInfo alone, so this has no
    // idea what console produced it - same core-agnostic split as every
    // other piece of this toolkit.
    public static class SpriteOverlay
    {
        public static void DrawSpriteOutline(byte[] rgba, int width, int height, DebugSpriteInfo s)
        {
            void SetPixel(int x, int y)
            {
                if (x < 0 || x >= width || y < 0 || y >= height) return;
                int i = (y * width + x) * 4;
                rgba[i] = 0; rgba[i + 1] = 255; rgba[i + 2] = 0; rgba[i + 3] = 255; // green
            }

            for (int x = s.X; x < s.X + s.Width; x++)
            {
                SetPixel(x, s.Y);
                SetPixel(x, s.Y + s.Height - 1);
            }
            for (int y = s.Y; y < s.Y + s.Height; y++)
            {
                SetPixel(s.X, y);
                SetPixel(s.X + s.Width - 1, y);
            }
        }
    }
}
