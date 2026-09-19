namespace EmuSen.Cores.Nintendo.Mars.Vi
{
    // Gamma and its dithering, the last thing that happens to a pixel before the raster - see Mars_VideoPasses.md §3.
    public sealed partial class Vi
    {
        private const int GammaEntries = 0x4000;

        // A channel reads the entry six bits above it, the six below being where a dither would go - see §3.
        private static readonly byte[] GammaTable = BuildGamma();

        private static byte[] BuildGamma()
        {
            var table = new byte[GammaEntries];
            for (int i = 0; i < GammaEntries; i++) table[i] = (byte)(Root(i) << 1);

            return table;
        }

        // The reference's own square root, two bits of the argument a step, which is what the hardware's table holds - see §3.
        private static int Root(int value)
        {
            int rest = value, result = 0, one = 1 << 30;

            while (one > rest) one >>= 2;

            while (one != 0)
            {
                if (rest >= result + one)
                {
                    rest -= result + one;
                    result += one << 1;
                }

                result >>= 1;
                one >>= 2;
            }

            return result;
        }

        // The table stands in for the hardware's gamma ROM; the six bits below a channel are the dither's, and Mars has none - see §3.1.
        private Pixel Gamma(Pixel pixel)
        {
            if (!_scanGamma) return pixel;

            return pixel with
            {
                Red = GammaTable[pixel.Red << 6],
                Green = GammaTable[pixel.Green << 6],
                Blue = GammaTable[pixel.Blue << 6],
            };
        }
    }
}
