namespace EmuSen.WiseMan.Fixtures
{
    // Names the runtime's listings and settings give, written out as a test's expected values - see EmuSen_Backend.md §40.10.
    public static class EngineNames
    {
        public const string EngineKey = "Engine";
        public const string MarsRt = "MarsRT (Rust)";
        public const string MercuryRt = "MercuryRT (Rust)";
        public const string MoonRt = "MoonRT (Rust)";
        public const string VenusRt = "VenusRT (Rust)";
        public const string GameBoyColorShelf = "Game Boy Color (Mercury)";
    }

    // A medium's card mode as consoles.json stores it - see EmuSen_SaveMedia.md §6.5.
    public static class CardModes
    {
        public const string Key = "Cards";
        public const string PerGame = "game";
        public const string Shared = "shared";
    }
}
