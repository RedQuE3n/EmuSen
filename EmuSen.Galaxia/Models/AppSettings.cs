namespace EmuSen.Galaxia.Models
{
    // General app preferences (log/ROM directories, selected core) - see
    // EmuSen_Config_Reference.md §3.1.
    //
    // SelectedCore is the console context the library and both cheat windows share - see EmuSen_Multicore.md §10.
    public class AppSettings
    {
        public string? LogDirectory { get; set; }
        public string? RomDirectory { get; set; }
        public string? StateDirectory { get; set; }

        // The user's own .cht tree - point this at an existing RetroArch
        // cheats folder to use it as-is. See `man cheat`.
        public string? CheatDatabaseDirectory { get; set; }
        // The no-filter choice; CoreCatalog re-exports this so a UI has one spelling.
        public const string AllConsoles = "All consoles";

        // What the value was defaulted to while nothing read it - see Upgraded() below.
        public const string LegacySelectedCoreDefault = "SNES (Venus)";

        public string SelectedCore { get; set; } = AllConsoles;

        // Absent from a file written before the filter read SelectedCore, which is what makes the upgrade happen once - see EmuSen_Multicore.md §10.3a.
        public bool SelectedCoreUpgraded { get; set; } = false;

        // The search half of the filter bars SelectedCore is the facet half of - see EmuSen_Config_Reference.md §3.1.
        public string LibrarySearch { get; set; } = "";

        public string CheatSearch { get; set; } = "";

        // Off by default - forcing this on unconditionally would break any
        // real two-controller game by feeding Controller 2 the same input
        // as Controller 1 even when a genuine second pad is plugged in.
        // Exists for games that read Controller 2 instead of Controller 1
        // for classic-game-in-a-compilation reasons (Super Mario All-Stars'
        // SMB1/2/3 being the known example - see EmuSen_Games_Tested.md),
        // toggled from InputSettingsWindow - the same workaround real
        // hardware players and other emulators use, not an input redesign.
        public bool MirrorPlayer1ToPlayer2 { get; set; } = false;

        // Stick-as-d-pad and its deadzone - see EmuSen_Settings_Reference.md §4.4.
        public bool AnalogStickAsDpad { get; set; } = true;
        public double StickDeadzone { get; set; } = 0.5;

        // Full screen, no menu bar and larger type, for a handheld or a television; a Steam Deck's session asks for it by itself - see EmuSen_Settings_Reference.md §4.29.
        public bool BigScreen { get; set; } = false;

        // The status bar at the bottom of the main window, and each of its two parts - see EmuSen_Settings_Reference.md §4.51.
        public bool ShowStatusBar { get; set; } = true;
        public bool ShowStatusText { get; set; } = true;
        public bool ShowFpsBar { get; set; } = true;

        // Ask, Resume or Restart when a game with a resume state starts - see EmuSen_Settings_Reference.md §4.31.
        public string ResumeOnLaunch { get; set; } = ResumeAsk;
        public const string ResumeAsk = "Ask";
        public const string ResumeAlways = "Resume";
        public const string ResumeNever = "Restart";

        // How the library shows its games, and which part of it - see EmuSen_Settings_Reference.md §4.33.
        public string LibraryView { get; set; } = LibraryGrid;
        public const string LibraryGrid = "Grid";
        public const string LibraryList = "List";
        public double LibraryTileScale { get; set; } = 1.0;
        public string LibraryCollection { get; set; } = "all";

        // The in-game bar's volume, 0 to 1, and OpenEmu's pause when the window is not the one in front - see EmuSen_Settings_Reference.md §4.34.
        public double Volume { get; set; } = 1.0;
        public bool PauseInBackground { get; set; } = true;

        // Read only from a file written before 2026-09-26, then dropped: OpenEmu's covers became the failover below - see EmuSen_Settings_Reference.md §4.60.
        [System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        public bool? OnlineCovers { get; set; }

        // OpenVGDB and libretro's thumbnails, asked only where ScreenScraper has nothing or cannot be used - see EmuSen_Settings_Reference.md §4.39 and §4.60.
        public bool OpenEmuFallback { get; set; } = true;

        // ScreenScraper as the library's source of media and game text, where EmuSen's developer file exists - see EmuSen_Settings_Reference.md §4.60.
        public bool Scraping { get; set; } = true;
        public bool ScrapeCovers { get; set; } = true;
        public bool ScrapeScreenshots { get; set; } = true;
        public bool ScrapeMarquees { get; set; } = true;
        public bool ScrapeTitleScreens { get; set; } = false;
        public bool ScrapeMiximages { get; set; } = true;
        public string ScrapeRegion { get; set; } = "auto";
        public string ScrapeLanguage { get; set; } = "en";
        public bool ScrapeRegionFallback { get; set; } = true;
        public int ScrapeThreads { get; set; } = 1;

        // Box art the library shows, read and never written except by Add Cover Art - see EmuSen_Settings_Reference.md §4.33.
        public string? ArtworkDirectory { get; set; }

        // A big-screen session's library: Mistress's own, or an ES-DE theme's view when one is found - see EmuSen_Settings_Reference.md §4.52.
        public string LibraryStyle { get; set; } = LibraryStyleTheme;
        public const string LibraryStyleMistress = "Mistress";
        public const string LibraryStyleTheme = "Theme";

        // The ES-DE theme folder, read in place; and an ES-DE downloaded_media folder, also only read - see §4.52.
        public string? BigPictureTheme { get; set; }
        public string? EsdeMediaDirectory { get; set; }

        // The theme's navigation sounds, on a stream beside the game's - see §4.52.
        public bool NavigationSounds { get; set; } = true;

        // Games the player hid from the library, with Hide from Library or the Hidden field, are listed again - see EmuSen_Settings_Reference.md §4.59.
        public bool ShowHiddenGames { get; set; }

        // The theme settings sheet's choices, keyed by the theme folder's full path so each theme keeps its own - see EmuSen_Settings_Reference.md §4.53.
        public System.Collections.Generic.Dictionary<string, BigPictureChoices> BigPicture { get; set; } = new();

        private static readonly ConfigFile<AppSettings> File = new("appsettings.json");

        public void Save() => File.Save(this);

        public static AppSettings Load() => Upgraded(File.Load(() => new AppSettings()));

        // A stored legacy default is not a choice - see EmuSen_Multicore.md §10.3.
        private static AppSettings Upgraded(AppSettings settings)
        {
            if (!settings.SelectedCoreUpgraded && settings.SelectedCore == LegacySelectedCoreDefault) settings.SelectedCore = AllConsoles;
            settings.SelectedCoreUpgraded = true;
            // A stored false was the old default every save wrote, so it is no choice; the failover keeps its new default either way and the old key goes - see §4.60.
            settings.OnlineCovers = null;
            return settings;
        }
    }
}
