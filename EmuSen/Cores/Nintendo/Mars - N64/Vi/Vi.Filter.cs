namespace EmuSen.Cores.Nintendo.Mars.Vi
{
    // The anti-aliasing filter: a partly covered pixel is pulled towards the neighbours that are whole - see Mars_VideoFilter.md §2.
    public sealed partial class Vi
    {
        // Above this mode every pixel is taken as whole, and no coverage is read at all - see §1.
        private const int Covered = 1;

        // A pixel as the interface reads it: eight bits a channel, and the coverage stored beside them - see §1.
        private readonly record struct Pixel(int Red, int Green, int Blue, int Coverage);

        // Six neighbours, of which only the whole ones count; the row below becomes this row when the fetch bug is on - see §2.1 and §3.
        private Pixel Filter(uint origin, int at, bool wide, int width, int bug, Pixel centre)
        {
            Span<int> red = stackalloc int[7];
            Span<int> green = stackalloc int[7];
            Span<int> blue = stackalloc int[7];

            red[0] = centre.Red;
            green[0] = centre.Green;
            blue[0] = centre.Blue;
            int full = 1;

            Span<int> around = stackalloc int[6]
            {
                at - width - 1, at - width + 1, at - 2, at + 2,
                bug == 1 ? at - 2 : at + width - 1,
                bug == 1 ? at + 2 : at + width + 1,
            };

            foreach (int neighbour in around)
            {
                Pixel pixel = Fetched(origin, neighbour, wide);
                if (pixel.Coverage != 7) continue;

                red[full] = pixel.Red;
                green[full] = pixel.Green;
                blue[full] = pixel.Blue;
                full++;
            }

            int missing = 7 - centre.Coverage;

            return new Pixel(
                Pull(red[..full], centre.Red, missing),
                Pull(green[..full], centre.Green, missing),
                Pull(blue[..full], centre.Blue, missing),
                centre.Coverage);
        }

        // The two runners-up stand for the neighbourhood, and the pixel moves towards them by the coverage it lacks - see §2.2.
        private static int Pull(ReadOnlySpan<int> values, int centre, int missing)
        {
            (int low, int high) = Runners(values);

            // The runners-up bracket the pixel, so the result cannot leave a byte and is not masked to one - see §4.2.
            return ((((low + high - (centre << 1)) * missing) + 4) >> 3) + centre;
        }

        // The runner-up at each end, except that a centre nothing ever beats stands as its own runner-up - see §2.2.
        private static (int Low, int High) Runners(ReadOnlySpan<int> values)
        {
            int lowest = 0, highest = 0;
            int low = values[0], high = values[0];

            // A runner-up is only written when the leader is displaced, so a leader that never moves leaves its own value there - see §2.2.
            for (int i = 1; i < values.Length; i++)
            {
                if (values[i] > values[highest])
                {
                    high = values[highest];
                    highest = i;
                }

                if (values[i] < values[lowest])
                {
                    low = values[lowest];
                    lowest = i;
                }
            }

            // Only what comes after the leader can still displace its runner-up, because everything before it already tried - see §2.2.
            for (int i = highest + 1; i < values.Length; i++)
            {
                if (values[i] > high) high = values[i];
            }

            for (int i = lowest + 1; i < values.Length; i++)
            {
                if (values[i] < low) low = values[i];
            }

            return (low, high);
        }
    }
}
