namespace EmuSen.Crystal.Systems.GameBoy
{
    // The Game Boy's and the Game Boy Color's system packs, two consoles a player knows apart that one engine runs, sharing the two code formats - see EmuSen_Backend.md §12, Mercury_Core.md §1.
    public static class GameBoySystems
    {
        public const string GameBoyId = "gb", GameBoyColorId = "gbc";

        // The systems table's rows - see EmuSen_Backend.md §12.
        public static readonly SystemEntry GameBoy = SystemTable.Entry(GameBoyId), GameBoyColor = SystemTable.Entry(GameBoyColorId);

        public static readonly SystemPack GameBoyPack = new(GameBoy, GameBoyCheatCodecs.GameShark, GameBoyCheatCodecs.GameGenie),
            GameBoyColorPack = new(GameBoyColor, GameBoyCheatCodecs.GameShark, GameBoyCheatCodecs.GameGenie);
    }
}
