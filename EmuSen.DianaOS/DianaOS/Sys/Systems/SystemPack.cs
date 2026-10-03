using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.DianaOS.DianaOS.Sys.Systems
{
    // A console's facts that are no one engine's: its names, library folders, hashing rule and cover shape - see EmuSen_CoreAPI.md §8.2, §20.
    public sealed record SystemEntry(
        string Id,
        string Name,
        string Console,
        string Manufacturer,
        int ReleaseYear,
        IReadOnlyList<string> Extensions,
        IReadOnlyList<string> CheatSystems,
        IReadOnlyList<string> OpenVgdbSystems,
        Func<byte[], byte[]> OpenVgdbBytes,
        double CoverAspect,
        string EsdeSystem,
        string EsdeFullName);

    // A console's system pack: its entry and its cheat codecs, which any engine of the console uses - see EmuSen_CoreAPI.md §8.2, §20.
    public sealed record SystemPack(SystemEntry Entry, Func<ICheatCodeCodec>? AutoDetectCodec, Func<ICheatCodeCodec>? ExplicitCodec);

    // The packs by system id; a system without one is shown by its engine's own name and has no codecs.
    public static class SystemPacks
    {
        public static IReadOnlyList<SystemPack> All { get; } = new[] { Snes.SnesSystem.Pack };

        public static SystemPack? For(string? systemId) => All.FirstOrDefault(p => p.Entry.Id == systemId);
    }
}
