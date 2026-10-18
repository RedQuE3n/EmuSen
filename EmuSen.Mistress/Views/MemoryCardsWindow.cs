using System;
using System.Collections.Generic;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Layout;
using EmuSen.Cores;
using EmuSen.Crystal.SaveMedia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;

namespace EmuSen.Mistress.Views
{
    // Every console's save media: each medium's cards, the saves on one, and copying, moving, deleting and importing them through the card manager - see EmuSen_SaveMedia.md §6.5.
    public sealed class MemoryCardsWindow : ToolWindow
    {
        private readonly GraphicsConfig _config;
        private readonly Tabs _tabs = new() { Name = "MemoryCardTabs" };
        private readonly HintText _status = new() { Name = "MemoryCardsStatus" };
        private readonly List<MediumPane> _panes = new();

        // Parameterless constructor exists only for tooling - real code always uses the one below.
        public MemoryCardsWindow() : this(new GraphicsConfig(), null) { }

        public MemoryCardsWindow(GraphicsConfig config, string? selectedConsole)
        {
            _config = config;
            Title = "Memory Cards";
            Width = 720;
            Height = 600;
            MinWidth = 560;
            MinHeight = 420;
            CanResize = true;

            Consoles = CoreCatalog.ConsolesInReleaseOrder.Select(c => c.Console).Where(c => MemoryCards.MediaFor(c).Count > 0).ToList();
            foreach (string console in Consoles) _tabs.Add(console, new ScrollViewer { Content = BuildConsole(console) });
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

        private Control BuildConsole(string console)
        {
            var panel = new StackPanel { Spacing = 18, Margin = new Avalonia.Thickness(0, 8, 0, 0) };
            foreach (SaveMedium medium in MemoryCards.MediaFor(console))
            {
                var pane = new MediumPane(this, medium);
                _panes.Add(pane);
                panel.Children.Add(pane.View);
            }
            return panel;
        }

        // The pane of a medium; every pane drawn again after an action, as a copy changes the card it lands on.
        internal IReadOnlyList<MediumPane> Panes => _panes;

        private void Done(MediaOutcome outcome)
        {
            _status.Text = outcome.Words;
            foreach (MediumPane pane in _panes) pane.Refresh();
        }

        // One medium: its mode, a chooser of its cards, the saves on the chosen one, and the actions on them.
        internal sealed class MediumPane
        {
            private readonly MemoryCardsWindow _owner;
            private readonly SaveMedium _medium;
            private readonly Dropdown _card;
            private readonly Dropdown _target;
            private readonly HintText _about = new();
            private readonly StackPanel _saves = Ui.Stack(4);
            private readonly TextBox _name = new() { MinWidth = 200, HorizontalAlignment = HorizontalAlignment.Left };
            private readonly Button _copy, _move, _delete, _create, _import;
            private IReadOnlyList<MemoryCards.MediumFile> _cards = Array.Empty<MemoryCards.MediumFile>();
            private string? _chosenCard;
            private string? _chosenSave;

            public MediumPane(MemoryCardsWindow owner, SaveMedium medium)
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
                _delete = Act("Delete Save", name + ".Delete", () => _owner.Done(SaveMediaManager.DeleteSave(_chosenCard!, _chosenSave!)));
                _create = Act("New Card", name + ".New", Create);
                _import = Act("Import Every Game's Saves", name + ".Import", Import);
                View = Ui.Stack(8,
                    Ui.Header(medium.Title),
                    new FieldRow { Label = "Card", Hint = "A card in the drawer, or a game's own.", Content = Ui.Stack(4, _card, _about) },
                    new FieldRow { Label = "Saves", Hint = "Choose a save to copy, move or delete it.", Content = _saves },
                    new FieldRow { Label = "Copy or move to", Content = Ui.Stack(8, _target, Ui.Row(8, _copy, _move, _delete)) },
                    new FieldRow { Label = "New card", Hint = $"A name for a new {MediumWords.InText(medium.Noun)} in the drawer.", Content = Ui.Row(8, _name, _create) },
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

            private string? PathOf(string shown) => _cards.FirstOrDefault(c => c.Label == shown)?.Path;

            // The chooser and the saves read again from the files, the actions enabled as the card and its holder allow.
            public void Refresh()
            {
                _cards = MemoryCards.CardsOf(_medium);
                if (_chosenCard is null || _cards.All(c => c.Path != _chosenCard)) _chosenCard = AssignedCard() ?? _cards.FirstOrDefault()?.Path;
                _filling = true;
                string[] labels = _cards.Select(c => c.Label).ToArray();
                _card.Fill(labels, _cards.FirstOrDefault(c => c.Path == _chosenCard)?.Label ?? "");
                string[] targets = _cards.Where(c => c.Path != _chosenCard).Select(c => c.Label).ToArray();
                _target.Fill(targets, targets.FirstOrDefault() ?? "");
                _filling = false;
                if (string.IsNullOrEmpty(_name.Text) || _cards.Any(c => !c.GamesOwn && c.Name == _name.Text)) _name.Text = MemoryCards.NextName(_medium);

                _saves.Children.Clear();
                MediumCard? card = _chosenCard is null ? null : SaveMediaManager.Open(_chosenCard);
                string mode = ModeText();
                if (card is null)
                {
                    _about.Text = $"{mode} No card of this medium is in the drawer or a game's yet.";
                }
                else if (card.Listing is not { } listing)
                {
                    _about.Text = $"{mode} {card.Length:N0} bytes. No format module reads this medium yet, so its saves are not listed; a new card can still be made.";
                }
                else
                {
                    _about.Text = $"{mode} " + (listing.Formatted ? $"{listing.Saves.Count} save(s), {listing.FreeUnits} of {card.Medium.Format!.Capacity} {card.Medium.Format.Unit}s free." : listing.Problem);
                    foreach (SaveEntry save in listing.Saves)
                    {
                        string id = save.Id;
                        Button row = Ui.Button($"{(id == _chosenSave ? "▸ " : "")}{save.Title}  ·  {save.Units} {card.Medium.Format!.Unit}(s){(save.Damaged ? "  ·  damaged" : "")}", () => { _chosenSave = id; Refresh(); });
                        row.Name = $"{_medium.Console}.{_medium.Id}.Save.{listing.Saves.ToList().IndexOf(save)}";
                        row.HorizontalAlignment = HorizontalAlignment.Stretch;
                        row.HorizontalContentAlignment = HorizontalAlignment.Left;
                        _saves.Children.Add(row);
                    }
                    if (listing.Saves.Count == 0) _saves.Children.Add(Ui.Hint("No saves."));
                }
                if (_chosenCard is not null && SaveMediaManager.HeldBy(_chosenCard) is { } holder)
                    _about.Text += $" In use by {holder}, which is running: its saves can be looked at, and changed when the game is closed.";
                Enable();
            }

            private void Enable()
            {
                bool held = _chosenCard is not null && SaveMediaManager.HeldBy(_chosenCard) is not null;
                bool readable = _medium.Entry?.Format is not null;
                bool chosen = _chosenSave is not null && _chosenCard is not null;
                string? target = _target.SelectedItem as string is { Length: > 0 } t ? PathOf(t) : null;
                _copy.IsEnabled = chosen && readable && target is not null;
                _move.IsEnabled = chosen && readable && target is not null && !held;
                _delete.IsEnabled = chosen && readable && !held;
                _import.IsEnabled = readable && _chosenCard is not null && !held && _cards.FirstOrDefault(c => c.Path == _chosenCard) is { GamesOwn: false };
            }

            // The mode, and which card each slot holds under it, as the console's settings say.
            private string ModeText()
            {
                CardChoice choice = MemoryCards.Read(CoreSettings.StoredFor(_medium.Console, _owner._config), _medium);
                if (!choice.Apart) return "Per game: each game keeps its own.";
                string slots = string.Join(", ", Enumerable.Range(0, _medium.Slots.Count).Select(n => $"{_medium.NameFor(choice, n)} in {_medium.SlotLabel(n)}"));
                return $"{(choice.Mode == MemoryCards.Shared ? "Shared" : "Named")}: {slots}.";
            }

            // The card the first slot holds under a shared or named mode, where it exists.
            private string? AssignedCard()
            {
                CardChoice choice = MemoryCards.Read(CoreSettings.StoredFor(_medium.Console, _owner._config), _medium);
                string path = _medium.PathOf(_medium.NameFor(choice, 0));
                return choice.Apart && _cards.Any(c => c.Path == path) ? path : null;
            }

            private void Transfer(bool move)
            {
                if (_target.SelectedItem is not string shown || PathOf(shown) is not { } target || _chosenCard is null || _chosenSave is null) return;
                _owner.Done(move ? SaveMediaManager.MoveSave(_chosenCard, _chosenSave, target) : SaveMediaManager.CopySave(_chosenCard, _chosenSave, target));
                if (move) _chosenSave = null;
            }

            private void Create()
            {
                string name = (_name.Text ?? "").Trim();
                if (!MemoryCards.IsName(name))
                {
                    _owner.Done(MediaOutcome.No($"\"{name}\" is not a name a card can have: no slash, no leading dot."));
                    return;
                }
                string path = _medium.PathOf(name);
                MediaOutcome made = SaveMediaManager.New(path);
                if (made.Done) _chosenCard = path;
                _name.Text = "";
                _owner.Done(made);
            }

            private void Import()
            {
                if (_chosenCard is null) return;
                _owner.Done(SaveMediaManager.Import(_chosenCard, _cards.Where(c => c.GamesOwn).Select(c => c.Path).ToArray()));
            }
        }
    }
}
