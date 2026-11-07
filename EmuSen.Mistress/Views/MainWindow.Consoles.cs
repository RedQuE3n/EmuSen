using System;
using System.Collections.Generic;
using System.Linq;
using Avalonia;
using Avalonia.Styling;
using EmuSen.Crystal.Client;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Pages;

namespace EmuSen.Mistress.Views
{
    // Choose a console between Home and the grid, and the night sky under the own look's three screens - see EmuSen_Interface.md §24.
    public partial class MainWindow
    {
        public const string ConsolesScreenName = "Consoles";

        private PlanetLayout? _planetLayout;
        private ConsolesScreen? _consoles;

        public PlanetLayout? ConsolesLayout => _planetLayout?.IsVisible == true ? _planetLayout : null;

        public ConsolesScreen? ConsolesShown => ConsolesLayout is null ? null : _consoles;

        // The own look stands on the night sky unless the player chose the light variant, which keeps the day's surfaces.
        private bool Night => ActualThemeVariant != ThemeVariant.Light;

        // A shelf's planet hue: its system's in Moonrise's table.
        private static double? HueOfShelf(string? shelf) => RuntimeCatalog.ShelfByName(shelf) is { } s ? StartupSplash.HueOf(s.System) : null;

        // Every shelf of the systems table, in its order, with the facts Choose a console says.
        private static IReadOnlyList<ConsoleShelf> ShelfTable() => RuntimeCatalog.Shelves.Select(s =>
        {
            SystemFacts? system = RuntimeCatalog.Systems.FirstOrDefault(f => f.Id == s.System);
            string label = RuntimeCatalog.ShelfDisplayName(s.Name);
            return new ConsoleShelf(s.Name, label, system?.Name is { Length: > 0 } name ? name : label, system?.Manufacturer ?? "", system?.ReleaseYear ?? 0, StartupSplash.HueOf(s.System));
        }).ToList();

        // Home's Games place, and B on the grid with the focus on the section left.
        internal void ShowConsoles(string? focus = null)
        {
            _ownScreen = ConsolesScreenName;
            ShowLibraryEntries();
            if (focus is not null) _planetLayout?.FocusOn(focus);
            _planetLayout?.Focus();
        }

        private void FillConsoles()
        {
            _consoles ??= new ConsolesScreen(this);
            if (_planetLayout is null)
            {
                _planetLayout = new PlanetLayout { Name = "ConsolesScreen", Title = ConsolesScreen.Title, Subtitle = ConsolesScreen.Subtitle };
                _planetLayout.ItemChosen += OpenConsole;
                _planetLayout.HintInvoked += ConsolesHint;
                _planetLayout.PictureOf = CoverPicture;
                WireOwnPictures();
                LibraryArea.Children.Add(_planetLayout);
            }
            _planetLayout.PadFamily = HelpFamily == PadFamily.Generic ? PadFamily.Xbox : HelpFamily;
            _planetLayout.Night = Night;
            _planetLayout.IsVisible = true;
            _planetLayout.HintsFor = _ => Hints(ConsolesScreen.Hints);
            _planetLayout.Count = _consoles.CountLine;
            _planetLayout.SetTiles(_consoles.Places.Select(Choice));
            _planetLayout.SetPlanets(_consoles.Consoles.Select(Choice));
        }

        // A console's body, by its shelf's system and maker; null for a wide place.
        private static WorldBody? BodyOfShelf(ConsoleChoice c) =>
            c.Hue is null ? null : RuntimeCatalog.ShelfByName(c.Key) is { } shelf ? StartupSplash.BodyOf(shelf.System) : WorldBody.Plain;

        private PlanetItem Choice(ConsoleChoice c) => new(c.Key, c.Name)
        {
            Hue = c.Hue, Status = c.Status, Title = c.Title, Facts = c.Facts, Icon = ConsolesScreen.Icons.GetValueOrDefault(c.Key), Body = BodyOfShelf(c),
            Engine = c.Engine is { } e ? new PlanetEngine(e.Name, ConsolesScreen.TagOf(e), !e.Libretro) { Mark = e.Mark } : null,
            Covers = c.Covers.Select(g => CoverOf(g.Facts.Path, g.Facts.Title, g.Facts.Mark, g.Facts.CoverAspect, false, "", g.Facts.Console, HueOfShelf(g.Shelf))).ToList(),
        };

        // A on a world: its grid.
        private void OpenConsole(string key) => OpenSection(ConsolesScreen.SectionOf(key));

        private void OpenSection(string section, string? focus = null)
        {
            _games ??= new GamesScreen(this);
            _games.ShowCollection(null);
            _games.ChooseSection(section);
            ShowGames();
            if (focus is not null) _gallery?.FocusOn(focus);
        }

        // Y opens Favourites; X a random game of the focused world, its grid open on it.
        private void ConsolesHint(string key)
        {
            if (_consoles is null || _planetLayout?.Focused is not { } focused) return;
            switch (key)
            {
                case "back": ShowHome(); Cue("back"); break;
                case "favourites": OpenSection(GamesScreen.Favourites); break;
                case "random" when _consoles.RandomGame(focused) is { } path: OpenSection(ConsolesScreen.SectionOf(focused), path); break;
            }
        }

        private void ConsolesPadCommand(PlanetLayout layout, UiButton button)
        {
            switch (button)
            {
                case UiButton.Up: Heard(button, layout.Move(0, -1)); return;
                case UiButton.Down: Heard(button, layout.Move(0, 1)); return;
                case UiButton.Left: Heard(button, layout.Move(-1, 0)); return;
                case UiButton.Right: Heard(button, layout.Move(1, 0)); return;
                case UiButton.Menu: ShowHome(); Cue("back"); return;
            }
            if (ToolPaneWindow.Glyph(button) is { } glyph) Heard(button, layout.Press(glyph));
        }

        IReadOnlyList<ConsoleShelf> IGamesHost.Table => ShelfTable();

        void IGamesHost.ChooseConsole(string section) => ShowConsoles(ConsolesScreen.KeyOf(section));

        EngineShown? IGamesHost.EngineOf(string shelf)
        {
            try { return RuntimeCatalog.ShelfByName(shelf) is { } s ? RuntimeCatalog.EngineOf(s.Core.Console) : null; }
            catch (Exception e) when (e is CrystalException or System.Text.Json.JsonException) { return null; }
        }
    }
}
