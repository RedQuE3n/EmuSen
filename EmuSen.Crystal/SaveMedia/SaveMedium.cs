using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Crystal.Systems;

namespace EmuSen.Crystal.SaveMedia
{
    // One medium's file system, every method pure: bytes in, bytes out, no file, no clock - see EmuSen_SaveMedia.md §6.2.
    public interface ISaveMediumFormat
    {
        string Medium { get; }

        // The file's length, and the medium's capacity in its own unit ("block", "page").
        int Length { get; }
        int Capacity { get; }
        string Unit { get; }

        MediumListing Read(ReadOnlySpan<byte> image);

        // The target with the save copied onto it, or the refusal in words.
        MediumEdit Copy(ReadOnlySpan<byte> from, string save, ReadOnlySpan<byte> to);

        MediumEdit Delete(ReadOnlySpan<byte> image, string save);

        // A formatted blank, or null where the medium's format is not known to write.
        byte[]? Format();
    }

    // A save as the console's own manager shows it; Id is the medium's own name for it, which a copy carries.
    public sealed record SaveEntry(string Id, string Title, int Units, string? Game, bool Damaged = false);

    public sealed record MediumListing(bool Formatted, IReadOnlyList<SaveEntry> Saves, int FreeUnits, string? Problem = null)
    {
        public static MediumListing Unreadable(string problem) => new(false, Array.Empty<SaveEntry>(), 0, problem);
    }

    public sealed record MediumEdit(byte[]? Image, string? Refusal)
    {
        public static MediumEdit Done(byte[] image) => new(image, null);
        public static MediumEdit Refused(string why) => new(null, why);
    }

    // A console's medium as its system pack names it, read by a format module where one exists - see EmuSen_SaveMedia.md §6.2.
    public sealed record SaveMediumEntry(string Id, string Title, string Noun, string Extension, ISaveMediumFormat? Format = null);

    // A medium's noun or title as it reads inside a sentence: every word lower case but an acronym, so "Backup RAM" is "backup RAM" - see EmuSen_SaveMedia.md §6.2.
    public static class MediumWords
    {
        public static string InText(string words) =>
            string.Join(' ', words.Split(' ').Select(w => w.Length > 1 && w.All(c => !char.IsLetter(c) || char.IsUpper(c)) ? w : w.ToLowerInvariant()));
    }

    // The media every system pack names, by id or by extension.
    public static class SaveMediumFormats
    {
        // The media of a console with no system pack yet, its facts still its catalog core's, by system id - see EmuSen_SaveMedia.md §7.2.
        private static readonly (string System, SaveMediumEntry Entry)[] Unpacked = { (Systems.N64.N64Media.SystemId, Systems.N64.N64Media.Pak) };

        public static IEnumerable<SaveMediumEntry> All => SystemPacks.All.SelectMany(p => p.Media ?? Array.Empty<SaveMediumEntry>()).Concat(Unpacked.Select(u => u.Entry));

        public static SaveMediumEntry? Entry(string? id) => id is null ? null : All.FirstOrDefault(e => e.Id == id);

        // A system's media, its pack's or those named before it has one.
        public static IEnumerable<SaveMediumEntry> Of(string systemId) =>
            SystemPacks.For(systemId)?.Media ?? Unpacked.Where(u => u.System == systemId).Select(u => u.Entry);

        // A system's medium of this extension, or any pack's where no system is given.
        public static SaveMediumEntry? ForExtension(string extension, string? systemId = null) =>
            (systemId is null ? All : Of(systemId)).FirstOrDefault(e => string.Equals(e.Extension, extension, StringComparison.OrdinalIgnoreCase));
    }
}
