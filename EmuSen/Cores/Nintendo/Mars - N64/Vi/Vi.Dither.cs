namespace EmuSen.Cores.Nintendo.Mars.Vi
{
    // Undoing the display processor's dither: a whole pixel steps towards each brighter neighbour and away from each darker one - see Mars_VideoPasses.md §1.
    public sealed partial class Vi
    {
        private sealed partial class Walker
        {
            // All eight neighbours this time, in the reference's own order; the row below folds onto this one when the fetch bug is on - see §1.1.
            private Pixel Dither(uint origin, int at, bool wide, int width, int bug, Pixel centre)
            {
                Span<int> around = stackalloc int[8]
                {
                    at - width - 1, at - width, at - width + 1,
                    bug == 1 ? at - 1 : at + width - 1,
                    bug == 1 ? at : at + width,
                    bug == 1 ? at + 1 : at + width + 1,
                    at - 1, at + 1,
                };

                int red = centre.Red, green = centre.Green, blue = centre.Blue;

                foreach (int neighbour in around)
                {
                    Pixel pixel = Fetched(origin, neighbour, wide);

                    red += Step(centre.Red, pixel.Red);
                    green += Step(centre.Green, pixel.Green);
                    blue += Step(centre.Blue, pixel.Blue);
                }

                return centre with { Red = red, Green = green, Blue = blue };
            }
        }

        // Only the five bits the processor dithered at are compared, and every neighbour is weighed against the pixel as it arrived - see §1.
        private static int Step(int centre, int neighbour) => System.Math.Sign((neighbour & 0xF8) - (centre & 0xF8));
    }
}
