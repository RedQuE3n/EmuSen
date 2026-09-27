namespace EmuSen.Galaxia.Models
{
    // Big picture's interface switches, ES-DE's UI, system status and sound settings spelled as its es_settings.xml values - see EmuSen_Settings_Reference.md §4.66.
    public sealed class BigPictureInterface
    {
        public const string QuickSelectLeftRightOrShoulders = "leftrightshoulders", QuickSelectLeftRightOrTriggers = "leftrighttriggers",
            QuickSelectShoulders = "shoulders", QuickSelectTriggers = "triggers", QuickSelectLeftRight = "leftright", QuickSelectDisabled = "disabled";
        public const string ViewSystem = "system", ViewGamelist = "gamelist";
        public const string SortRelease = "release", SortFullNames = "fullnames", SortReleaseYear = "releaseyear";
        public const string LaunchNormal = "normal", LaunchBrief = "brief", LaunchLong = "long", LaunchPopup = "popup", LaunchDisabled = "disabled";
        public const string SaverDim = "dim", SaverBlack = "black", SaverSlideshow = "slideshow", SaverVideo = "video";

        // ES-DE's LaunchScreenDuration - see EmuSen_Settings_Reference.md §4.71.
        public string LaunchScreenDuration { get; set; } = LaunchNormal;

        public bool DisplayClock { get; set; }

        public bool DisplayHelp { get; set; } = true;

        public bool StatusBluetooth { get; set; } = true;
        public bool StatusWifi { get; set; } = true;
        public bool StatusBattery { get; set; } = true;
        public bool StatusBatteryPercentage { get; set; } = true;

        public string QuickSystemSelect { get; set; } = QuickSelectLeftRightOrShoulders;

        // An ES-DE system name, or empty for the first system of the order.
        public string StartupSystem { get; set; } = "";

        public string StartupView { get; set; } = ViewSystem;

        public string SystemsSorting { get; set; } = SortRelease;

        public bool ListScrollOverlay { get; set; }

        // ES-DE's SoundVolumeNavigation, 0 to 100.
        public int NavigationVolume { get; set; } = 70;

        // ES-DE's screensaver settings, its times in milliseconds - see EmuSen_Settings_Reference.md §4.75.
        public int ScreensaverTimer { get; set; } = 300000;

        // Dim until videos exist (Q34); ES-DE's own default is video, which falls back to Dim without them.
        public string ScreensaverType { get; set; } = SaverDim;

        public bool ScreensaverControls { get; set; } = true;

        public int ScreensaverSwapImageTimeout { get; set; } = 10000;

        public bool ScreensaverSlideshowOnlyFavorites { get; set; }

        public bool ScreensaverStretchImages { get; set; }

        public bool ScreensaverSlideshowGameInfo { get; set; } = true;

        public bool ScreensaverSlideshowCustomImages { get; set; }

        public bool ScreensaverSlideshowRecurse { get; set; }

        public string ScreensaverSlideshowCustomDir { get; set; } = "";

        // Mistress's own: whether it may start in a Steam Game Mode session, where Steam dims the screen too (Q34, left open).
        public bool ScreensaverInGameMode { get; set; } = true;
    }
}
