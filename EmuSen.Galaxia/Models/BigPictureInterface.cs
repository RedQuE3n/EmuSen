namespace EmuSen.Galaxia.Models
{
    // Big picture's interface switches, ES-DE's UI, system status and sound settings spelled as its es_settings.xml values - see EmuSen_Settings_Reference.md §4.66.
    public sealed class BigPictureInterface
    {
        public const string QuickSelectLeftRightOrShoulders = "leftrightshoulders", QuickSelectLeftRightOrTriggers = "leftrighttriggers",
            QuickSelectShoulders = "shoulders", QuickSelectTriggers = "triggers", QuickSelectLeftRight = "leftright", QuickSelectDisabled = "disabled";
        public const string ViewSystem = "system", ViewGamelist = "gamelist";
        public const string SortRelease = "release", SortFullNames = "fullnames", SortReleaseYear = "releaseyear";

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
    }
}
