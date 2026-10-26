using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using Avalonia.Controls;
using Avalonia.Layout;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Library;

namespace EmuSen.Mistress.Views
{
    // A console's save medium as the runtime's card manager lists it.
    public sealed record CardMedium(string Console, string Id, string Title, string Noun, string Extension, bool ConsoleKind, int? Capacity, string? Unit, string Mode, string ModeKey,
        IReadOnlyList<(string Label, string Name, string Path)> Slots, string NextName, bool ImportDue, IReadOnlyList<CardFile> Cards)
    {
        public bool Readable => Capacity is not null;

        // Where a card of this name lives in the drawer, by the save library's names.
        public string PathOf(string name) => SaveLibrary.MediumPathFor(Console, ConsoleKind, name, Extension);

        // Each medium of a console, from cards_op's listing.
        public static IReadOnlyList<CardMedium> Of(CrystalRuntime runtime, string console)
        {
            using JsonDocument list = JsonDocument.Parse(runtime.CardsOp(CrystalAbi.CardsList, JsonSerializer.Serialize(new { console })));
            static string? Text(JsonElement e, string name) => e.TryGetProperty(name, out JsonElement v) && v.ValueKind == JsonValueKind.String ? v.GetString() : null;
            return list.RootElement.EnumerateArray().Select(m => new CardMedium(console, Text(m, "medium") ?? "", Text(m, "title") ?? "", Text(m, "noun") ?? "", Text(m, "extension") ?? "",
                Text(m, "kind") == "console", m.GetProperty("capacity").ValueKind == JsonValueKind.Number ? m.GetProperty("capacity").GetInt32() : null, Text(m, "unit"),
                Text(m, "mode") ?? "game", Text(m, "mode_key") ?? "", m.GetProperty("slots").EnumerateArray().Select(s => (Text(s, "label") ?? "", Text(s, "name") ?? "", Text(s, "path") ?? "")).ToArray(),
                Text(m, "next_name") ?? "", m.GetProperty("import_due").GetBoolean(),
                m.GetProperty("cards").EnumerateArray().Select(c => new CardFile(Text(c, "path") ?? "", Text(c, "name") ?? "", Text(c, "label") ?? "", c.GetProperty("games_own").GetBoolean(), Text(c, "held_by"))).ToArray())).ToArray();
        }

        // A medium's words inside a sentence: each word lower case unless it is an initialism, as "Controller Pak" reads "controller pak" and "SRAM" stays.
        public static string InText(string words) =>
            string.Join(' ', words.Split(' ').Select(w => w.Length > 1 && w.All(c => !char.IsLetter(c) || char.IsUpper(c)) ? w : w.ToLowerInvariant()));
    }

    // A card in the drawer, or a game's own, and the session that holds it, if one does.
    public sealed record CardFile(string Path, string Name, string Label, bool GamesOwn, string? HeldBy);

    // An action's answer from the card manager: done or not, and its words.
    public sealed record CardOutcome(bool Done, string Words)
    {
        public static CardOutcome Of(CrystalRuntime runtime, uint op, object args)
        {
            try
            {
                using JsonDocument answer = JsonDocument.Parse(runtime.CardsOp(op, JsonSerializer.Serialize(args)));
                return new CardOutcome(answer.RootElement.GetProperty("done").GetBoolean(), answer.RootElement.GetProperty("words").GetString() ?? "");
            }
            catch (CrystalException e) { return new CardOutcome(false, e.Message); }
        }
    }

    // Every console's save media: each medium's cards, the saves on one, and copying, moving, deleting and importing them through the runtime's card manager - see EmuSen_SaveMedia.md §6.5 and EmuSen_Backend.md §14.13.
    public sealed class MemoryCardsWindow : ToolWindow
    {
        private readonly Tabs _tabs = new() { Name = "MemoryCardTabs" };
        private readonly HintText _status = new() { Name = "MemoryCardsStatus" };
        private readonly List<MediumPane> _panes = new();
        private readonly CrystalRuntime _runtime;

        // Parameterless constructor exists only for tooling - real code always uses the one below.
        public MemoryCardsWindow() : this(new GraphicsConfig(), null) { }

        public MemoryCardsWindow(GraphicsConfig config, string? selectedConsole)
        {
            _runtime = CoreSettings.Runtime;
            Title = "Memory Cards";
            Width = 720;
            Height = 600;
            MinWidth = 560;
            MinHeight = 420;
            CanResize = true;

            var media = RuntimeCatalog.ConsoleNames.Concat(RuntimeCatalog.UnshelvedConsoles).Select(c => (Console: c, Media: CardMedium.Of(_runtime, c))).Where(c => c.Media.Count > 0).ToList();
            Consoles = media.Select(c => c.Console).ToList();
            foreach (var (console, list) in media) _tabs.Add(console, new ScrollViewer { Content = BuildConsole(list) });
            int selected = selectedConsole is null ? -1 : Consoles.ToList().IndexOf(selectedConsole);
            if (selected >= 0) _tabs.SelectedIndex = selected;

            Control hint = Ui.Hint("Each console's memory cards and the like: the cards in its drawer and each game's own, and the saves on them. A game that is running keeps its cards until it closes.");
            Button close = Ui.Button("Close", Close);
            close.Name = "MemoryCardsClose";
            Control foot = Ui.Stack(8, _status, Ui.Buttons(close));
            hint.Margin = new Avalonia.Thickness(0, 0, 0, 10);
            foot.Margin = new Avalonia.Thickness(0, 10, 0, 0);
            DockPanel.SetDock(hint, Dock.Top);
            DockPanel.SetDock(foot, Dock.Bottom);
            var dock = new DockPanel { Margin = new Avalonia.Thickness(16), LastChildFill = true };
            dock.Children.Add(hint);
            dock.Children.Add(foot);
            dock.Children.Add(Consoles.Count > 0 ? _tabs : new EmptyState { Message = "No memory cards", Detail = "No console's core here declares a memory card, a pak or a memory of its own." });
            Content = dock;
        }

        // The consoles shown, one tab each, in release order.
        public IReadOnlyList<string> Consoles { get; }

        // What the last action did or why it did nothing.
        public string? Status => _status.Text;

        private Control BuildConsole(IReadOnlyList<CardMedium> media)
        {
            var panel = new StackPanel { Spacing = 18, Margin = new Avalonia.Thickness(0, 8, 0, 0) };
            foreach (CardMedium medium in media)
            {
                var pane = new MediumPane(this, medium);
                _panes.Add(pane);
                panel.Children.Add(pane.View);
            }
            return panel;
        }

        // The pane of a medium; every pane drawn again after an action, as a copy changes the card it lands on.
        internal IReadOnlyList<MediumPane> Panes => _panes;

        private void Done(CardOutcome outcome)
        {
            _status.Text = outcome.Words;
            foreach (MediumPane pane in _panes) pane.Refresh();
        }

        // One medium: its mode, a chooser of its cards, the saves on the chosen one, and the actions on them.
        internal sealed class MediumPane
        {
            private readonly MemoryCardsWindow _owner;
            private CardMedium _medium;
            private readonly Dropdown _card;
            private readonly Dropdown _target;
            private readonly HintText _about = new();
            private readonly StackPanel _saves = Ui.Stack(4);
            private readonly TextBox _name = new() { MinWidth = 200, HorizontalAlignment = HorizontalAlignment.Left };
            private readonly Button _copy, _move, _delete, _create, _import;
            private string? _chosenCard;
            private string? _chosenSave;

            public MediumPane(MemoryCardsWindow owner, CardMedium medium)
            {
                _owner = owner;
                _medium = medium;
                string name = $"{medium.Console}.{medium.Id}";
                _card = new Dropdown { Name = name + ".Card", MinWidth = 260, HorizontalAlignment = HorizontalAlignment.Left };
                _target = new Dropdown { Name = name + ".Target", MinWidth = 260, HorizontalAlignment = HorizontalAlignment.Left };
                _card.Chose += chosen => { if (!_filling && chosen is string shown) { _chosenCard = PathOf(shown); _chosenSave = null; Refresh(); } };
                _target.Chose += _ => { if (!_filling) Enable(); };
                _about.Name = name + ".About";
                _saves.Name = name + ".Saves";
                _name.Name = name + ".NewName";
                _copy = Act("Copy Save", name + ".Copy", () => Transfer(move: false));
                _move = Act("Move Save", name + ".Move", () => Transfer(move: true));
                _delete = Act("Delete Save", name + ".Delete", () => _owner.Done(CardOutcome.Of(_owner._runtime, CrystalAbi.CardsDelete, new { card = _chosenCard, save = _chosenSave })));
                _create = Act("New Card", name + ".New", Create);
                _import = Act("Import Every Game's Saves", name + ".Import", Import);
                View = Ui.Stack(8,
                    Ui.Header(medium.Title),
                    new FieldRow { Label = "Card", Hint = "A card in the drawer, or a game's own.", Content = Ui.Stack(4, _card, _about) },
                    new FieldRow { Label = "Saves", Hint = "Choose a save to copy, move or delete it.", Content = _saves },
                    new FieldRow { Label = "Copy or move to", Content = Ui.Stack(8, _target, Ui.Row(8, _copy, _move, _delete)) },
                    new FieldRow { Label = "New card", Hint = $"A name for a new {CardMedium.InText(medium.Noun)} in the drawer.", Content = Ui.Row(8, _name, _create) },
                    new FieldRow { Label = "Import", Hint = "Every save on every game's own card copied onto the chosen card, all or none; each game's own card stays as it is.", Content = _import });
                Refresh();
            }

            public Control View { get; }

            private bool _filling;

            private static Button Act(string text, string name, Action act)
            {
                Button button = Ui.Button(text, act);
                button.Name = name;
                return button;
            }

            private string? PathOf(string shown) => _medium.Cards.FirstOrDefault(c => c.Label == shown)?.Path;

            private CardFile? Chosen => _medium.Cards.FirstOrDefault(c => c.Path == _chosenCard);

            // The chooser and the saves read again from the runtime, the actions enabled as the card and its holder allow.
            public void Refresh()
            {
                _medium = CardMedium.Of(_owner._runtime, _medium.Console).FirstOrDefault(m => m.Id == _medium.Id) ?? _medium;
                IReadOnlyList<CardFile> cards = _medium.Cards;
                if (_chosenCard is null || cards.All(c => c.Path != _chosenCard)) _chosenCard = AssignedCard() ?? cards.FirstOrDefault()?.Path;
                _filling = true;
                string[] labels = cards.Select(c => c.Label).ToArray();
                _card.Fill(labels, Chosen?.Label ?? "");
                string[] targets = cards.Where(c => c.Path != _chosenCard).Select(c => c.Label).ToArray();
                _target.Fill(targets, targets.FirstOrDefault() ?? "");
                _filling = false;
                if (string.IsNullOrEmpty(_name.Text) || cards.Any(c => !c.GamesOwn && c.Name == _name.Text)) _name.Text = _medium.NextName;

                _saves.Children.Clear();
                string mode = ModeText();
                JsonDocument? opened = null;
                try
                {
                    if (_chosenCard is not null)
                    {
                        try { opened = JsonDocument.Parse(_owner._runtime.CardsOp(CrystalAbi.CardsOpen, JsonSerializer.Serialize(new { path = _chosenCard }))); }
                        catch (CrystalException) { opened = null; }
                    }
                    if (opened is null)
                    {
                        _about.Text = $"{mode} No card of this medium is in the drawer or a game's yet.";
                    }
                    else if (!_medium.Readable)
                    {
                        _about.Text = $"{mode} {opened.RootElement.GetProperty("length").GetInt64():N0} bytes. No format module reads this medium yet, so its saves are not listed; a new card can still be made.";
                    }
                    else
                    {
                        JsonElement card = opened.RootElement;
                        JsonElement[] saves = card.GetProperty("saves").EnumerateArray().ToArray();
                        _about.Text = $"{mode} " + (card.GetProperty("formatted").GetBoolean()
                            ? $"{saves.Length} save(s), {card.GetProperty("free").GetInt64()} of {_medium.Capacity} {_medium.Unit}s free."
                            : card.GetProperty("problem").GetString());
                        for (int i = 0; i < saves.Length; i++)
                        {
                            string id = saves[i].GetProperty("id").GetString() ?? "";
                            string title = saves[i].GetProperty("title").GetString() ?? "";
                            bool damaged = saves[i].GetProperty("damaged").GetBoolean();
                            Button row = Ui.Button($"{(id == _chosenSave ? "▸ " : "")}{title}  ·  {saves[i].GetProperty("units").GetInt64()} {_medium.Unit}(s){(damaged ? "  ·  damaged" : "")}", () => { _chosenSave = id; Refresh(); });
                            row.Name = $"{_medium.Console}.{_medium.Id}.Save.{i}";
                            row.HorizontalAlignment = HorizontalAlignment.Stretch;
                            row.HorizontalContentAlignment = HorizontalAlignment.Left;
                            _saves.Children.Add(row);
                        }
                        if (saves.Length == 0) _saves.Children.Add(Ui.Hint("No saves."));
                    }
                }
                finally { opened?.Dispose(); }
                if (Chosen?.HeldBy is { } holder)
                    _about.Text += $" In use by {holder}, which is running: its saves can be looked at, and changed when the game is closed.";
                Enable();
            }

            private void Enable()
            {
                bool held = Chosen?.HeldBy is not null;
                bool chosen = _chosenSave is not null && _chosenCard is not null;
                string? target = _target.SelectedItem as string is { Length: > 0 } t ? PathOf(t) : null;
                _copy.IsEnabled = chosen && _medium.Readable && target is not null;
                _move.IsEnabled = chosen && _medium.Readable && target is not null && !held;
                _delete.IsEnabled = chosen && _medium.Readable && !held;
                _import.IsEnabled = _medium.Readable && _chosenCard is not null && !held && Chosen is { GamesOwn: false };
            }

            // The mode, and which card each slot holds under it, as the runtime places them.
            private string ModeText()
            {
                if (_medium.Mode is not ("shared" or "named")) return "Per game: each game keeps its own.";
                string slots = string.Join(", ", _medium.Slots.Select(s => $"{s.Name} in {s.Label}"));
                return $"{(_medium.Mode == "shared" ? "Shared" : "Named")}: {slots}.";
            }

            // The card the first slot holds under a shared or named mode, where it exists.
            private string? AssignedCard() =>
                _medium.Mode is "shared" or "named" && _medium.Slots.FirstOrDefault().Path is { } path && _medium.Cards.Any(c => c.Path == path) ? path : null;

            private void Transfer(bool move)
            {
                if (_target.SelectedItem is not string shown || PathOf(shown) is not { } target || _chosenCard is null || _chosenSave is null) return;
                _owner.Done(CardOutcome.Of(_owner._runtime, move ? CrystalAbi.CardsMove : CrystalAbi.CardsCopy, new { from = _chosenCard, save = _chosenSave, to = target }));
                if (move) _chosenSave = null;
            }

            // A name that is one file in the drawer: no separator, no leading dot, nothing a path would read otherwise.
            private static bool IsName(string name) =>
                name.Trim() == name && name.Length > 0 && !name.StartsWith('.') && name.IndexOfAny(System.IO.Path.GetInvalidFileNameChars()) < 0 && name.IndexOfAny(new[] { '/', '\\' }) < 0;

            private void Create()
            {
                string name = (_name.Text ?? "").Trim();
                if (!IsName(name))
                {
                    _owner.Done(new CardOutcome(false, $"\"{name}\" is not a name a card can have: no slash, no leading dot."));
                    return;
                }
                string path = _medium.PathOf(name);
                CardOutcome made = CardOutcome.Of(_owner._runtime, CrystalAbi.CardsNew, new { path });
                if (made.Done) _chosenCard = path;
                _name.Text = "";
                _owner.Done(made);
            }

            private void Import()
            {
                if (_chosenCard is null) return;
                _owner.Done(CardOutcome.Of(_owner._runtime, CrystalAbi.CardsImport, new { to = _chosenCard, from = _medium.Cards.Where(c => c.GamesOwn).Select(c => c.Path).ToArray() }));
            }
        }
    }
}
