using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores.Native;
using EmuSen.Galaxia.Library;

namespace EmuSen.Cores
{
    // Which card file is in each slot: the game's own, one shared by every game, or one the player names - see Diamond_Cards.md §2.
    public sealed record CardChoice(string Mode, string Slot1, string Slot2)
    {
        public static readonly CardChoice PerGame = new(MemoryCards.PerGame, MemoryCards.DefaultName(0), MemoryCards.DefaultName(1));

        // Whether the cards are kept apart from the game, so the machine is made with EMUSEN_CREATE_SHARED_REMOVABLE.
        public bool Apart => Mode is MemoryCards.Shared or MemoryCards.Named;

        // The card in slot n, counted from 0: under Shared always "Card n+1", under Named cards the player's choice.
        public string NameFor(int slot) => Mode == MemoryCards.Named && slot < 2 ? (slot == 0 ? Slot1 : Slot2) : MemoryCards.DefaultName(slot);
    }

    // The runtime's card mode, offered for a console whose engine lists a removable battery file - see Diamond_Cards.md §9.
    public static class MemoryCards
    {
        // graphics.json keys of the console's tab, which no core declares, so none is handed them - see EmuSen_Settings_Reference.md §4.114.
        public const string ModeKey = "Cards";
        public static readonly string[] SlotKeys = { "Card1", "Card2" };

        public const string PerGame = "game", Shared = "shared", Named = "named";

        // Card 1 to Card 8 are always offered under Named cards, beside every card the folder already holds.
        public const int NamesOffered = 8;

        public static string DefaultName(int slot) => $"Card {slot + 1}";

        // The removable files of a console's engine, as its core info lists them; empty for a console with none.
        public static IReadOnlyList<CoreBatteryFile> RemovableFor(string console) =>
            CoreCatalog.DiscoveredSystem(console)?.Battery?.Where(b => b.Removable).ToArray() ?? Array.Empty<CoreBatteryFile>();

        public static bool OfferedFor(string console) => RemovableFor(console).Count > 0;

        // The cards' extension, the last part of the first removable file's suffix: ".mcr" of ".2.mcr".
        public static string ExtensionOf(string suffix) => Path.GetExtension(suffix) is { Length: > 1 } e ? e : suffix;

        // The stored choice, a mode it does not know read as Per game and an empty name as the slot's default.
        public static CardChoice Read(Func<string, string?> stored)
        {
            string mode = stored(ModeKey) is Shared or Named ? stored(ModeKey)! : PerGame;
            string Name(int slot) => stored(SlotKeys[slot]) is { Length: > 0 } n && IsName(n) ? n : DefaultName(slot);
            return new CardChoice(mode, Name(0), Name(1));
        }

        // A name that is one file in the folder: no separator, no leading dot, nothing a path would read otherwise.
        public static bool IsName(string name) =>
            name.Trim() == name && name.Length > 0 && !name.StartsWith('.') && name.IndexOfAny(Path.GetInvalidFileNameChars()) < 0 && name.IndexOfAny(new[] { '/', '\\' }) < 0;

        // Card 1 to Card 8, then each card the console's folder holds that is not one of them.
        public static IReadOnlyList<string> NamesFor(string console)
        {
            var names = Enumerable.Range(0, NamesOffered).Select(DefaultName).ToList();
            if (RemovableFor(console).FirstOrDefault() is { } first)
                names.AddRange(SaveLibrary.CardNames(console, ExtensionOf(first.Suffix)).Where(n => IsName(n) && !names.Contains(n, StringComparer.OrdinalIgnoreCase)));
            return names;
        }

        // The rows of the console's tab: the mode, and under Named cards the card in each slot - see EmuSen_Settings_Reference.md §4.114.
        public static IReadOnlyList<CoreSetting> SettingsFor(string console)
        {
            if (!OfferedFor(console)) return Array.Empty<CoreSetting>();
            var names = NamesFor(console);
            var mode = new CoreSetting(ModeKey, "Memory cards",
                "Per game: each game keeps its own cards, as before. Shared: one card in slot 1, and one in slot 2 when that slot holds a card, used by every game, as a console with its cards always plugged in. Named cards: a drawer of cards, each a file of the Cards folder, and you choose which is in each slot. A save state never rolls a shared or named card back. Changing this moves, copies and deletes nothing: each game's own cards are used again when Per game is chosen again. Takes effect when a game is next loaded.",
                CoreSettingKind.Choice, PerGame, Choices: new[] { PerGame, Shared, Named }, ChoiceLabels: new[] { "Per game", "Shared", "Named cards" });
            CoreSetting Slot(int slot) => new(SlotKeys[slot], $"Card in slot {slot + 1}",
                $"Which of your cards is in slot {slot + 1} under Named cards; a name with no file yet is a new, blank card, which the game formats. Takes effect when a game is next loaded.",
                CoreSettingKind.Choice, DefaultName(slot), Choices: names, Note: slot == 1 ? SameCardNote : null);
            return new[] { mode, Slot(0), Slot(1) };
        }

        // Said under slot 2's row when both slots name one card, which cannot be two cards at once.
        private static string? SameCardNote(Func<string, string> value) =>
            value(ModeKey) == Named && string.Equals(value(SlotKeys[0]), value(SlotKeys[1]), StringComparison.OrdinalIgnoreCase)
                ? "The same card cannot be in both slots: slot 2 keeps the game's own card while they match." : null;

        // The file of a removable battery file at its slot under the choice, or null where it is the game's own.
        public static BatterySave? CardFor(string console, CoreBatteryFile file, int slot, CardChoice choice)
        {
            if (!file.Removable || !choice.Apart || slot < 0) return null;
            string name = choice.NameFor(slot);
            for (int earlier = 0; earlier < slot; earlier++)
                if (string.Equals(choice.NameFor(earlier), name, StringComparison.OrdinalIgnoreCase))
                {
                    Console.WriteLine($"[MemoryCards] {name} is already in slot {earlier + 1}; slot {slot + 1} keeps the game's own card.");
                    return null;
                }
            return BatterySave.Card(console, name, ExtensionOf(file.Suffix));
        }

        // A removable file's slot: its place among the removable files listed, from 0; -1 for one that is not listed so.
        public static int SlotOf(IEnumerable<CoreBatteryFile> listed, uint which) =>
            listed.Where(b => b.Removable).Select(b => b.Which).ToList().IndexOf(which);
    }
}
