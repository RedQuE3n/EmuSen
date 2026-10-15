using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Galaxia.Library;

namespace EmuSen.Crystal.SaveMedia
{
    // What an operation of the card manager did, or why it did nothing.
    public sealed record MediaOutcome(bool Done, string Words)
    {
        public static MediaOutcome Ok(string words) => new(true, words);
        public static MediaOutcome No(string words) => new(false, words);
    }

    // A card's contents as the manager reads it: its medium, whether a file exists, and its saves where a module reads it.
    public sealed record MediumCard(string Path, SaveMediumEntry Medium, bool Exists, long Length, MediumListing? Listing)
    {
        public string Name => System.IO.Path.GetFileNameWithoutExtension(Path);
    }

    // The card manager's one implementation, over files, a format module per medium and the lock; it needs no core - see EmuSen_SaveMedia.md §6.
    public static class SaveMediaManager
    {
        public const string Holder = "the card manager";

        // A file's medium, by its extension among the system packs' media.
        public static SaveMediumEntry? MediumOf(string path) => SaveMediumFormats.ForExtension(System.IO.Path.GetExtension(path));

        public static MediumCard? Open(string path)
        {
            if (MediumOf(path) is not { } medium) return null;
            byte[]? bytes = File.Exists(path) ? AtomicFile.TryRead(path) : null;
            MediumListing? listing = medium.Format is { } format ? bytes is null ? new MediumListing(false, Array.Empty<SaveEntry>(), format.Capacity, "no file yet: a new, blank card") : format.Read(bytes) : null;
            return new MediumCard(path, medium, bytes is not null, bytes?.LongLength ?? 0, listing);
        }

        // Who holds a card: its game in this program, another program, or null.
        public static string? HeldBy(string path) => MediumLock.HolderOf(path);

        // A save named by its id, or by its place in the card's list from 1.
        public static SaveEntry? Find(MediumListing listing, string save) =>
            listing.Saves.FirstOrDefault(s => s.Id == save) ?? (int.TryParse(save, out int n) && n >= 1 && n <= listing.Saves.Count ? listing.Saves[n - 1] : null);

        public static MediaOutcome CopySave(string from, string save, string to) => Edit(to, target =>
        {
            if (Readable(from, out ISaveMediumFormat? format, out byte[] source) is { } problem) return (null, problem);
            if (MediumOf(to)?.Id != format!.Medium) return (null, $"{Show(to)} is not a card of the same medium as {Show(from)}");
            if (Find(format.Read(source), save) is not { } entry) return (null, $"no save {save} is on {Show(from)}");
            MediumEdit edit = format.Copy(source, entry.Id, Formatted(format, target) ?? target);
            return edit.Image is null ? (null, edit.Refusal!) : (edit.Image, $"{entry.Title} ({entry.Id}) copied onto {Show(to)}");
        });

        // A copy, read back, then the delete: the first card loses the save only once the second holds it.
        public static MediaOutcome MoveSave(string from, string save, string to)
        {
            if (Readable(from, out ISaveMediumFormat? format, out byte[] source) is { } problem) return MediaOutcome.No(problem);
            if (Find(format!.Read(source), save) is not { } entry) return MediaOutcome.No($"no save {save} is on {Show(from)}");
            if (HeldBy(from) is { } holder) return MediaOutcome.No($"{Show(from)} is in use by {holder}, so nothing is moved off it");
            MediaOutcome copied = CopySave(from, entry.Id, to);
            if (!copied.Done) return copied;
            if (AtomicFile.TryRead(to) is not { } written || format.Read(written).Saves.All(s => s.Id != entry.Id)) return MediaOutcome.No($"{entry.Id} did not read back from {Show(to)}, so {Show(from)} keeps it");
            MediaOutcome deleted = DeleteSave(from, entry.Id);
            return deleted.Done ? MediaOutcome.Ok($"{entry.Title} ({entry.Id}) moved from {Show(from)} to {Show(to)}") : deleted;
        }

        // As the console's manager deletes, which the medium's module says how.
        public static MediaOutcome DeleteSave(string card, string save) => Edit(card, image =>
        {
            if (MediumOf(card)?.Format is not { } format) return (null, $"no format module reads {Show(card)}'s medium, so its saves cannot be told apart");
            if (Find(format.Read(image), save) is not { } entry) return (null, $"no save {save} is on {Show(card)}");
            MediumEdit edit = format.Delete(image, entry.Id);
            return edit.Image is null ? (null, edit.Refusal!) : (edit.Image, $"{entry.Title} ({entry.Id}) deleted from {Show(card)}");
        });

        // Every save of the given cards onto one card, all or none: refused whole past the card's capacity or where two cards hold different saves of one name - see EmuSen_SaveMedia.md §6.6.
        public static MediaOutcome Import(string to, IReadOnlyList<string> from) => Edit(to, target =>
        {
            if (MediumOf(to)?.Format is not { } format) return (null, $"no format module reads {Show(to)}'s medium; a game's whole card can be duplicated onto a new name instead");
            byte[] image = Formatted(format, target) ?? target;
            MediumListing there = format.Read(image);
            if (!there.Formatted) return (null, $"{Show(to)} is not a formatted card: {there.Problem}");
            var taken = new List<(string Card, byte[] Image, SaveEntry Save)>();
            var present = new List<string>();
            foreach (string card in from)
            {
                if (string.Equals(Path.GetFullPath(card), Path.GetFullPath(to), StringComparison.Ordinal)) continue;
                if (MediumOf(card)?.Id != format.Medium) return (null, $"{Show(card)} is not a card of the same medium as {Show(to)}");
                if (!File.Exists(card)) continue;
                if (AtomicFile.TryRead(card) is not { } source) return (null, $"{Show(card)} could not be read");
                MediumListing listing = format.Read(source);
                if (!listing.Formatted) continue;
                foreach (SaveEntry save in listing.Saves)
                {
                    if (save.Damaged) return (null, $"{save.Id} on {Show(card)} is damaged and cannot be copied; nothing was imported");
                    if (there.Saves.Any(s => s.Id == save.Id)) { present.Add(save.Id); continue; }
                    if (taken.FirstOrDefault(t => t.Save.Id == save.Id) is { Card: { } other }) return (null, $"{Show(other)} and {Show(card)} both hold a save named {save.Id}; copy the one wanted by itself. Nothing was imported");
                    taken.Add((card, source, save));
                }
            }
            int needed = taken.Sum(t => t.Save.Units);
            if (needed > there.FreeUnits)
                return (null, $"{Show(to)} has {there.FreeUnits} free {format.Unit}(s) of {format.Capacity} and these need {needed}, so nothing was imported:\n" +
                    string.Join("\n", taken.Select(t => $"  {t.Save.Units} {format.Unit}(s)  {t.Save.Id}  {t.Save.Title}  (from {Show(t.Card)})")));
            foreach (var (_, source, save) in taken)
            {
                MediumEdit edit = format.Copy(source, save.Id, image);
                if (edit.Image is null) return (null, $"{save.Id}: {edit.Refusal}; nothing was imported");
                image = edit.Image;
            }
            string kept = present.Count == 0 ? "" : $"; {present.Count} already on it left as they are";
            return (image, $"{taken.Count} save(s), {needed} {format.Unit}(s), imported onto {Show(to)}{kept}");
        });

        // A new card: formatted where the medium's module can, else no file, which the game formats as a new card.
        public static MediaOutcome New(string path)
        {
            if (MediumOf(path) is not { } medium) return MediaOutcome.No($"{Show(path)} is of no medium a system pack names");
            if (File.Exists(path)) return MediaOutcome.No($"{Show(path)} already exists");
            if (medium.Format?.Format() is not { } blank) return MediaOutcome.Ok($"{Show(path)} is named; with no file yet it is a new, blank {medium.Noun.ToLowerInvariant()}, which the game formats");
            return Write(path, blank, $"{Show(path)} made, a formatted {medium.Title.ToLowerInvariant()}");
        }

        // A whole card to a new name, read as bytes, which needs no module.
        public static MediaOutcome Duplicate(string from, string to)
        {
            if (MediumOf(from) is not { } medium || MediumOf(to)?.Id != medium.Id) return MediaOutcome.No($"{Show(from)} and {Show(to)} are not files of one medium");
            if (File.Exists(to)) return MediaOutcome.No($"{Show(to)} already exists; nothing is written over a card");
            if (AtomicFile.TryRead(from) is not { } bytes) return MediaOutcome.No($"{Show(from)} could not be read");
            return Write(to, bytes, $"{Show(from)} copied whole to {Show(to)}");
        }

        public static MediaOutcome Rename(string from, string to)
        {
            if (MediumOf(from) is not { } medium || MediumOf(to)?.Id != medium.Id) return MediaOutcome.No($"{Show(from)} and {Show(to)} are not files of one medium");
            if (!File.Exists(from)) return MediaOutcome.No($"{Show(from)} does not exist");
            if (File.Exists(to)) return MediaOutcome.No($"{Show(to)} already exists");
            if (HeldBy(from) is { } holder) return MediaOutcome.No($"{Show(from)} is in use by {holder}");
            using MediumLock? held = MediumLock.TryTake(from, Holder);
            if (held is null) return MediaOutcome.No($"{Show(from)} is in use by {HeldBy(from) ?? "another program"}");
            File.Move(from, to);
            return MediaOutcome.Ok($"{Show(from)} renamed {Show(to)}");
        }

        private static string Show(string path) => Path.GetFileName(path);

        private static string? Readable(string path, out ISaveMediumFormat? format, out byte[] bytes)
        {
            bytes = Array.Empty<byte>();
            format = MediumOf(path)?.Format;
            if (format is null) return MediumOf(path) is null ? $"{Show(path)} is of no medium a system pack names" : $"no format module reads {Show(path)}'s medium";
            if (AtomicFile.TryRead(path) is not { } read) return $"{Show(path)} could not be read";
            bytes = read;
            return null;
        }

        // An absent or all-zero card formatted first, as a new card is; any other image as it is.
        private static byte[]? Formatted(ISaveMediumFormat format, byte[] image) =>
            image.Length == 0 || image.All(b => b == 0) ? format.Format() : null;

        // One write of a card under the manager's own lock, refused where a game or another program holds it.
        private static MediaOutcome Edit(string path, Func<byte[], (byte[]? Image, string Words)> change)
        {
            if (MediumOf(path) is null) return MediaOutcome.No($"{Show(path)} is of no medium a system pack names");
            if (HeldBy(path) is { } holder) return MediaOutcome.No($"{Show(path)} is in use by {holder}; it can be changed once that ends");
            using MediumLock? held = MediumLock.TryTake(path, Holder);
            if (held is null) return MediaOutcome.No($"{Show(path)} is in use by {HeldBy(path) ?? "another program"}");
            byte[] image = File.Exists(path) ? AtomicFile.TryRead(path) ?? Array.Empty<byte>() : Array.Empty<byte>();
            var (edited, words) = change(image);
            if (edited is null) return MediaOutcome.No(words);
            return AtomicFile.Write(path, edited) ? MediaOutcome.Ok(words) : MediaOutcome.No($"{Show(path)} could not be written");
        }

        private static MediaOutcome Write(string path, byte[] bytes, string words)
        {
            using MediumLock? held = MediumLock.TryTake(path, Holder);
            if (held is null) return MediaOutcome.No($"{Show(path)} is in use by {HeldBy(path) ?? "another program"}");
            return AtomicFile.Write(path, bytes) ? MediaOutcome.Ok(words) : MediaOutcome.No($"{Show(path)} could not be written");
        }
    }
}
