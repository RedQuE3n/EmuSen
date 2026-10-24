using System;
using System.Collections.Generic;
using System.Text;
using EmuSen.Crystal.Media;

namespace EmuSen.Crystal.Systems.Ps2
{
    // The PlayStation 2's system pack: its systems-table entry, its recogniser, its firmware and its screen; no codec and no medium yet - see Sapphire_Plan.md §4, Sapphire_Native.md §3.
    public static class Ps2System
    {
        public const string Id = "ps2";

        // The C# literal, the failover; Entry is the systems table's row with this literal's code - see EmuSen_Backend.md §12.
        public static readonly SystemEntry Literal = new(
            Id,
            "Sony PlayStation 2",
            "PlayStation 2",
            "Sony",
            2000,
            new[] { ".elf", ".iso", ".chd", ".cue", ".bin", ".m3u" },
            Array.Empty<string>(),
            Array.Empty<string>(),
            file => file,
            1.41,
            "ps2",
            "Sony PlayStation 2",
            SelfDelimitingImages: false,
            Recognise: Recognise,
            ScreenScraperSystem: 58);

        public static readonly SystemEntry Entry = SystemTable.Entry(Literal);

        // What the core's own info lists at S1: the ELF alone, the discs following at S23 - see Sapphire_Plan.md §11 Q16.
        public static readonly IReadOnlyList<string> CoreExtensions = new[] { ".elf" };

        // The bytes read first: sector 16 in either an .iso's 2,048-byte sectors or a raw track's 2,352-byte ones.
        public const int First = 40 * 1024;

        // The program SYSTEM.CNF's BOOT2 line names, as written after its '='; null where no line's key is BOOT2.
        public static string? Boot2(byte[]? config)
        {
            if (config is null) return null;
            foreach (string line in Encoding.ASCII.GetString(config).Split('\n'))
            {
                int eq = line.IndexOf('=');
                if (eq >= 0 && string.Equals(line[..eq].Trim(), "BOOT2", StringComparison.OrdinalIgnoreCase)) return line[(eq + 1)..].Trim();
            }
            return null;
        }

        // A disc, CD or DVD, whose root's SYSTEM.CNF names a BOOT2 program, with that program as its serial; null otherwise - see Sapphire_Plan.md §4.1.
        public static Recognition? Recognise(IMediaProbe probe)
        {
            byte[]? config = DiscSectors.RootFileThrough(probe, First, probe.Read(First), "SYSTEM.CNF", DiscSectors.RootFileLimit);
            return Boot2(config) is { } program ? new Recognition(Id, Psx.PsxSystem.SerialOfFile(program), null) : null;
        }

        // The BIOS of each market by its No-Intro name, each optional - see Sapphire_Plan.md §5.9, Sapphire_Native.md §2.5.
        public static readonly IReadOnlyList<(string Market, string Name, long Size)> Bios = new[]
        {
            ("Japan", "ps2-0210j-20040917.bin", 4_194_304L),
            ("Americas", "ps2-0230a-20080220.bin", 4_194_304L),
            ("Europe", "ps2-0230e-20080220.bin", 4_194_304L),
        };

        // A television's 4:3, the shape the console's pictures are shown in at S1.
        public const double DisplayAspect = 4.0 / 3.0;

        public static readonly SystemPack Pack = new(Entry, null, null, ByteOrder: "little");
    }
}
