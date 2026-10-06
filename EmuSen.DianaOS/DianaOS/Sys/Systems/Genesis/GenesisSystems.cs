using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.DianaOS.DianaOS.Sys.Systems.Genesis
{
    // The Genesis's three system packs, the console and its two attachments, which share the console's two cheat formats - see Nephrite_Plan.md §4.1, §4.5.
    public static class GenesisSystems
    {
        public const string MegaDriveId = "md", MegaCdId = "mcd", S32xId = "32x";

        // The space names the Genesis's engines share, so a cheat means the same memory on each.
        public const string Wram = "WRAM";

        public static readonly SystemEntry MegaDrive = new(
            MegaDriveId,
            "Sega Genesis / Mega Drive",
            "Genesis",
            "Sega",
            1988,
            new[] { ".md", ".gen", ".bin", ".smd" },
            new[] { "Sega - Mega Drive - Genesis" },
            new[] { "MD" },
            file => file,
            0.7,
            "genesis",
            "Sega Genesis",
            SelfDelimitingImages: false);

        public static readonly SystemEntry MegaCd = new(
            MegaCdId,
            "Sega CD / Mega-CD",
            "Sega CD",
            "Sega",
            1991,
            new[] { ".iso" },
            new[] { "Sega - Mega-CD - Sega CD" },
            new[] { "SCD" },
            file => file,
            0.7,
            "segacd",
            "Sega CD",
            SelfDelimitingImages: false);

        public static readonly SystemEntry S32x = new(
            S32xId,
            "Sega 32X",
            "32X",
            "Sega",
            1994,
            new[] { ".32x" },
            new[] { "Sega - 32X" },
            new[] { "32X" },
            file => file,
            0.7,
            "sega32x",
            "Sega Mega Drive 32X",
            SelfDelimitingImages: false);

        public static ICheatCodeCodec ActionReplay() => new DelegateCheatCodec("Action Replay", CheatCodeKind.RamPoke, Wram,
            GenesisCheatFormats.IsActionReplay, GenesisCheatFormats.DecodeActionReplay, decodeWrites: GenesisCheatFormats.ActionReplayWrites);

        public static ICheatCodeCodec GameGenie() => new DelegateCheatCodec("Game Genie", CheatCodeKind.RomPatch, null,
            GenesisCheatFormats.IsPatch, GenesisCheatFormats.DecodePatch, decodeWrites: GenesisCheatFormats.PatchWrites);

        public static readonly SystemPack MegaDrivePack = new(MegaDrive, ActionReplay, GameGenie);
        public static readonly SystemPack MegaCdPack = new(MegaCd, ActionReplay, GameGenie);
        public static readonly SystemPack S32xPack = new(S32x, ActionReplay, GameGenie);
    }
}
