namespace EmuSen.Cores.Nintendo.Mars.Vi
{
    // Divot: three pixels across the row, of which the middle takes their median - see Mars_VideoPasses.md §2.
    public sealed partial class Vi
    {
        // A row of whole pixels is left alone, so the pass only reaches the edges the display processor left partly covered - see §2.
        private static Pixel Divot(Pixel centre, Pixel left, Pixel right)
        {
            if ((centre.Coverage & left.Coverage & right.Coverage) == 7) return centre;

            return centre with
            {
                Red = Median(centre.Red, left.Red, right.Red),
                Green = Median(centre.Green, left.Green, right.Green),
                Blue = Median(centre.Blue, left.Blue, right.Blue),
            };
        }

        // The median as the reference spells it, by asking which of the two ends lies between the other end and the middle - see §2.
        private static int Median(int centre, int left, int right)
        {
            if ((left >= centre && right >= left) || (left >= right && centre >= left)) return left;
            if ((right >= centre && left >= right) || (right >= left && centre >= right)) return right;

            return centre;
        }
    }
}
