using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.Serenity.Shaders;

namespace EmuSen.Mistress.Pages
{
    // Consoles: every console a row, and its page with Engine (Cores' Make Default, one setting with two doors), Firmware, Memory cards and its console-wide settings - see EmuSen_BigPicture.md §52.4 and EmuSen_Interface.md §18.10.
    public sealed partial class SettingsScreen
    {
        public const string NotSupportedYet = "Not supported yet";

        // A page of choices once there are more than Left and Right step comfortably (EmuSen_Interface.md §18.11).
        public const int StepLimit = 6;

        private string NameOf(string console) => _host.Systems.FirstOrDefault(s => s.Console == console) is { } s ? s.DisplayName is { Length: > 0 } shown ? shown : s.Name : console;

        private string HostOf(string console) => _host.Systems.FirstOrDefault(s => s.Console == console)?.AttachedTo ?? console;

        private bool HasTelevision(string console) => CrtFilter.TelevisionConsoles.Contains(HostOf(console));

        // Each console once, those an engine runs first, each group in the order the consoles came out.
        private IReadOnlyList<SystemFacts> ConsoleList => _host.Systems.GroupBy(s => s.Console, StringComparer.OrdinalIgnoreCase).Select(g => g.First())
            .OrderBy(s => s.Engines.Count == 0).ThenBy(s => s.ReleaseYear).ThenBy(s => NameOf(s.Console), StringComparer.OrdinalIgnoreCase).ToList();

        private IReadOnlyList<Setting> ConsoleRows() => ConsoleList.Select(s =>
        {
            string console = s.Console, name = NameOf(console);
            bool runs = s.Engines.Count > 0;
            return Opens("console:" + console, name, runs ? ConsoleFacts(console) : $"EmuSen has no engine for the {name} yet.", runs ? null : NotSupportedYet,
                () => Push(name, () => ConsolePage(console), "console:" + console), () => ConsolePane(console), enabled: runs);
        }).ToList();

        private ScreenPane ConsolePane(string console)
        {
            string name = NameOf(console);
            if (_host.Systems.FirstOrDefault(s => s.Console == console) is { Engines.Count: 0 }) return Said($"EmuSen has no engine for the {name} yet.");
            var lines = new List<SettingsLine> { new($"{name} · {Count(_host.Games(console), "game", "games")} · engine {EngineName(console)}") };
            if (FirmwareLine(console) is { } firmware) lines.Add(firmware);
            var facts = new List<string>();
            if (HasTelevision(console)) facts.Add($"CRT profile {ScreenWord(console)}");
            foreach (SettingRow row in ConsoleSchema(console).Where(r => r.Kind == SettingRow.Choice).Take(1)) facts.Add($"{row.Label} {ValueText(console, row)}");
            if (facts.Count > 0) lines.Add(new SettingsLine(string.Join(" · ", facts)));
            lines.Add(new SettingsLine($"What every {name} game shares, unless the game's own options say otherwise.", SettingsTone.Muted));
            return new ScreenPane(lines);
        }

        private SettingRow? EngineRow(string console) => _host.Schema(console).FirstOrDefault(r => r.Key == RuntimeCatalog.EngineKey);

        private string EngineValue(string console) => _host.CoreValue(console, RuntimeCatalog.EngineKey) is { Length: > 0 } v ? v : EngineRow(console)?.Default ?? "";

        private string EngineName(string console) => EngineRow(console) is { } row ? Label(row, EngineValue(console)) : _host.Systems.FirstOrDefault(s => s.Console == console)?.Engines.FirstOrDefault() ?? "";

        private static string Label(SettingRow row, string value)
        {
            int at = row.Choices.ToList().IndexOf(value);
            return at >= 0 && at < row.ChoiceLabels.Count && row.ChoiceLabels[at] is { Length: > 0 } label ? label : value;
        }

        private IEnumerable<string> SystemsOf(string console) => _host.Systems.Where(s => s.Console == console).Select(s => s.Id);

        private IReadOnlyList<FirmwareFile> FirmwareOf(string console) =>
            SystemsOf(console).SelectMany(id => _host.Firmware(id) ?? []).GroupBy(f => f.File, StringComparer.OrdinalIgnoreCase).Select(g => g.First()).ToList();

        // A console's line in a list, the same facts its pane gives: its games, its engine, and its firmware where it has one to speak of.
        private string ConsoleFacts(string console) =>
            string.Join(" · ", new[] { Count(_host.Games(console), "game", "games"), EngineName(console), FirmwareShort(console) }.Where(f => f is { Length: > 0 }));

        // A file missing on a console that starts with a cartridge is a chip's, which only the games that carry the chip need; on one that starts empty every game does.
        private bool NeededByAll(string console) => _host.Systems.Any(s => s.Console == console && s.StartsEmpty);

        // One need of a console's firmware and the files that meet it: on a console that starts empty, the regional copies of one thing
        // (its BIOS) are one need, named by the last word their labels share; a cartridge console's chips are a need each.
        private sealed record FirmwareNeed(string Name, string Word, IReadOnlyList<FirmwareFile> Files)
        {
            public FirmwareFile? Found => Files.FirstOrDefault(f => f.Status == "present");
            public bool Missing => Found is null && Files.Any(f => f.Status == "missing");
            public string Recognised => Found is { } f ? System.IO.Path.GetFileNameWithoutExtension(f.File).ToUpperInvariant() : "";
        }

        private IReadOnlyList<FirmwareNeed> NeedsOf(string console)
        {
            IReadOnlyList<FirmwareFile> files = FirmwareOf(console);
            if (!NeededByAll(console)) return files.Select(f => new FirmwareNeed(f.Label, "firmware", [f])).ToList();
            static string Word(FirmwareFile f) => System.Text.RegularExpressions.Regex.Replace(f.Label, @"\s*\([^)]*\)", "").Split(' ', StringSplitOptions.RemoveEmptyEntries).LastOrDefault() ?? "firmware";
            return files.GroupBy(Word, StringComparer.OrdinalIgnoreCase).Select(g => new FirmwareNeed($"{NameOf(console)} {g.Key}", g.Key, g.ToList())).ToList();
        }

        private bool MissingForAll(string console) => NeededByAll(console) && NeedsOf(console).Any(n => n.Missing);

        // The firmware in a few words for a console's line in a list; nothing where it has none to speak of.
        private string? FirmwareShort(string console)
        {
            IReadOnlyList<FirmwareNeed> needs = NeedsOf(console);
            if (needs.Count == 0 || !NeededByAll(console)) return null;
            return needs.FirstOrDefault(n => n.Found is not null) is { } found ? $"✓ {found.Word} found"
                : needs.FirstOrDefault(n => n.Missing) is { } missing ? $"⚠ {missing.Word} not found" : "open replacement";
        }

        // The firmware in one line: found, needed by every game and not found, the open replacement in its place, or none needed.
        private SettingsLine? FirmwareLine(string console)
        {
            IReadOnlyList<FirmwareNeed> needs = NeedsOf(console);
            if (needs.Count == 0 || !NeededByAll(console)) return new SettingsLine("No firmware needed", SettingsTone.Success) { Mark = "✓" };
            if (needs.FirstOrDefault(n => n.Found is not null) is { } found) return new SettingsLine($"{found.Word} found", SettingsTone.Success) { Mark = "✓" };
            if (needs.FirstOrDefault(n => n.Missing) is { } missing) return new SettingsLine($"{missing.Word} not found: add it under Firmware", SettingsTone.Attention) { Mark = "⚠" };
            return new SettingsLine($"No {needs[0].Word} of your own: the open replacement runs in its place");
        }

        private string FirmwareValue(string console)
        {
            IReadOnlyList<FirmwareNeed> needs = NeedsOf(console);
            if (needs.Count == 0) return "none needed";
            if (!NeededByAll(console)) return needs.Any(n => n.Found is not null) ? "✓ Found" : needs.Any(n => n.Missing) ? "only some games" : "none needed";
            return needs.Any(n => n.Found is not null) ? "✓ Found" : MissingForAll(console) ? "⚠ Not found" : "Not found";
        }

        private string NeedStatus(string console, FirmwareNeed need) =>
            need.Found is not null ? $"✓ Found · {need.Recognised}"
            : need.Missing && NeededByAll(console) ? "⚠ Not found"
            : need.Missing ? "Only some games"
            : NeededByAll(console) ? "Not found" : "Not needed";

        // The keys the console page draws in rows of its own, so the schema's rows leave them out.
        private IReadOnlyList<SettingRow> ConsoleSchema(string console)
        {
            HashSet<string> own = [RuntimeCatalog.EngineKey, .. MediaOf(console).SelectMany(m => m.Slots.Select((_, i) => SlotKey(m, i)).Prepend(m.ModeKey))];
            Dictionary<string, string?> values = _host.Schema(console).ToDictionary(r => r.Key, r => (string?)(_host.CoreValue(console, r.Key) ?? r.Default));
            return _host.Schema(console).Where(r => !own.Contains(r.Key) && !r.Locked && (r.ShownWithKey is null || values.GetValueOrDefault(r.ShownWithKey) == r.ShownWithValue)).ToList();
        }

        // The setting a medium's slot is stored under, as the runtime's card manager names it (Platform/crystal/src/media.rs).
        private static string SlotKey(CardMedium m, int slot) => m.ModeKey switch
        {
            "Cards" => $"Card{slot + 1}",
            "ConsoleMemory" => $"ConsoleMemory{slot + 1}",
            _ => $"{m.ModeKey}.{slot + 1}",
        };

        private IReadOnlyList<CardMedium> MediaOf(string console)
        {
            try { return _host.Cards(console); }
            catch (InvalidOperationException) { return []; }
        }

        private string ValueText(string console, SettingRow row)
        {
            string value = _host.CoreValue(console, row.Key) is { Length: > 0 } v ? v : row.Default;
            return row.Kind switch
            {
                SettingRow.Switch => value is "true" or "1" or "on" ? On : Off,
                SettingRow.Count => value,
                _ => Label(row, value),
            };
        }

        // LB and RB on a console's page: the console before or after it in the list, its page in place of this one.
        public bool StepConsole(int by)
        {
            if (_section != Consoles || _pages.Count != 1) return false;
            List<string> runs = ConsoleList.Where(s => s.Engines.Count > 0).Select(s => s.Console).ToList();
            string from = _pages[0].From is { } key ? key["console:".Length..] : "";
            int at = runs.IndexOf(from);
            if (at < 0 || runs.Count < 2) return false;
            string to = runs[((at + by) % runs.Count + runs.Count) % runs.Count];
            _pages.RemoveAt(0);
            Push(NameOf(to), () => ConsolePage(to), "console:" + to);
            Read();
            return true;
        }

        private IReadOnlyList<Setting> ConsolePage(string console)
        {
            string name = NameOf(console);
            var rows = new List<Setting>();
            if (EngineRow(console) is { } engine && engine.Choices.Count > 0)
            {
                Setting stepped = SchemaRow(console, engine, "engine", "Engine");
                rows.Add(stepped with
                {
                    Pane = () => EnginePane(console),
                    Choose = () => Push("Engine", () => EngineChoices(console), "engine", choices: true),
                });
            }
            rows.Add(Opens("firmware", "Firmware", $"The firmware the {name}'s engines use, found by its content in your folders.", FirmwareValue(console),
                () => Push("Firmware", () => FirmwareRows(console), "firmware"), () => FirmwarePane(console)));
            IReadOnlyList<CardMedium> media = MediaOf(console);
            bool cards = media.Count > 0;
            rows.Add(Opens("cards", "Memory cards", cards ? $"The {name}'s cards: shared by every game as the console's were, or one per game." : $"The {name}'s games keep their saves in the cartridge.", null,
                () => Push("Memory cards", () => CardRows(console), "cards"), () => cards ? CardsPane(console) : Said($"The {name}'s games keep their saves in the cartridge; there is no card to choose."), enabled: cards));
            foreach (SettingRow row in ConsoleSchema(console)) rows.Add(SchemaRow(console, row, row.Key, row.Label));
            if (HasTelevision(console))
            {
                IReadOnlyList<string> filters = ScreenFilters.NamesFor(console);
                string current = _host.Graphics.Value(console, GraphicsKeys.ScreenFilter) ?? ScreenFilters.None;
                rows.Add(Choice("crtprofile", "CRT profile", $"The television the {name}'s picture is drawn on.", ["Off", .. CrtTiers], ScreenTier(console), i =>
                {
                    _host.Graphics.SetValue(console, GraphicsKeys.ScreenFilter, i == 0 ? ScreenFilters.None : CrtFilter.Name);
                    if (i > 0) _host.Graphics.SetParameter(console, CrtFilter.Name, "quality", (i - 1).ToString(CultureInfo.InvariantCulture));
                    _host.SaveGraphics(console);
                }));
                rows.Add(Opens("consolelevers", "Shader levers", $"The CRT's own levers for the {name}.", null, () => Push("Shader levers", () => LeverRows(console), "consolelevers"), enabled: current == CrtFilter.Name));
                rows.Add(GraphicsChoice(console, "consolecrop", "Crop overscan", "The whole picture, or the part a television showed.", GraphicsKeys.Overscan, [GraphicsKeys.NoOverscan, GraphicsKeys.TvOverscan], ["Whole picture", "TV overscan"]));
            }
            else
            {
                IReadOnlyList<string> filters = ScreenFilters.NamesFor(console);
                rows.Add(Choice("screen", "Screen", $"The {name}'s screen as it is drawn.", filters, filters.ToList().IndexOf(_host.Graphics.Value(console, GraphicsKeys.ScreenFilter) ?? ScreenFilters.None),
                    i => { _host.Graphics.SetValue(console, GraphicsKeys.ScreenFilter, filters[i]); _host.SaveGraphics(console); }));
            }
            rows.Add(GraphicsChoice(console, "consoleshape", "Picture shape", "The screen the console was made for, or each pixel square.", GraphicsKeys.PictureShape, [GraphicsKeys.TvShape, GraphicsKeys.SquarePixels], ["TV shape", "Square pixels"]));
            return rows;
        }

        private int ScreenTier(string console) => _host.Graphics.Value(console, GraphicsKeys.ScreenFilter) != CrtFilter.Name ? 0
            : 1 + Math.Clamp((int)ParseFloat(_host.Graphics.ParametersFor(console, CrtFilter.Name).GetValueOrDefault("quality"), 1), 0, 2);

        private string ScreenWord(string console) => ScreenTier(console) is 0 ? Off : CrtTiers[ScreenTier(console) - 1];

        private Setting GraphicsChoice(string console, string key, string label, string sentence, string setting, string[] values, string[] texts) =>
            Choice(key, label, sentence, texts, Math.Max(0, Array.IndexOf(values, _host.Graphics.Value(console, setting) ?? values[0])),
                i => { _host.Graphics.SetValue(console, setting, values[i]); _host.SaveGraphics(console); });

        // A row of the console's settings schema, stepped through its choices, its switch or its count.
        private Setting SchemaRow(string console, SettingRow row, string key, string label)
        {
            string value = _host.CoreValue(console, row.Key) is { Length: > 0 } v ? v : row.Default;
            void Store(string? to) => _host.SetCoreValue(console, row.Key, to);
            string sentence = row.Note is { Length: > 0 } note && row.HasNote ? $"{row.Hint} {note}" : row.Hint;
            switch (row.Kind)
            {
                case SettingRow.Switch:
                    bool on = value is "true" or "1" or "on";
                    return Choice(key, label, sentence, [On, Off], on ? 0 : 1, i => Store(i == 0 ? "true" : "false"));
                case SettingRow.Count:
                    int n = int.TryParse(value, out int parsed) ? parsed : row.Min;
                    return new Setting(new PageRow(key, label) { Kind = PageRowKind.Steps, Status = n.ToString(CultureInfo.InvariantCulture), Sentence = sentence }, null,
                        by => Store(Math.Clamp(n + by, row.Min, row.Max).ToString(CultureInfo.InvariantCulture)));
                default:
                    var texts = row.Choices.Select((c, i) => i < row.ChoiceLabels.Count && row.ChoiceLabels[i] is { Length: > 0 } l ? l : c).ToList();
                    return Choice(key, label, sentence, texts, row.Choices.ToList().IndexOf(value), i => Store(row.Choices[i]));
            }
        }

        private ScreenPane EnginePane(string console)
        {
            string name = NameOf(console), engine = EngineName(console);
            var lines = new List<SettingsLine> { new($"{engine}, the {name}'s engine: every {name} game opens on it unless the game's own options choose another.") };
            string[] others = EngineRow(console) is { } row ? row.Choices.Where(c => c != EngineValue(console)).Select(c => Label(row, c)).ToArray() : [];
            if (others.Length > 0) lines.Add(new SettingsLine($"{string.Join(", ", others)} can be chosen here too; this is the same setting as Cores' Make Default.", SettingsTone.Muted));
            return new ScreenPane(lines);
        }

        // The console's engines as a page of choices: the chosen one marked, A choosing and closing the page, B keeping what was chosen.
        private IReadOnlyList<Setting> EngineChoices(string console)
        {
            if (EngineRow(console) is not { } row) return [];
            string chosen = EngineValue(console);
            return row.Choices.Select(c => new Setting(new PageRow("engine:" + c, Label(row, c)) { Kind = PageRowKind.Action, Status = c == chosen ? "● Default" : "✓ Installed" },
                () => Said(c == chosen ? $"{Label(row, c)} runs every {NameOf(console)} game." : $"{Label(row, c)} would run every {NameOf(console)} game.", "A chooses; B keeps what was chosen."),
                Choose: () => { _host.SetCoreValue(console, RuntimeCatalog.EngineKey, c); Pop(); })).ToList();
        }

        private ScreenPane FirmwarePane(string console)
        {
            IReadOnlyList<FirmwareNeed> needs = NeedsOf(console);
            if (needs.Count == 0) return Said($"The {NameOf(console)} needs no firmware.", "EmuSen never downloads firmware.");
            var lines = new List<SettingsLine>();
            if (FirmwareLine(console) is { } line) lines.Add(line);
            if (!NeededByAll(console) && needs.Any(n => n.Missing)) lines.Add(new SettingsLine($"{Count(needs.Count(n => n.Missing), "chip file", "chip files")} only some games use not found.", SettingsTone.Muted));
            lines.Add(new SettingsLine("Recognised by its content in your firmware folders. EmuSen never downloads firmware; without your own, the open replacement runs most games.", SettingsTone.Muted));
            return new ScreenPane(lines);
        }

        // A row per need, its value Found with the copy recognised or Not found; the pane says what runs: your own copy when found, else the open replacement.
        private IReadOnlyList<Setting> FirmwareRows(string console)
        {
            string engine = EngineName(console);
            var rows = NeedsOf(console).Select(n => new Setting(
                new PageRow("fw:" + n.Name, n.Name) { Kind = PageRowKind.Action, Status = NeedStatus(console, n), Sentence = string.Join(", ", n.Files.Select(f => f.File)) },
                () => new ScreenPane(n.Found is { } found
                    ? [new SettingsLine($"Your own {n.Name} ({n.Recognised}) was recognised by its content, and {engine} uses it.", SettingsTone.Success) { Mark = "✓" }, new SettingsLine("EmuSen never downloads firmware.", SettingsTone.Muted)]
                    : n.Missing && NeededByAll(console)
                        ? [new SettingsLine($"No {n.Name} of your own was found, and the {NameOf(console)}'s games need one.", SettingsTone.Attention) { Mark = "⚠" }, new SettingsLine($"Put your own copy ({string.Join(", ", n.Files.Select(f => f.File))}) in your firmware folder, then Check again.", SettingsTone.Muted)]
                        : n.Missing
                            ? [new SettingsLine($"{n.Name}: needed only by the games that carry its chip; none of your own was found."), new SettingsLine($"Put your own copy ({n.Files[0].File}) in your firmware folder for those games, then Check again.", SettingsTone.Muted)]
                            : [new SettingsLine($"No {n.Name} of your own was found, so {engine} runs its open replacement in its place."), new SettingsLine($"Your own copy ({string.Join(", ", n.Files.Select(f => f.File))}) is used once it is found. EmuSen never downloads firmware.", SettingsTone.Muted)]))).ToList();
            rows.Add(Opens("fwfolder", "Folder", "Your firmware folder, searched before EmuSen's; only ever read.", _host.App.FirmwareDirectory is { Length: > 0 } d ? d : "EmuSen's folder", () => _host.Open(DoorFirmwareFolder)));
            rows.Add(Act("fwcheck", "Check again", "Reads the folders again, as after a file is put in one.", () => { }));
            return rows;
        }

        private ScreenPane CardsPane(string console)
        {
            CardMedium? card = MediaOf(console).FirstOrDefault();
            if (card is null) return Said("No cards.");
            string mode = ValueText(console, _host.Schema(console).FirstOrDefault(r => r.Key == card.ModeKey) ?? new SettingRow(card.ModeKey, "Mode", "", SettingRow.Choice, card.Mode, 0, 0, [], [], null, null, null, null));
            return Said($"{mode}: every {NameOf(console)} game sees the same cards when they are shared, as the console did; per game gives each game its own.",
                $"{Count(card.Cards.Count(c => !c.GamesOwn), CardMedium.InText(card.Noun), CardMedium.InText(card.Noun) + "s")} in the drawer");
        }

        private IReadOnlyList<Setting> CardRows(string console)
        {
            var rows = new List<Setting>();
            foreach (CardMedium medium in MediaOf(console))
            {
                if (_host.Schema(console).FirstOrDefault(r => r.Key == medium.ModeKey) is { } mode)
                    rows.Add(SchemaRow(console, mode, "mode:" + medium.Id, "Mode") with { Pane = () => CardsPane(console) });
                foreach (var (slot, i) in medium.Slots.Select((s, i) => (s, i)))
                    rows.Add(_host.Schema(console).FirstOrDefault(r => r.Key == SlotKey(medium, i)) is { } slotRow
                        ? SchemaRow(console, slotRow, $"slot:{medium.Id}:{i}", slot.Label)
                        : Act($"slot:{medium.Id}:{i}", slot.Label, "The card in this slot when a game starts.", () => { }, slot.Name));
                CardMedium m = medium;
                rows.Add(Opens("cards:" + medium.Id, "Cards", "Every card in the drawer, and the saves on each.", Count(medium.Cards.Count(c => !c.GamesOwn), "card", "cards"),
                    () => Push($"{m.Title}s", () => m.Cards.Where(c => !c.GamesOwn).Select(c => Act("card:" + c.Path, c.Label, "A card in the drawer.", () => { }, c.HeldBy is { } held ? $"in {held}" : null)).ToList(), "cards:" + m.Id)));
                rows.Add(Opens("saves:" + medium.Id, "Saves on the cards", "Each card's saves, copied, moved or deleted.", null, () => _host.Open(DoorCardSaves + console)));
                rows.Add(Asks("new:" + medium.Id, $"New {CardMedium.InText(medium.Noun)}", "A blank card in the drawer, named for you.", () => _host.NewCard(console)));
            }
            return rows;
        }
    }
}
