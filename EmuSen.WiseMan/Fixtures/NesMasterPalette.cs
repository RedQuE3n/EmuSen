using System;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Systems.Nes;

namespace EmuSen.WiseMan.Fixtures
{
    // The NES pack's 64 colours as RGB triples, read back through the library's palette view one colour at a time, since the pack holds no C# copy.
    public static class NesMasterPalette
    {
        private static readonly Lazy<byte[]> Read = new(() =>
        {
            var rgb = new byte[64 * 3];
            for (int value = 0; value < 64; value++)
            {
                var space = DelegateDebugMemorySpace.Over("PALETTE", Array.ConvertAll(new byte[32], _ => (byte)value), isWritable: false);
                var (r, g, b) = NesSystem.Pack.DebugViews!.Palettes(new IDebugMemorySpace[] { space })[0].Colors[0];
                (rgb[value * 3], rgb[value * 3 + 1], rgb[value * 3 + 2]) = (r, g, b);
            }
            return rgb;
        });

        public static byte[] Bytes => Read.Value;
    }
}
