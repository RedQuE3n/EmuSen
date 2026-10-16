using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores.Native;
using EmuSen.Crystal.SaveMedia;
using EmuSen.Galaxia.Library;

namespace EmuSen.Cores
{
    // One medium's choice: its mode, and the name stored for each slot, a slot's default where none is - see EmuSen_SaveMedia.md §3.3.
    public sealed record CardChoice(string Mode, IReadOnlyList<string> Names)
    {
        public CardChoice(string mode, string slot1, string slot2) : this(mode, new[] { slot1, slot2 }) { }

        public static readonly CardChoice PerGame = new(MemoryCards.PerGame, "Card 1", "Card 2");

        // Whether the medium is kept apart from the game, so the machine is made with its kind's create flag.
        public bool Apart => Mode is MemoryCards.Shared or MemoryCards.Named;

        public string Slot1 => Names.Count > 0 ? Names[0] : "";
        public string Slot2 => Names.Count > 1 ? Names[1] : "";

        public bool Equals(CardChoice? other) => other is not null && Mode == other.Mode && Names.SequenceEqual(other.Names);

        public override int GetHashCode() => HashCode.Combine(Mode, Names.Count);
    }

    // One medium of a console: its slots as core info lists them, its words from the system pack, its keys and files - see EmuSen_SaveMedia.md §3.2, §3.3.
    public sealed record SaveMedium(string Console, string Id, CoreBatteryKind Kind, string Extension, string Title, string Noun, IReadOnlyList<CoreBatteryFile> Slots, int Ordinal, SaveMediumEntry? Entry)
    {
        public bool IsConsole => Kind == CoreBatteryKind.Console;

        // The first medium of each kind keeps its kind's plain keys, 1.7's Cards and Card<n> for the removable kind.
        public string ModeKey => Ordinal > 0 ? $"Media.{Id}" : IsConsole ? MemoryCards.ConsoleModeKey : MemoryCards.ModeKey;

        public string SlotKey(int slot) => Ordinal > 0 ? $"Media.{Id}.{slot + 1}" : IsConsole ? $"{MemoryCards.ConsoleModeKey}{slot + 1}" : $"Card{slot + 1}";

        public string DefaultName(int slot) => $"{Noun} {slot + 1}";

        // The file a slot holds under the choice: the stored name under Named, the slot's default under Shared.
        public string NameFor(CardChoice choice, int slot) => choice.Mode == MemoryCards.Named && slot < choice.Names.Count ? choice.Names[slot] : DefaultName(slot);

        // The slot's words: the core's label, else its number.
        public string SlotLabel(int slot) => Slots[slot].Label ?? (Slots.Count == 1 ? Title : $"Slot {slot + 1}");

        public string PathOf(string name) => SaveLibrary.MediumPathFor(Console, IsConsole, name, Extension);

        // Every name the drawer offers: the noun's first eight, then each file the folder holds.
        public IReadOnlyList<string> Names()
        {
            var names = Enumerable.Range(0, MemoryCards.NamesOffered).Select(DefaultName).ToList();
            names.AddRange(SaveLibrary.MediumNames(Console, IsConsole, Extension).Where(n => MemoryCards.IsName(n) && !names.Contains(n, StringComparer.OrdinalIgnoreCase)));
            return names;
        }
    }

    // The files of a game's save media at one load, the flags its machine is made with, and the locks held on them - see EmuSen_SaveMedia.md §3.3, §3.5.
    public sealed class MediaPlacement : IDisposable
    {
        private readonly List<MediumLock> _locks = new();

        public Dictionary<uint, BatterySave> Files { get; } = new();
        // The console media's slots a new machine finds formatted where no file exists yet - see EmuSen_SaveMedia.md §10 Q2.
        public Dictionary<uint, ISaveMediumFormat> Fresh { get; } = new();
        public bool RemovableApart { get; internal set; }
        public bool ConsoleApart { get; internal set; }
        public List<string> Notices { get; } = new();

        internal void Hold(MediumLock taken) => _locks.Add(taken);

        public void Dispose()
        {
            foreach (MediumLock held in _locks) held.Dispose();
            _locks.Clear();
        }
    }

    // The runtime's card mode for every medium of every console: Per game, Shared or Named - see EmuSen_SaveMedia.md §3.
    public static class MemoryCards
    {
        // graphics.json keys of the console's tab, which no core declares, so none is handed them; 1.7's for the first removable medium - see EmuSen_Settings_Reference.md §4.114.
        public const string ModeKey = "Cards", ConsoleModeKey = "ConsoleMemory";
        public static readonly string[] SlotKeys = { "Card1", "Card2" };

        public const string PerGame = "game", Shared = "shared", Named = "named";

        // The noun's first eight names are always offered under Named, beside every file the folder already holds.
        public const int NamesOffered = 8;

        public static string DefaultName(int slot) => $"Card {slot + 1}";

        // The removable files of a console's engine, as its core info lists them; empty for a console with none.
        public static IReadOnlyList<CoreBatteryFile> RemovableFor(string console) =>
            CoreCatalog.EngineSystem(console)?.Battery?.Where(b => b.Removable).ToArray() ?? Array.Empty<CoreBatteryFile>();

        public static bool OfferedFor(string console) => MediaFor(console).Count > 0;

        // The extension of a suffix, its last part: ".mcr" of ".2.mcr".
        public static string ExtensionOf(string suffix) => Path.GetExtension(suffix) is { Length: > 1 } e ? e : suffix;

        // A console's media, as its engine's core info lists them.
        public static IReadOnlyList<SaveMedium> MediaFor(string console) => MediaOf(console, CoreCatalog.EngineSystem(console));

        // The removable and console slots of a system grouped by medium, a core of 1.7's by kind and extension - see EmuSen_SaveMedia.md §3.2.
        public static IReadOnlyList<SaveMedium> MediaOf(string console, CoreSystem? system)
        {
            var media = new List<SaveMedium>();
            var slots = (system?.Battery ?? Array.Empty<CoreBatteryFile>()).Where(b => b.Kind != CoreBatteryKind.Game);
            foreach (var group in slots.GroupBy(b => b.Medium ?? $"{b.Kind.ToString().ToLowerInvariant()}{ExtensionOf(b.Suffix)}"))
            {
                CoreBatteryFile first = group.First();
                string extension = ExtensionOf(first.Suffix);
                SaveMediumEntry? entry = SaveMediumFormats.Entry(first.Medium) ?? SaveMediumFormats.ForExtension(extension, system?.Id);
                bool inside = first.Kind == CoreBatteryKind.Console;
                media.Add(new SaveMedium(console, group.Key, first.Kind, extension,
                    entry?.Title ?? (inside ? "Console memory" : "Memory card"), entry?.Noun ?? (inside ? "Memory" : "Card"),
                    group.ToArray(), media.Count(m => m.Kind == first.Kind), entry));
            }
            return media;
        }

        // The mode a medium has when nothing is stored: Shared where a format module gives it the import, Per game otherwise - see EmuSen_SaveMedia.md §9, §10 Q2.
        public static string DefaultMode(SaveMedium? medium) => medium is { Kind: CoreBatteryKind.Removable or CoreBatteryKind.Console, Entry.Format: not null } ? Shared : PerGame;

        // The key recording that the import was offered for a medium, beside its mode's.
        public static string OfferedKey(SaveMedium medium) => medium.ModeKey + "Offered";

        // The medium whose import a frontend offers before a game starts: Shared by default with nothing stored, its first shared card absent, games' own cards present, and not offered before - see EmuSen_SaveMedia.md §9.
        public static SaveMedium? ImportToOffer(IReadOnlyList<SaveMedium> media, Func<string, string?> stored) =>
            media.FirstOrDefault(m => stored(m.ModeKey) is null && DefaultMode(m) == Shared && stored(OfferedKey(m)) is null
                && !File.Exists(m.PathOf(m.DefaultName(0))) && CardsOf(m).Any(c => c.GamesOwn));

        // 1.7's reading: the console's first removable medium, two slots named Card n.
        public static CardChoice Read(Func<string, string?> stored) => Read(stored, null);

        // The stored choice: a mode it does not know read as Per game, nothing stored as the medium's default, a name that is no file name as the slot's default.
        public static CardChoice Read(Func<string, string?> stored, SaveMedium? medium)
        {
            string? kept = stored(medium?.ModeKey ?? ModeKey);
            string mode = kept is Shared or Named or PerGame ? kept : kept is null ? DefaultMode(medium) : PerGame;
            int count = medium?.Slots.Count ?? 2;
            string Name(int slot) => stored(medium?.SlotKey(slot) ?? SlotKeys[slot]) is { Length: > 0 } n && IsName(n) ? n : medium?.DefaultName(slot) ?? DefaultName(slot);
            return new CardChoice(mode, Enumerable.Range(0, count).Select(Name).ToArray());
        }

        // Every medium's stored choice, by medium id.
        public static IReadOnlyDictionary<string, CardChoice> Stored(IReadOnlyList<SaveMedium> media, Func<string, string?> stored) =>
            media.ToDictionary(m => m.Id, m => Read(stored, m));

        // A file of a medium the card manager lists: a card in the drawer, or a game's own, labelled by its game and slot.
        public sealed record MediumFile(string Path, string Name, string Label, bool GamesOwn);

        // The drawer's cards of a medium, then each game's own, as the files hold them - see EmuSen_SaveMedia.md §6.5.
        public static IReadOnlyList<MediumFile> CardsOf(SaveMedium medium)
        {
            var cards = SaveLibrary.MediumNames(medium.Console, medium.IsConsole, medium.Extension).Select(n => new MediumFile(medium.PathOf(n), n, n, false)).ToList();
            string own = Path.Combine(DataStore.Saves, medium.Console);
            if (!Directory.Exists(own)) return cards;
            var suffixes = medium.Slots.Select((s, n) => (s.Suffix, Slot: n)).OrderByDescending(s => s.Suffix.Length).ToArray();
            foreach (string file in Directory.EnumerateFiles(own).Where(f => string.Equals(Path.GetExtension(f), medium.Extension, StringComparison.OrdinalIgnoreCase)).Order(StringComparer.OrdinalIgnoreCase))
            {
                string name = Path.GetFileName(file);
                var (suffix, slot) = suffixes.FirstOrDefault(s => name.EndsWith(s.Suffix, StringComparison.OrdinalIgnoreCase) && name.Length > s.Suffix.Length);
                string game = suffix is null ? Path.GetFileNameWithoutExtension(file) : name[..^suffix.Length];
                cards.Add(new MediumFile(file, game, slot > 0 ? $"{game}, {medium.SlotLabel(slot)} (the game's own)" : $"{game} (the game's own)", true));
            }
            return cards;
        }

        // The first of the noun's names that no file in the drawer has.
        public static string NextName(SaveMedium medium)
        {
            var taken = SaveLibrary.MediumNames(medium.Console, medium.IsConsole, medium.Extension).ToHashSet(StringComparer.OrdinalIgnoreCase);
            for (int n = 0; ; n++)
                if (!taken.Contains(medium.DefaultName(n))) return medium.DefaultName(n);
        }

        // A name that is one file in the folder: no separator, no leading dot, nothing a path would read otherwise.
        public static bool IsName(string name) =>
            name.Trim() == name && name.Length > 0 && !name.StartsWith('.') && name.IndexOfAny(Path.GetInvalidFileNameChars()) < 0 && name.IndexOfAny(new[] { '/', '\\' }) < 0;

        // 1.7's list, the first removable medium's names.
        public static IReadOnlyList<string> NamesFor(string console) =>
            MediaFor(console).FirstOrDefault(m => m.Kind == CoreBatteryKind.Removable)?.Names() ?? Enumerable.Range(0, NamesOffered).Select(DefaultName).ToArray();

        // A medium's rows: its mode, and under Named a row for each slot - see EmuSen_Settings_Reference.md §4.114.
        public static (CoreSetting Mode, IReadOnlyList<CoreSetting> Slots) RowsFor(SaveMedium medium)
        {
            var names = medium.Names();
            string noun = medium.Noun.ToLowerInvariant();
            var mode = new CoreSetting(medium.ModeKey, medium.Title,
                $"Per game: each game keeps its own, as before. Shared: one {noun} in each slot, used by every game, as a console with it always there. Named: a drawer of them, each a file of the save folder, and you choose which is in each slot. A save state never rolls a shared or named {noun} back. Changing this moves, copies and deletes nothing: each game's own are used again when Per game is chosen again. Takes effect when a game is next loaded.",
                CoreSettingKind.Choice, DefaultMode(medium), Choices: new[] { PerGame, Shared, Named }, ChoiceLabels: new[] { "Per game", "Shared", "Named" });
            CoreSetting Slot(int slot) => new(medium.SlotKey(slot), $"{medium.Title}: {medium.SlotLabel(slot)}",
                $"Which {noun} is here under Named; a name with no file yet is a new, blank one, which the game formats. Takes effect when a game is next loaded.",
                CoreSettingKind.Choice, medium.DefaultName(slot), Choices: names, Note: slot > 0 ? value => SameNameNote(medium, slot, value) : null);
            return (mode, Enumerable.Range(0, medium.Slots.Count).Select(Slot).ToArray());
        }

        // The rows of the console's tab, medium after medium.
        public static IReadOnlyList<CoreSetting> SettingsFor(string console) =>
            MediaFor(console).SelectMany(m => { var (mode, slots) = RowsFor(m); return slots.Prepend(mode); }).ToArray();

        // Said under a slot's row when an earlier slot names the same file, which cannot be in two slots at once.
        private static string? SameNameNote(SaveMedium medium, int slot, Func<string, string> value) =>
            value(medium.ModeKey) == Named && Enumerable.Range(0, slot).Any(s => string.Equals(value(medium.SlotKey(s)), value(medium.SlotKey(slot)), StringComparison.OrdinalIgnoreCase))
                ? $"The same {medium.Noun.ToLowerInvariant()} cannot be in two slots: this slot keeps the game's own while they match." : null;

        // Each slot's file at a load: under Shared or Named its medium's file, locked for holder, else own(slot) - see EmuSen_SaveMedia.md §3.3, §3.5.
        public static MediaPlacement Place(string console, CoreSystem? system, IReadOnlyDictionary<string, CardChoice> choices, string holder, Func<CoreBatteryFile, BatterySave> own)
        {
            var placed = new MediaPlacement();
            var taken = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
            foreach (SaveMedium medium in MediaOf(console, system))
            {
                CardChoice choice = choices.TryGetValue(medium.Id, out CardChoice? c) ? c : new CardChoice(PerGame, Array.Empty<string>());
                for (int slot = 0; slot < medium.Slots.Count; slot++)
                {
                    CoreBatteryFile file = medium.Slots[slot];
                    if (medium.IsConsole && medium.Entry?.Format is { } format) placed.Fresh[file.Which] = format;
                    BatterySave? apart = choice.Apart ? Apart(console, medium, medium.NameFor(choice, slot), slot, holder, taken, placed) : null;
                    if (apart is not null)
                    {
                        placed.Files[file.Which] = apart;
                        if (medium.IsConsole) placed.ConsoleApart = true; else placed.RemovableApart = true;
                        continue;
                    }
                    BatterySave save = own(file);
                    placed.Files[file.Which] = save;
                    if (save.Path is not null && MediumLock.TryTake(save.Path, holder, shared: true) is { } held) placed.Hold(held);
                }
            }
            return placed;
        }

        // The medium's file named name for a slot, locked; null, with the reason noted, where an earlier slot or another program has it.
        private static BatterySave? Apart(string console, SaveMedium medium, string name, int slot, string holder, HashSet<string> taken, MediaPlacement placed)
        {
            BatterySave save = BatterySave.Medium(console, medium.IsConsole, name, medium.Extension);
            string key = medium.PathOf(name);
            if (!taken.Add(key))
            {
                Note(placed, $"{name} is already in another slot; {medium.SlotLabel(slot)} keeps the game's own {medium.Noun.ToLowerInvariant()}.");
                return null;
            }
            if (save.Path is null) return save;
            if (MediumLock.TryTake(save.Path, holder, shared: true) is not { } held)
            {
                Note(placed, $"{name} is in use by {MediumLock.HolderOf(save.Path) ?? "another program"}; {medium.SlotLabel(slot)} keeps the game's own {medium.Noun.ToLowerInvariant()} until it is free.");
                return null;
            }
            placed.Hold(held);
            return save;
        }

        private static void Note(MediaPlacement placed, string words)
        {
            Console.WriteLine($"[MemoryCards] {words}");
            placed.Notices.Add(words);
        }
    }
}
