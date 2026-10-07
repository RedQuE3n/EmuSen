using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.DianaOS.DianaOS.Sys.Systems
{
    // A console's facts that are no one engine's; SelfDelimitingImages says a malformed image must be refused, which C4 asks only of such a format - see EmuSen_CoreAPI.md §8.2, §20, §21.4.
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
        string EsdeFullName,
        bool SelfDelimitingImages = false);

    // A console's system pack: its entry and its cheat codecs, which any engine of the console uses - see EmuSen_CoreAPI.md §8.2, §20.
    // PlayerControllers answers each player's controller id from the console's settings, where its ports can hold adapters - see EmuSen_Settings_Reference.md §4.102.
    public sealed record SystemPack(SystemEntry Entry, Func<ICheatCodeCodec>? AutoDetectCodec, Func<ICheatCodeCodec>? ExplicitCodec,
        Func<Func<string, string?>, IReadOnlyList<string>>? PlayerControllers = null);

    // The packs by system id; a system without one is shown by its engine's own name and has no codecs.
    public static class SystemPacks
    {
        public static IReadOnlyList<SystemPack> All { get; } = new[] { Snes.SnesSystem.Pack, Genesis.GenesisSystems.MegaDrivePack, Genesis.GenesisSystems.MegaCdPack, Genesis.GenesisSystems.S32xPack };

        public static SystemPack? For(string? systemId) => All.FirstOrDefault(p => p.Entry.Id == systemId);
    }
}
