using EmuSen.Crystal.Cheats;

namespace EmuSen.Crystal.Systems.Sms
{
    // The Master System's and the SG-1000's system packs, one extension each, which share the two cheat formats and the debugger's views - see Endou_Plan.md §4, Endou_Native.md §3, §14 and §15.
    public static class SmsSystems
    {
        public const string MasterSystemId = "sms", Sg1000Id = "sg1000";

        // The C# literals, the failover; each entry is the systems table's row with its literal's code - see EmuSen_Backend.md §12.
        public static readonly SystemEntry MasterSystemLiteral = new(
            MasterSystemId,
            "Sega Master System / Mark III",
            "Master System",
            "Sega",
            1985,
            new[] { ".sms" },
            new[] { "Sega - Master System - Mark III" },
            new[] { "SMS" },
            file => file,
            1.4,
            "mastersystem",
            "Sega Master System",
            SelfDelimitingImages: false,
            ScreenScraperSystem: 2);

        public static readonly SystemEntry Sg1000Literal = new(
            Sg1000Id,
            "Sega SG-1000",
            "SG-1000",
            "Sega",
            1983,
            new[] { ".sg" },
            new[] { "Sega - SG-1000" },
            new[] { "SG1000" },
            file => file,
            1.4,
            "sg-1000",
            "Sega SG-1000",
            SelfDelimitingImages: false,
            ScreenScraperSystem: 109);

        public static readonly SystemEntry MasterSystem = SystemTable.Entry(MasterSystemLiteral), Sg1000 = SystemTable.Entry(Sg1000Literal);

        // The space a Pro Action Replay code writes, as Endou names the work RAM.
        public const string Ram = "RAM";

        public static ICheatCodeCodec ManagedProActionReplay() => new DelegateCheatCodec("Pro Action Replay", CheatCodeKind.RamPoke, Ram,
            SmsCheatFormats.IsProActionReplay, SmsCheatFormats.DecodeProActionReplay);

        public static ICheatCodeCodec ManagedGameGenie() => new DelegateCheatCodec("Game Genie", CheatCodeKind.RomPatch, null,
            SmsCheatFormats.IsGameGenie, SmsCheatFormats.DecodeGameGenie, SmsCheatFormats.GameGenieCompare);

        // Each codec is emusen-systems' under its id, its Managed twin the C# - see EmuSen_Backend.md §13.
        public static ICheatCodeCodec ProActionReplay() => SystemCode.Codec("sms.par", ManagedProActionReplay());

        public static ICheatCodeCodec GameGenie() => SystemCode.Codec("sms.gg", ManagedGameGenie());

        public static readonly SystemPack MasterSystemPack = new(MasterSystem, ProActionReplay, GameGenie, DebugViews: SystemCode.Views(MasterSystemId, SmsDebugViews.MasterSystem), ByteOrder: "little");
        public static readonly SystemPack Sg1000Pack = new(Sg1000, ProActionReplay, GameGenie, DebugViews: SystemCode.Views(Sg1000Id, SmsDebugViews.Sg1000), ByteOrder: "little");
    }
}
