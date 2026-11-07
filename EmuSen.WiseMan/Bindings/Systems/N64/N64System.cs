using EmuSen.WiseMan.Bindings.Cheats;
using EmuSen.WiseMan.Bindings.SaveMedia;

namespace EmuSen.WiseMan.Bindings.Systems.N64
{
    // The N64's system pack: its systems-table entry, its GameShark codec, its Controller Pak and its debugger views - see EmuSen_Backend.md §12, §13, EmuSen_SaveMedia.md §7.2.
    public static class N64System
    {
        public const string Id = "n64";

        // The space every N64 engine names its main memory, which the GameShark writes into.
        public const string Rdram = "RDRAM";

        // The systems table's row: all three container orders, since the magic word decides - see EmuSen_Backend.md §12, Mars_Rom.md §1.1.
        public static readonly SystemEntry Entry = SystemTable.Entry(Id);

        // The Controller Pak: the table's words with the pack's extension and module - see EmuSen_Backend.md §13.
        public static readonly SaveMediumEntry Pak = SystemTable.Medium(N64PakFormat.Id, ".mpk", N64PakFormat.Instance);

        public static ICheatCodeCodec GameShark() => SystemCode.Codec("n64.gs");

        // The GameShark pokes; no N64 format patches ROM, so the explicit slot stays empty - see Mars_Cheats.md §1.
        public static readonly SystemPack Pack = new(Entry, GameShark, null, Media: new[] { Pak });
    }
}
