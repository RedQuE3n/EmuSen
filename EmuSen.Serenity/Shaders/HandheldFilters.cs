namespace EmuSen.Serenity.Shaders
{
    // Handheld LCD filters, as the platform library's catalog gives them - see EmuSen_Serenity.md §3.5.
    public static class HandheldFilters
    {
        public static ScreenFilter DmgLcd => FilterCatalog.Named("Game Boy LCD");
        public static ScreenFilter PocketLcd => FilterCatalog.Named("Game Boy Pocket LCD");
        public static ScreenFilter LightLcd => FilterCatalog.Named("Game Boy Light LCD");
        public static ScreenFilter GbcLcd => FilterCatalog.Named("Game Boy Color LCD");
        public static ScreenFilter AgbLcd => FilterCatalog.Named("Game Boy Advance LCD");
        public static ScreenFilter Ags101Lcd => FilterCatalog.Named("Game Boy Advance SP (AGS-101) LCD");
    }
}
