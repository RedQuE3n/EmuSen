using System.Collections.Generic;
using EmuSen.Crystal.Cheats;
using EmuSen.Galaxia.Input;

namespace EmuSen.Crystal.Systems.Nes
{
    // The NES's system pack: its systems-table entry, its pad, its two cheat formats and its debugger views - see EmuSen_CoreAPI.md §20, §31.
    public static class NesSystem
    {
        public const string Id = "nes";

        // The space names every NES engine publishes, so a cheat or a view means the same memory on each.
        public const string Ram = "RAM", PrgRam = "PRGRAM", Chr = "CHR", Ciram = "CIRAM", Oam = "OAM", Palette = "PALETTE", CpuBus = "CPUBUS";

        public const int Ports = 2;

        // The C# literal, the failover, Entry being the systems table's row with this literal's code; Famicom Disk System is other hardware and is not claimed - see EmuSen_Backend.md §12.
        public static readonly SystemEntry Literal = new(
            Id,
            "Nintendo Entertainment System",
            "NES",
            "Nintendo",
            1983,
            new[] { ".nes" },
            new[] { "Nintendo - Nintendo Entertainment System" },
            new[] { "NES" },
            WithoutInesHeader,
            1.43,
            "nes",
            "Nintendo Entertainment System",
            ScreenScraperSystem: 3);

        public static readonly SystemEntry Entry = SystemTable.Entry(Literal);

        // The pad in the order a bindings screen lists it.
        public static readonly IReadOnlyList<PadButton> PadButtons = new[]
        {
            PadButton.Up, PadButton.Down, PadButton.Left, PadButton.Right,
            PadButton.Select, PadButton.Start, PadButton.B, PadButton.A,
        };

        // OpenVGDB's NES row gives a 16-byte header to skip, which is the iNES header when the file has one.
        public static byte[] WithoutInesHeader(byte[] file) =>
            file.Length > 16 && file[0] == (byte)'N' && file[1] == (byte)'E' && file[2] == (byte)'S' && file[3] == 0x1A ? file[16..] : file;

        // "Game Genie", not "NES Game Genie", so the detected-format message reads as it always has.
        public static ICheatCodeCodec ManagedGameGenie() =>
            new DelegateCheatCodec("Game Genie", CheatCodeKind.RomPatch, null, NesGameGenieCodec.CanDecode, NesGameGenieCodec.Decode, NesGameGenieCodec.DecodeCompare);

        public static ICheatCodeCodec GameGenie() => SystemCode.Codec("nes.gg", ManagedGameGenie());

        public static ICheatCodeCodec ManagedRaw() =>
            new DelegateCheatCodec("NES address:value", CheatCodeKind.RamPoke, CpuBus, NesRawCodec.CanDecode, NesRawCodec.Decode);

        // Each codec is emusen-systems' under its id, its Managed twin the C# - see EmuSen_Backend.md §13.
        public static ICheatCodeCodec Raw() => SystemCode.Codec("nes.raw", ManagedRaw());

        // The pair a loaded NES game and a console chosen in the UI both get: detected, then explicit.
        public static (ICheatCodeCodec AutoDetect, ICheatCodeCodec Explicit) Pair() => (Raw(), GameGenie());

        // The boards the pack names by iNES mapper number, the C# failover and the parity class's oracle - see EmuSen_Backend.md §13.5.
        public static readonly IReadOnlyDictionary<int, string> BoardNamesManaged = new Dictionary<int, string>
        {
            [0] = "NROM",
            [1] = "MMC1",
            [2] = "UxROM",
            [3] = "CNROM",
            [4] = "MMC3",
            [7] = "AxROM",
            [9] = "MMC2",
            [11] = "Color Dreams",
            [64] = "RAMBO-1",
            [65] = "Irem H3001",
            [66] = "GxROM",
            [67] = "Sunsoft-3",
            [68] = "Sunsoft-4",
            [69] = "Sunsoft FME-7",
            [71] = "Camerica BF909x",
            [79] = "NINA-003-006",
        };

        // A mapper's board by name; null for a number the pack names none for.
        public static string? BoardName(int mapper) => SystemCode.Active ? SystemCode.BoardName(mapper) : BoardNamesManaged.GetValueOrDefault(mapper);

        public static readonly SystemPack Pack = new(Entry, Raw, GameGenie, DebugViews: SystemCode.Views(Id, NesDebugViews.Instance), ByteOrder: "little");
    }
}
