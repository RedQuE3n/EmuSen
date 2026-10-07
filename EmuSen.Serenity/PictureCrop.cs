using System;

namespace EmuSen.Serenity
{
    // The share of the picture hidden at each edge when it is drawn, the rest filling the screen - see EmuSen_Serenity.md §2.10.
    public readonly record struct PictureCrop(double Left, double Top, double Right, double Bottom)
    {
        public static readonly PictureCrop None = default;

        // How far past the glass a television's raster reached, as a share of the glass: Sony's figure for normal scan - see EmuSen_Serenity.md §2.10.
        public const double TelevisionOverscan = 0.07;

        // The most a setting may hide at one edge, so that half the picture is always left.
        public const double MostAtAnEdge = 0.25;

        public static PictureCrop Television { get; } = Enlarged(1 + TelevisionOverscan);

        // The picture enlarged about its centre, as a raster wider and taller than the glass is.
        public static PictureCrop Enlarged(double zoom)
        {
            double edge = zoom > 1 ? (1 - 1 / zoom) / 2 : 0;
            return new PictureCrop(edge, edge, edge, edge);
        }

        // Each edge as a percentage of the picture's width or height, held to what a setting may hide.
        public static PictureCrop Percent(double left, double top, double right, double bottom) => new(Share(left), Share(top), Share(right), Share(bottom));

        private static double Share(double percent) => double.IsFinite(percent) ? Math.Clamp(percent / 100, 0, MostAtAnEdge) : 0;

        // The shares of the width and of the height that are kept.
        public double Width => 1 - Left - Right;
        public double Height => 1 - Top - Bottom;

        // Nothing hidden, or values no picture could be cut by, which are taken as nothing.
        public bool IsNone => !(Left >= 0 && Top >= 0 && Right >= 0 && Bottom >= 0 && Width > 0 && Height > 0) || (Left == 0 && Top == 0 && Right == 0 && Bottom == 0);

        // Where the whole frame is drawn for the part kept to fill a rectangle, in whole pixels from that rectangle's corner.
        public (double X, double Y, double Width, double Height) Whole(double x, double y, double width, double height)
        {
            if (IsNone) return (x, y, width, height);
            double w = Math.Round(width / Width), h = Math.Round(height / Height);
            return (x - Math.Round(Left * w), y - Math.Round(Top * h), w, h);
        }
    }
}
