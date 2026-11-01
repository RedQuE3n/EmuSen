using System;
using System.Linq;
using System.Text;

namespace EmuSen.Crystal.Systems.SegaDisc
{
    // The Saturn's and the Dreamcast's system packs, consoles only hosted cores run: their systems-table entries and the recogniser of their discs' system area; no codec, no medium - see EmuSen_Cores.md §4.9.
    public static class SegaDiscSystems
    {
        public const string SaturnId = "saturn", DreamcastId = "dreamcast";

        // The first sector of a raw track, the system area's bytes and the header before them.
        public const int First = 2352;

        private static readonly byte[] Sync = { 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00 };

        public static readonly SystemEntry SaturnLiteral = new(
            SaturnId,
            "Sega Saturn",
            "Saturn",
            "Sega",
            1994,
            new[] { ".cue", ".chd", ".ccd", ".m3u" },
            new[] { "Sega - Saturn" },
            new[] { "Saturn" },
            file => file,
            1.14,
            "saturn",
            "Sega Saturn",
            SelfDelimitingImages: false,
            Recognise: probe => Recognise(SaturnId, "SEGA SEGASATURN ", 0x20, probe),
            ScreenScraperSystem: 22);

        public static readonly SystemEntry DreamcastLiteral = new(
            DreamcastId,
            "Sega Dreamcast",
            "Dreamcast",
            "Sega",
            1998,
            new[] { ".gdi", ".cdi", ".chd", ".cue", ".ccd", ".m3u" },
            new[] { "Sega - Dreamcast" },
            Array.Empty<string>(),
            file => file,
            1.0,
            "dreamcast",
            "Sega Dreamcast",
            SelfDelimitingImages: false,
            Recognise: probe => Recognise(DreamcastId, "SEGA SEGAKATANA ", 0x40, probe),
            ScreenScraperSystem: 23);

        public static readonly SystemEntry Saturn = SystemTable.Entry(SaturnLiteral), Dreamcast = SystemTable.Entry(DreamcastLiteral);

        public static readonly SystemPack SaturnPack = new(Saturn, null, null, ByteOrder: "big"), DreamcastPack = new(Dreamcast, null, null, ByteOrder: "little");

        // The first sector's user data: past a raw sector's sync and header, mode 1 or 2, else the bytes as they are.
        public static ReadOnlySpan<byte> SystemArea(ReadOnlySpan<byte> head)
        {
            int at = head.Length >= 16 && head[..12].SequenceEqual(Sync) ? (head[15] == 1 ? 16 : 24) : 0;
            return at <= head.Length ? head[at..] : ReadOnlySpan<byte>.Empty;
        }

        // A disc whose system area begins with the system's identifier, its ten-byte product number the serial; null otherwise.
        private static Recognition? Recognise(string system, string identifier, int product, IMediaProbe probe)
        {
            ReadOnlySpan<byte> area = SystemArea(probe.Read(First));
            if (area.Length < 16 || !area[..16].SequenceEqual(Encoding.ASCII.GetBytes(identifier))) return null;
            string? serial = area.Length >= product + 10 ? Encoding.ASCII.GetString(area.Slice(product, 10)).Trim() : null;
            return new Recognition(system, string.IsNullOrEmpty(serial) ? null : serial);
        }
    }
}
