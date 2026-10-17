using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.SaveMedia;

namespace EmuSen.Crystal.Systems.N64
{
    // The N64's system pack: its systems-table entry, its GameShark codec and its Controller Pak - see EmuSen_Backend.md §12, §13, EmuSen_SaveMedia.md §7.2.
    public static class N64System
    {
        public const string Id = "n64";

        // The space every N64 engine names its main memory, which the GameShark writes into.
        public const string Rdram = "RDRAM";

        // The C# literal, the failover, Entry being the systems table's row with this literal's code; all three container orders, since the magic word decides - see EmuSen_Backend.md §12, Mars_Rom.md §1.1.
        public static readonly SystemEntry Literal = new(
            Id,
            "Nintendo 64",
            "N64",
            "Nintendo",
            1996,
            new[] { ".z64", ".n64", ".v64" },
            new[] { "Nintendo - Nintendo 64" },
            new[] { "N64" },
            N64ImageOrder.ToByteSwapped,
            0.7,
            "n64",
            "Nintendo 64",
            ScreenScraperSystem: 14);

        public static readonly SystemEntry Entry = SystemTable.Entry(Literal);

        // The Controller Pak: the table's words with this literal's extension and module - see EmuSen_Backend.md §13.
        public static readonly SaveMediumEntry PakLiteral = new(N64PakFormat.Id, "Controller Pak", "Pak", ".mpk", N64PakFormat.Instance), Pak = SystemTable.Medium(PakLiteral);

        public static ICheatCodeCodec ManagedGameShark() =>
            new DelegateCheatCodec("GameShark", CheatCodeKind.RamPoke, Rdram, N64GameSharkCodec.CanDecode, N64GameSharkCodec.Decode, decodeWrites: N64GameSharkCodec.DecodeWrites);

        public static ICheatCodeCodec GameShark() => SystemCode.Codec("n64.gs", ManagedGameShark());

        // The GameShark pokes; no N64 format patches ROM, so the explicit slot stays empty - see Mars_Cheats.md §1.
        public static readonly SystemPack Pack = new(Entry, GameShark, null, Media: new[] { Pak });
    }
}
