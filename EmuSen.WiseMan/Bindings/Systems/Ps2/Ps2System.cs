using System.Collections.Generic;
using EmuSen.WiseMan.Bindings.SaveMedia;

namespace EmuSen.WiseMan.Bindings.Systems.Ps2
{
    // The PlayStation 2's system pack: its systems-table entry, its firmware, its screen and its memory card; no codec yet - see Sapphire_Plan.md §4, Sapphire_Native.md §3 and §30.16.
    public static class Ps2System
    {
        public const string Id = "ps2";

        // The systems table's row, its recogniser the library's - see EmuSen_Backend.md §12, §13.3.
        public static readonly SystemEntry Entry = SystemTable.Entry(Id);

        // What the core's own info lists: the pack's forms but .m3u, which the host reads - see Sapphire_Native.md §49.2.
        public static readonly IReadOnlyList<string> CoreExtensions = new[] { ".elf", ".iso", ".chd", ".cue", ".bin" };

        // The BIOS of each market by its Redump name, each optional - see Sapphire_Plan.md §5.9, Sapphire_Native.md §2.5 and §12.3.
        public static readonly IReadOnlyList<(string Market, string Name, long Size)> Bios = new[]
        {
            ("Japan", "ps2-0230j-20080220.bin", 4_194_304L),
            ("Americas", "ps2-0230a-20080220.bin", 4_194_304L),
            ("Europe", "ps2-0230e-20080220.bin", 4_194_304L),
        };

        // A television's 4:3, the shape the console's pictures are shown in at S1.
        public const double DisplayAspect = 4.0 / 3.0;

        // The memory card, the medium ps2.card: the table's words with the pack's extension and module - see EmuSen_SaveMedia.md §7.4.
        public static readonly SaveMediumEntry Card = SystemTable.Medium(Ps2CardFormat.Id, ".ps2", Ps2CardFormat.Instance);

        public static readonly SystemPack Pack = new(Entry, null, null, Media: new[] { Card });
    }
}
