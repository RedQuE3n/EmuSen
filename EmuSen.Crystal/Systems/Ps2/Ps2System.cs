using System;
using System.Collections.Generic;
using System.Text;
using EmuSen.Crystal.Media;
using EmuSen.Crystal.SaveMedia;

namespace EmuSen.Crystal.Systems.Ps2
{
    // The PlayStation 2's system pack: its systems-table entry, its recogniser, its firmware, its screen and its memory card; no codec yet - see Sapphire_Plan.md §4, Sapphire_Native.md §3 and §30.16.
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

        // What the core's own info lists: the pack's forms but .m3u, which the host reads - see Sapphire_Native.md §49.2.
        public static readonly IReadOnlyList<string> CoreExtensions = new[] { ".elf", ".iso", ".chd", ".cue", ".bin" };

        // The bytes read first: sector 16 in either an .iso's 2,048-byte sectors or a raw track's 2,352-byte ones.
        public const int First = 40 * 1024;

        // The value of SYSTEM.CNF's line whose key is key, as written after its '='; null where no line has that key.
        public static string? Value(byte[]? config, string key)
        {
            if (config is null) return null;
            foreach (string line in Encoding.ASCII.GetString(config).Split('\n'))
            {
                int eq = line.IndexOf('=');
                if (eq >= 0 && string.Equals(line[..eq].Trim(), key, StringComparison.OrdinalIgnoreCase)) return line[(eq + 1)..].Trim();
            }
            return null;
        }

        // The program SYSTEM.CNF's BOOT2 line names; null where no line's key is BOOT2.
        public static string? Boot2(byte[]? config) => Value(config, "BOOT2");

        // The disc's market: its serial's third letter, Sony's region letter, else Europe where VMODE says PAL - see Sapphire_Native.md §49.2.
        public static string? Region(string? serial, byte[]? config) => (serial is { Length: > 2 } ? char.ToUpperInvariant(serial[2]) : '\0') switch
        {
            'U' => "Americas",
            'E' => "Europe",
            'P' or 'A' or 'K' or 'C' => "Japan",
            _ => string.Equals(Value(config, "VMODE"), "PAL", StringComparison.OrdinalIgnoreCase) ? "Europe" : null,
        };

        // A disc, CD or DVD, whose root's SYSTEM.CNF names a BOOT2 program, with that program as its serial and its market; null otherwise - see Sapphire_Plan.md §4.1.
        public static Recognition? Recognise(IMediaProbe probe)
        {
            byte[]? config = DiscSectors.RootFileThrough(probe, First, probe.Read(First), "SYSTEM.CNF", DiscSectors.RootFileLimit);
            if (Boot2(config) is not { } program) return null;
            string? serial = Psx.PsxSystem.SerialOfFile(program);
            return new Recognition(Id, serial, Region(serial, config));
        }

        // The BIOS of each market by its Redump name, each optional - see Sapphire_Plan.md §5.9, Sapphire_Native.md §2.5 and §12.3.
        public static readonly IReadOnlyList<(string Market, string Name, long Size)> Bios = new[]
        {
            ("Japan", "ps2-0230j-20080220.bin", 4_194_304L),
            ("Americas", "ps2-0230a-20080220.bin", 4_194_304L),
            ("Europe", "ps2-0230e-20080220.bin", 4_194_304L),
        };

        // A television's 4:3, the shape the console's pictures are shown in at S1.
        public const double DisplayAspect = 4.0 / 3.0;

        // The memory card, the medium ps2.card: the table's words with this literal's extension and module - see EmuSen_SaveMedia.md §7.4.
        public static readonly SaveMediumEntry CardLiteral = new(Ps2CardFormat.Id, "Memory Card (PS2)", "Card", ".ps2", Ps2CardFormat.Instance), Card = SystemTable.Medium(CardLiteral);

        public static readonly SystemPack Pack = new(Entry, null, null, Media: new[] { Card }, ByteOrder: "little");
    }
}
