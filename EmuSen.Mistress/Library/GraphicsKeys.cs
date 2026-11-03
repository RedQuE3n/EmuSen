namespace EmuSen.Mistress.Library
{
    // The frontend's own per-console keys in graphics.json, beside the core's schema keys, one owner for every screen that reads them - see EmuSen_Settings_Reference.md §4.40, §4.97 and §4.99.
    public static class GraphicsKeys
    {
        public const string ScreenFilter = "ScreenFilter";
        public const string PictureShape = "PictureShape", TvShape = "tv", SquarePixels = "square";
        public const string Overscan = "Overscan", NoOverscan = "off", TvOverscan = "tv", CustomCrop = "custom";
    }
}
