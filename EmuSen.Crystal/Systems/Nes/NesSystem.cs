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

        public static readonly SystemEntry Entry = new(
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
            "Nintendo Entertainment System");

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
        public static ICheatCodeCodec GameGenie() =>
            new DelegateCheatCodec("Game Genie", CheatCodeKind.RomPatch, null, NesGameGenieCodec.CanDecode, NesGameGenieCodec.Decode, NesGameGenieCodec.DecodeCompare);

        public static ICheatCodeCodec Raw() =>
            new DelegateCheatCodec("NES address:value", CheatCodeKind.RamPoke, CpuBus, NesRawCodec.CanDecode, NesRawCodec.Decode);

        // The pair a loaded NES game and a console chosen in the UI both get: detected, then explicit.
        public static (ICheatCodeCodec AutoDetect, ICheatCodeCodec Explicit) Pair() => (Raw(), GameGenie());

        public static readonly SystemPack Pack = new(Entry, Raw, GameGenie, DebugViews: NesDebugViews.Instance);
    }
}
