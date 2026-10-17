using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Crystal.Cheats;

namespace EmuSen.Crystal.Systems
{
    // What routing by content shows a recogniser of a file: its path, the image's own name where it is inside an archive, and its first bytes, a sheet's first track's for a sheet - see EmuSen_CoreAPI_Extensions.md §8.4.
    public interface IMediaProbe
    {
        string Path { get; }
        string ImageName { get; }

        // The image's first bytes, at most length of them, read once and kept.
        byte[] Read(int length);
    }

    // A recogniser's claim: the system, and the game's serial and market where the pack can read them.
    public sealed record Recognition(string SystemId, string? Serial = null, string? Region = null);

    // A console's facts that are no one engine's; SelfDelimitingImages says a malformed image must be refused, which C4 asks only of such a format; Recognise claims a file by its content where several systems take its extension; AttachedTo names the console an attachment plugs into, whose screen and pads it uses; ScreenScraperSystem is ScreenScraper's systemeid - see EmuSen_CoreAPI.md §8.2, §20, §21.4, EmuSen_CoreAPI_Extensions.md §8.4, EmuSen_Serenity.md §3.12 and EmuSen_Backend.md §12.
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
        bool SelfDelimitingImages = false,
        Func<IMediaProbe, Recognition?>? Recognise = null,
        string? AttachedTo = null,
        int? ScreenScraperSystem = null);

    // A console's system pack: its entry, its cheat codecs and its debugger views, which any engine of the console uses - see EmuSen_CoreAPI.md §8.2, §20, §31.
    // PlayerControllers answers each player's controller id from the console's settings (EmuSen_Settings_Reference.md §4.102); Media names its save media and their format modules (EmuSen_SaveMedia.md §6.2).
    public sealed record SystemPack(SystemEntry Entry, Func<ICheatCodeCodec>? AutoDetectCodec, Func<ICheatCodeCodec>? ExplicitCodec,
        Func<Func<string, string?>, IReadOnlyList<string>>? PlayerControllers = null, ISystemDebugViews? DebugViews = null, IReadOnlyList<SaveMedia.SaveMediumEntry>? Media = null);

    // The packs by system id, in the systems table's order; a system without one is shown by its engine's own name and has no codecs.
    public static class SystemPacks
    {
        public static IReadOnlyList<SystemPack> All { get; } = new[]
        {
            Nes.NesSystem.Pack, Snes.SnesSystem.Pack, Genesis.GenesisSystems.MegaDrivePack, Genesis.GenesisSystems.MegaCdPack, Genesis.GenesisSystems.S32xPack, Psx.PsxSystem.Pack,
            GameBoy.GameBoySystems.GameBoyPack, GameBoy.GameBoySystems.GameBoyColorPack, N64.N64System.Pack,
        };

        // Each pack's C# literal entry in the same order: the failover under EMUSEN_SYSTEMS_NATIVE=0 and the parity class's oracle - see EmuSen_Backend.md §12.
        public static IReadOnlyList<SystemEntry> Literals { get; } = new[]
        {
            Nes.NesSystem.Literal, Snes.SnesSystem.Literal, Genesis.GenesisSystems.MegaDriveLiteral, Genesis.GenesisSystems.MegaCdLiteral, Genesis.GenesisSystems.S32xLiteral, Psx.PsxSystem.Literal,
            GameBoy.GameBoySystems.GameBoyLiteral, GameBoy.GameBoySystems.GameBoyColorLiteral, N64.N64System.Literal,
        };

        public static SystemPack? For(string? systemId) => All.FirstOrDefault(p => p.Entry.Id == systemId);
    }
}
