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
    // PlayerControllers answers each player's controller id from the console's settings (EmuSen_Settings_Reference.md §4.102); Media names its save media and their format modules (EmuSen_SaveMedia.md §6.2); ByteOrder is the table's byte_order literal (EmuSen_Backend.md §18.7).
    public sealed record SystemPack(SystemEntry Entry, Func<ICheatCodeCodec>? AutoDetectCodec, Func<ICheatCodeCodec>? ExplicitCodec,
        Func<Func<string, string?>, IReadOnlyList<string>>? PlayerControllers = null, ISystemDebugViews? DebugViews = null, IReadOnlyList<SaveMedia.SaveMediumEntry>? Media = null,
        string? ByteOrder = null);

    // The packs by system id, in the systems table's order; a system without one is shown by its engine's own name and has no codecs.
    public static class SystemPacks
    {
        public static IReadOnlyList<SystemPack> All { get; } = new[]
        {
            Nes.NesSystem.Pack, Snes.SnesSystem.Pack, Genesis.GenesisSystems.MegaDrivePack, Genesis.GenesisSystems.MegaCdPack, Genesis.GenesisSystems.S32xPack, Psx.PsxSystem.Pack,
            GameBoy.GameBoySystems.GameBoyPack, GameBoy.GameBoySystems.GameBoyColorPack, N64.N64System.Pack, Sms.SmsSystems.MasterSystemPack, Sms.SmsSystems.Sg1000Pack,
        };

        // Each pack's C# literal entry in the same order: the failover under EMUSEN_SYSTEMS_NATIVE=0 and the parity class's oracle - see EmuSen_Backend.md §12.
        public static IReadOnlyList<SystemEntry> Literals { get; } = new[]
        {
            Nes.NesSystem.Literal, Snes.SnesSystem.Literal, Genesis.GenesisSystems.MegaDriveLiteral, Genesis.GenesisSystems.MegaCdLiteral, Genesis.GenesisSystems.S32xLiteral, Psx.PsxSystem.Literal,
            GameBoy.GameBoySystems.GameBoyLiteral, GameBoy.GameBoySystems.GameBoyColorLiteral, N64.N64System.Literal, Sms.SmsSystems.MasterSystemLiteral, Sms.SmsSystems.Sg1000Literal,
        };

        // Each pack's C# literal media by its system's id, in the table's order: the parity class's oracle for the table's media - see EmuSen_Backend.md §13.
        public static IReadOnlyList<(string System, SaveMedia.SaveMediumEntry Medium)> MediumLiterals { get; } = new[]
        {
            (Genesis.GenesisSystems.MegaCdId, Genesis.GenesisSystems.RamCartridgeLiteral), (Genesis.GenesisSystems.MegaCdId, Genesis.GenesisSystems.BackupRamLiteral),
            (Psx.PsxSystem.Id, Psx.PsxSystem.CardLiteral), (N64.N64System.Id, N64.N64System.PakLiteral),
        };

        // Each pack's codecs by the library's ids with the C# codecs' factories, in the table's order: the parity class's oracle for the library's codecs - see EmuSen_Backend.md §13.
        public static IReadOnlyList<(string System, (string Id, Func<ICheatCodeCodec> Managed)? Auto, (string Id, Func<ICheatCodeCodec> Managed)? Explicit)> ManagedCodecs { get; } = new (string, (string, Func<ICheatCodeCodec>)?, (string, Func<ICheatCodeCodec>)?)[]
        {
            (Nes.NesSystem.Id, ("nes.raw", Nes.NesSystem.ManagedRaw), ("nes.gg", Nes.NesSystem.ManagedGameGenie)),
            (Snes.SnesSystem.Id, ("snes.par", Snes.SnesSystem.ManagedProActionReplay), ("snes.gg", Snes.SnesSystem.ManagedGameGenie)),
            (Genesis.GenesisSystems.MegaDriveId, ("md.ar", Genesis.GenesisSystems.ManagedActionReplay), ("md.gg", Genesis.GenesisSystems.ManagedGameGenie)),
            (Genesis.GenesisSystems.MegaCdId, ("md.ar", Genesis.GenesisSystems.ManagedActionReplay), ("md.gg", Genesis.GenesisSystems.ManagedGameGenie)),
            (Genesis.GenesisSystems.S32xId, ("md.ar", Genesis.GenesisSystems.ManagedActionReplay), ("md.gg", Genesis.GenesisSystems.ManagedGameGenie)),
            (Psx.PsxSystem.Id, ("psx.gs", Psx.PsxSystem.ManagedGameShark), ("psx.xp", Psx.PsxSystem.ManagedXploder)),
            (GameBoy.GameBoySystems.GameBoyId, ("gb.gs", GameBoy.GameBoyCheatCodecs.ManagedGameShark), ("gb.gg", GameBoy.GameBoyCheatCodecs.ManagedGameGenie)),
            (GameBoy.GameBoySystems.GameBoyColorId, ("gb.gs", GameBoy.GameBoyCheatCodecs.ManagedGameShark), ("gb.gg", GameBoy.GameBoyCheatCodecs.ManagedGameGenie)),
            (N64.N64System.Id, ("n64.gs", N64.N64System.ManagedGameShark), null),
        };

        public static SystemPack? For(string? systemId) => All.FirstOrDefault(p => p.Entry.Id == systemId);

        // Whether a system's memory holds a multi-byte value big-end first: the table's byte_order where the switch chose it, else the pack's literal.
        public static bool BigEndian(string? systemId) =>
            For(systemId) is { } pack && (SystemTable.Active && SystemTable.ByteOrders.TryGetValue(pack.Entry.Id, out string? order) ? order : pack.ByteOrder) == "big";
    }
}
