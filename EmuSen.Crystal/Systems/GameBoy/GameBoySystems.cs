namespace EmuSen.Crystal.Systems.GameBoy
{
    // The Game Boy's and the Game Boy Color's system packs, two consoles a player knows apart that one engine runs, sharing the two code formats - see EmuSen_Backend.md §12, Mercury_Core.md §1.
    public static class GameBoySystems
    {
        public const string GameBoyId = "gb", GameBoyColorId = "gbc";

        // The C# literals, the failover; each entry is the systems table's row with its literal's code - see EmuSen_Backend.md §12.
        public static readonly SystemEntry GameBoyLiteral = new(
            GameBoyId,
            "Game Boy",
            "GB",
            "Nintendo",
            1989,
            new[] { ".gb" },
            new[] { "Nintendo - Game Boy" },
            new[] { "GB" },
            file => file,
            1.0,
            "gb",
            "Game Boy",
            ScreenScraperSystem: 9);

        public static readonly SystemEntry GameBoyColorLiteral = new(
            GameBoyColorId,
            "Game Boy Color",
            "GBC",
            "Nintendo",
            1998,
            new[] { ".gbc" },
            new[] { "Nintendo - Game Boy Color" },
            new[] { "GBC" },
            file => file,
            1.0,
            "gbc",
            "Game Boy Color",
            ScreenScraperSystem: 10);

        public static readonly SystemEntry GameBoy = SystemTable.Entry(GameBoyLiteral), GameBoyColor = SystemTable.Entry(GameBoyColorLiteral);

        public static readonly SystemPack GameBoyPack = new(GameBoy, GameBoyCheatCodecs.GameShark, GameBoyCheatCodecs.GameGenie, ByteOrder: "little"),
            GameBoyColorPack = new(GameBoyColor, GameBoyCheatCodecs.GameShark, GameBoyCheatCodecs.GameGenie, ByteOrder: "little");
    }
}
