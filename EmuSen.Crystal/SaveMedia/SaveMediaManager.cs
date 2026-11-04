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

    // The card manager, emusen-savemedia over files, a format module per medium and Galaxia's lock; it needs no core - see EmuSen_SaveMedia.md §6, EmuSen_Backend.md §15.2.
    public static class SaveMediaManager
    {
        public const string Holder = "the card manager";

        // A file's medium, by its extension among the system packs' media.
        public static SaveMediumEntry? MediumOf(string path) => SaveMediumFormats.ForExtension(System.IO.Path.GetExtension(path));

        public static MediumCard? Open(string path) => SaveMediaNative.Open(path);

        // Who holds a card: its game in this program, another program, or null.
        public static string? HeldBy(string path) => MediumLock.HolderOf(path);

        // A save named by its id, or by its place in the card's list from 1.
        public static SaveEntry? Find(MediumListing listing, string save) =>
            listing.Saves.FirstOrDefault(s => s.Id == save) ?? (int.TryParse(save, out int n) && n >= 1 && n <= listing.Saves.Count ? listing.Saves[n - 1] : null);

        public static MediaOutcome CopySave(string from, string save, string to) => SaveMediaNative.CopySave(from, save, to);

        // A copy, read back, then the delete: the first card loses the save only once the second holds it.
        public static MediaOutcome MoveSave(string from, string save, string to) => SaveMediaNative.MoveSave(from, save, to);

        // As the console's manager deletes, which the medium's module says how.
        public static MediaOutcome DeleteSave(string card, string save) => SaveMediaNative.DeleteSave(card, save);

        // Every save of the given cards onto one card, all or none: refused whole past the card's capacity or where two cards hold different saves of one name - see EmuSen_SaveMedia.md §6.6.
        public static MediaOutcome Import(string to, IReadOnlyList<string> from) => SaveMediaNative.Import(to, from);

        // A new card: formatted where the medium's module can, else no file, which the game formats as a new card.
        public static MediaOutcome New(string path) => SaveMediaNative.New(path);

        // A whole card to a new name, read as bytes, which needs no module.
        public static MediaOutcome Duplicate(string from, string to) => SaveMediaNative.Duplicate(from, to);

        public static MediaOutcome Rename(string from, string to) => SaveMediaNative.Rename(from, to);
    }
}
