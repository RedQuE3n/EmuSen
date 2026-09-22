using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Galaxia.Library;

namespace EmuSen.Mistress.Library
{
    public enum MediaKind { SaveState, Screenshot }

    // One save state or screenshot, and the game it belongs to when the library has it - see EmuSen_Settings_Reference.md §4.35.
    public sealed record MediaItem(MediaKind Kind, string Path, string? PicturePath, string GameStem, string Label, DateTime When, RomEntry? Game)
    {
        public string Title => Game?.Title ?? GameStem;
    }

    // The files Mistress writes, read back into what OpenEmu's Save States and Screenshots views show.
    public static class MediaLibrary
    {
        // "<stem>.state" is slot 1, "<stem>.slotN.state" slot N, "<stem>.resume.state" where the game was left - see EmuSen_Galaxia.md §5.
        public static IReadOnlyList<MediaItem> SaveStates(string directory, IReadOnlyList<RomEntry> games)
        {
            if (!Directory.Exists(directory)) return Array.Empty<MediaItem>();
            ILookup<string, RomEntry> byStem = games.ToLookup(g => Path.GetFileNameWithoutExtension(g.FullPath), StringComparer.Ordinal);
            var items = new List<MediaItem>();
            foreach (string file in Directory.EnumerateFiles(directory, "*" + SaveLibrary.StateExtension))
            {
                string name = Path.GetFileNameWithoutExtension(file);
                (string stem, string label) = Parse(name);
                string picture = SaveLibrary.PicturePathFor(file);
                items.Add(new MediaItem(MediaKind.SaveState, file, File.Exists(picture) ? picture : null, stem, label,
                    File.GetLastWriteTime(file), byStem[stem].FirstOrDefault()));
            }
            return items.OrderBy(i => i.Title, StringComparer.OrdinalIgnoreCase).ThenByDescending(i => i.When).ToList();
        }

        public static (string Stem, string Label) Parse(string stateName)
        {
            if (stateName.EndsWith(".resume", StringComparison.Ordinal)) return (stateName[..^".resume".Length], "Where you left off");
            int dot = stateName.LastIndexOf(".slot", StringComparison.Ordinal);
            if (dot > 0 && int.TryParse(stateName[(dot + ".slot".Length)..], out int slot)) return (stateName[..dot], $"Slot {slot}");
            return (stateName, $"Slot {SaveLibrary.DefaultStateSlot}");
        }

        // "<stem> yyyy-MM-dd HH.mm.ss.fff.png", as the screenshot hotkey names them; any other picture there is shown under its own name.
        public static IReadOnlyList<MediaItem> Screenshots(string directory, IReadOnlyList<RomEntry> games)
        {
            if (!Directory.Exists(directory)) return Array.Empty<MediaItem>();
            ILookup<string, RomEntry> byStem = games.ToLookup(g => Path.GetFileNameWithoutExtension(g.FullPath), StringComparer.Ordinal);
            var items = new List<MediaItem>();
            foreach (string file in Directory.EnumerateFiles(directory, "*.png"))
            {
                string name = Path.GetFileNameWithoutExtension(file);
                string stem = name.Length > 24 && name[^24] == ' ' ? name[..^24] : name;
                DateTime when = File.GetLastWriteTime(file);
                items.Add(new MediaItem(MediaKind.Screenshot, file, file, stem, when.ToString("d MMM yyyy, HH:mm:ss"), when, byStem[stem].FirstOrDefault()));
            }
            return items.OrderByDescending(i => i.When).ToList();
        }
    }
}
