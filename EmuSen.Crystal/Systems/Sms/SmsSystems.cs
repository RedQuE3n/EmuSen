namespace EmuSen.Crystal.Systems.Sms
{
    // The Master System's and the SG-1000's system packs, one extension each; codecs, views and settings come with Endou's later stages - see Endou_Plan.md §4, Endou_Native.md §3.
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
            0.7,
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
            0.7,
            "sg-1000",
            "Sega SG-1000",
            SelfDelimitingImages: false,
            ScreenScraperSystem: 109);

        public static readonly SystemEntry MasterSystem = SystemTable.Entry(MasterSystemLiteral), Sg1000 = SystemTable.Entry(Sg1000Literal);

        public static readonly SystemPack MasterSystemPack = new(MasterSystem, null, null, ByteOrder: "little");
        public static readonly SystemPack Sg1000Pack = new(Sg1000, null, null, ByteOrder: "little");
    }
}
