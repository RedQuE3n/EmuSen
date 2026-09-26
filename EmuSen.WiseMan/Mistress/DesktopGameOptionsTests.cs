using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.VisualTree;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.Mistress.Views.Covers;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Mistress.BigPicture;

namespace EmuSen.WiseMan.Mistress
{
    // Q19: the game options menu and the metadata editor from Mistress's sidebar library, on the desktop and in its big screen - see EmuSen_Settings_Reference.md §4.62.
    [Collection(TestCollections.ProcessGlobals)]
    public class DesktopGameOptionsTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(DesktopGameOptionsTests).GetTypeInfo().Assembly);

        private static readonly string[] GameEntries = ["Add to Favourites", "Edit This Game's Metadata", "Scrape This Game..."];

        internal static ThemedSession Desktop(string view = AppSettings.LibraryList) =>
            new(settings: a => { a.BigScreen = false; a.LibraryView = view; });

        private static ThemedSession BuiltInBigScreen() => new(settings: a => a.LibraryStyle = AppSettings.LibraryStyleMistress);

        internal static LunaList<RomEntry> List(MainWindow w) => (LunaList<RomEntry>)w.GetControl<ListBox>("LibraryList");

        private static TileGrid<RomEntry> Grid(MainWindow w) => w.GetControl<TileGrid<RomEntry>>("LibraryGrid");

        private static SheetLayer Sheets(MainWindow w) => w.GetControl<SheetLayer>("Sheets");

        internal static string SnesPath(ThemedSession s, int n) => Path.Combine(s.RomDirectory, ThemedSession.SnesGames[n] + ".sfc");

        private static RomEntry Entry(ThemedSession s, int n) => List(s.Window).Models.Single(e => e.FullPath == SnesPath(s, n));

        // The list's names without the console the mixed list adds after a dash.
        internal static IEnumerable<string> ListLabels(MainWindow w) => List(w).Models.Select(List(w).Label).Select(l => l.Split("   —   ")[0]);

        internal static T Named<T>(Visual root, string name) where T : Control => root.GetVisualDescendants().OfType<T>().Single(c => c.Name == name);

        internal static void Click(ThemedSession s, Button button)
        {
            Assert.True(button.IsEffectivelyEnabled && button.IsEffectivelyVisible, button.Name);
            button.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
            s.Settle();
        }

        // The context menu's entry clicked, as a right-click then a left-click on it would; returns every label the menu showed.
        internal static IReadOnlyList<string> ChooseFromContextMenu(ThemedSession s, Control owner, string label)
        {
            ContextMenu menu = owner.ContextMenu!;
            Assert.True(menu.IsOpen);
            ActionMenuItem[] items = menu.ItemsSource!.OfType<ActionMenuItem>().ToArray();
            string[] labels = items.Select(i => i.Action.Text).ToArray();
            ActionMenuItem item = items.Single(i => i.Action.Text == label);
            Assert.True(item.IsEnabled, label);
            menu.Close();
            item.RaiseEvent(new RoutedEventArgs(MenuItem.ClickEvent));
            s.Settle();
            return labels;
        }

        // A right-click on the game's tile, which selects it and opens the grid's context menu.
        private static void RightClickTile(ThemedSession s, int n)
        {
            string title = ArtworkIndex.Untagged(ThemedSession.SnesGames[n]);
            RightClick(s, s.Window.GetVisualDescendants().OfType<CoverTile>().Single(t => t.IsEffectivelyVisible && t.Title == title));
        }

        // A right-click on the game's row in the list.
        private static void RightClickRow(ThemedSession s, int n)
        {
            LunaList<RomEntry> list = List(s.Window);
            RightClick(s, list.ContainerFromIndex(list.Models.ToList().IndexOf(Entry(s, n)))!);
        }

        private static void RightClick(ThemedSession s, Control target)
        {
            Point centre = target.TranslatePoint(new Point(target.Bounds.Width / 2, target.Bounds.Height / 2), s.Window)!.Value;
            s.Window.MouseDown(centre, MouseButton.Right);
            s.Window.MouseUp(centre, MouseButton.Right);
            s.Settle();
        }

        private static GameOptionsWindow OwnedOptions(ThemedSession s)
        {
            GameOptionsWindow options = Assert.IsType<GameOptionsWindow>(s.Window.GameOptionsShown);
            Assert.False(Sheets(s.Window).IsPresenting);
            Assert.Contains(options, s.Window.OwnedWindows);
            return options;
        }

        private static MetadataEditorWindow OwnedEditor(ThemedSession s)
        {
            MetadataEditorWindow editor = Assert.IsType<MetadataEditorWindow>(s.Window.MetadataEditorShown);
            Assert.False(Sheets(s.Window).IsPresenting);
            Assert.Contains(editor, s.Window.OwnedWindows);
            return editor;
        }

        private static void SaveName(ThemedSession s, MetadataEditorWindow editor, string name)
        {
            // A box raises TextChanged once the dispatcher runs, as typing would.
            ((TextBox)editor.EditorOf(GameMetadata.Name)).Text = name;
            s.Settle();
            Click(s, Named<Button>(editor, "MetadataSave"));
            Assert.Null(s.Window.MetadataEditorShown);
        }

        [Fact]
        public Task Right_clicking_a_cover_opens_the_game_options_and_the_editor_as_windows() => Session.Dispatch(() =>
        {
            using ThemedSession s = Desktop(AppSettings.LibraryGrid);
            Assert.True(Grid(s.Window).IsVisible);
            RightClickTile(s, 2);
            Assert.Same(Entry(s, 2), List(s.Window).Selected);
            Assert.True(Grid(s.Window).ContextMenu!.IsOpen);

            IReadOnlyList<string> labels = ChooseFromContextMenu(s, Grid(s.Window), "Game _Options...");
            Assert.Contains("_Edit Metadata...", labels);
            GameOptionsWindow options = OwnedOptions(s);
            Assert.Equal(ThemedSession.SnesGames[2], Named<TextBlock>(options, "GameOptionsTitle").Text);
            // The themed gamelist's jump, sort, filter and search rows are big picture's; the desktop has its own (§4.62).
            Assert.Equal(GameEntries, options.Entries.Select(b => b.Content as string));
            Assert.DoesNotContain(options.Options, o => o.Row is not null || o.Apply is not null);

            Click(s, options.Entries.Single(b => (string?)b.Content == "Edit This Game's Metadata"));
            Assert.Null(s.Window.GameOptionsShown);
            MetadataEditorWindow editor = OwnedEditor(s);
            Assert.Equal(SnesPath(s, 2), editor.GamePath);
            SaveName(s, editor, "Grid Name");
            Assert.Equal("Grid Name", Grid(s.Window).Label(Entry(s, 2)));
            Assert.Contains("Grid Name", ListLabels(s.Window));
        }, default);

        // One edit, made on the desktop, is the name big picture shows; one made in big picture is the desktop's.
        [Fact]
        public Task An_edit_from_the_list_s_context_menu_shows_in_the_list_the_grid_and_big_picture_and_back() => Session.Dispatch(() =>
        {
            using ThemedSession s = Desktop();
            RightClickRow(s, 1);
            Assert.Same(Entry(s, 1), List(s.Window).Selected);
            IReadOnlyList<string> labels = ChooseFromContextMenu(s, List(s.Window), "_Edit Metadata...");
            Assert.Contains("Game _Options...", labels);
            MetadataEditorWindow editor = OwnedEditor(s);
            Assert.Equal(SnesPath(s, 1), editor.GamePath);
            ((TextBox)editor.EditorOf(GameMetadata.Description)).Text = "Written on the desktop.";
            s.Settle();
            SaveName(s, editor, "Desk Name");

            Assert.Contains("Desk Name", ListLabels(s.Window));
            Assert.DoesNotContain(ThemedSession.SnesGames[1], ListLabels(s.Window));
            Assert.Equal("Desk Name", Grid(s.Window).Label(Entry(s, 1)));

            BigPictureSwitchTests.Choose(s, "_Big Picture");
            Assert.True(s.Shown);
            ThemedLibraryPadTests.Enter(s, "snes");
            Assert.Contains("Desk Name", ThemedGameOptionsTests.ListedTitles(s));
            var shown = s.Themed.Stage!.Current.Data.System.Games.Single(g => g.File == SnesPath(s, 1));
            Assert.Equal(("Desk Name", "Written on the desktop."), (shown.Name, shown.Description));

            // And back: an edit in big picture's editor is the desktop's name.
            s.Pad.Down(3);
            ThemedGameOptionsTests.OpenEditor(s);
            ThemedGameOptionsTests.Type(s, "Meta_" + GameMetadata.Name, "couch");
            ThemedGameOptionsTests.Reach(s, "MetadataSave");
            s.Pad.A();
            s.Settle();
            BigPictureSwitchTests.Press(s, Key.F10);
            Assert.False(s.Shown);
            Assert.Contains("couch", ListLabels(s.Window));
            Assert.Contains("Desk Name", ListLabels(s.Window));
        }, default);

        [Fact]
        public Task Ctrl_I_opens_the_editor_for_the_selected_game_but_not_while_typing_in_the_search() => Session.Dispatch(() =>
        {
            using ThemedSession s = Desktop();
            List(s.Window).Select(Entry(s, 3));
            List(s.Window).Focus();
            s.Settle();
            s.Window.KeyPress(Key.I, RawInputModifiers.Control, PhysicalKey.I, null);
            s.Window.KeyRelease(Key.I, RawInputModifiers.Control, PhysicalKey.I, null);
            s.Settle();
            MetadataEditorWindow editor = OwnedEditor(s);
            Assert.Equal(SnesPath(s, 3), editor.GamePath);
            Click(s, Named<Button>(editor, "MetadataCancel"));
            Assert.Null(s.Window.MetadataEditorShown);

            // I alone is not the gesture, and in the search box Ctrl+I is the box's own.
            s.Window.KeyPress(Key.I, RawInputModifiers.None, PhysicalKey.I, null);
            s.Window.KeyRelease(Key.I, RawInputModifiers.None, PhysicalKey.I, null);
            s.Settle();
            Assert.Null(s.Window.MetadataEditorShown);
            TextBox search = s.Window.GetVisualDescendants().OfType<TextBox>().First(b => b.IsEffectivelyVisible && b.FindAncestorOfType<FilterBar>() is not null);
            search.Focus();
            s.Settle();
            s.Window.KeyPress(Key.I, RawInputModifiers.Control, PhysicalKey.I, null);
            s.Window.KeyRelease(Key.I, RawInputModifiers.Control, PhysicalKey.I, null);
            s.Settle();
            Assert.Null(s.Window.MetadataEditorShown);
        }, default);

        // The built-in look in a big-screen session: Start's menu has Game Options..., shown as sheets, and the editor typed by the pad.
        [Fact]
        public Task The_built_in_big_screen_library_s_pad_menu_opens_the_options_and_the_editor_as_sheets() => Session.Dispatch(() =>
        {
            using ThemedSession s = BuiltInBigScreen();
            Assert.False(s.Shown);
            Assert.True(Sheets(s.Window).PresentsWindows);
            List(s.Window).Select(Entry(s, 4));
            s.Settle();

            BigPictureSwitchTests.ChooseFromPadMenu(s, "Game Options...");
            GameOptionsWindow options = Assert.IsType<GameOptionsWindow>(Sheets(s.Window).Current);
            Assert.Empty(s.Window.OwnedWindows);
            Assert.Equal(GameEntries, options.Entries.Select(b => b.Content as string));
            Control sheet = Sheets(s.Window).SheetOf(options)!;
            PadAudit.Reach(sheet, s.Pad, e => e is Button { Content: "Edit This Game's Metadata" });
            s.Pad.A();
            s.Settle();

            MetadataEditorWindow editor = Assert.IsType<MetadataEditorWindow>(Sheets(s.Window).Current);
            Assert.Empty(s.Window.OwnedWindows);
            Assert.Equal(SnesPath(s, 4), editor.GamePath);
            sheet = Sheets(s.Window).SheetOf(editor)!;
            PadAudit.Reach(sheet, s.Pad, e => e is Control { Name: "Meta_name" });
            s.Pad.A();
            OnScreenKeyboard keyboard = OnScreenKeyboard.OpenOver(s.Window)!;
            Assert.NotNull(keyboard);
            for (int n = ThemedSession.SnesGames[4].Length; n > 0; n--) s.Pad.B();
            PadCheatsTests.TypeByPad(s.Pad, keyboard, "pad name");
            s.Pad.Start();
            PadAudit.Reach(sheet, s.Pad, e => e is Control { Name: "MetadataSave" });
            s.Pad.A();
            s.Settle();
            Assert.False(Sheets(s.Window).IsPresenting);
            Assert.Contains("pad name", ListLabels(s.Window));
        }, default);

        // Start's menu over the desktop's list offers the same entry, shown as a window there; the themed gamelist keeps it on Select instead.
        [Fact]
        public Task The_desktop_pad_menu_offers_game_options_and_the_themed_view_s_does_not() => Session.Dispatch(() =>
        {
            using (ThemedSession s = Desktop())
            {
                List(s.Window).Select(Entry(s, 0));
                s.Settle();
                BigPictureSwitchTests.ChooseFromPadMenu(s, "Game Options...");
                GameOptionsWindow options = OwnedOptions(s);
                Assert.Equal(ThemedSession.SnesGames[0], Named<TextBlock>(options, "GameOptionsTitle").Text);
                options.Close();
                s.Settle();
            }

            using (ThemedSession s = new())
            {
                ThemedLibraryPadTests.Enter(s, "snes");
                s.Pad.Start();
                string[] lines = s.Window.GetControl<ListBox>("PadMenuList").ItemsSource!.Cast<object>().Select(o => o.ToString()!).ToArray();
                Assert.DoesNotContain("Game Options...", lines);
                Assert.Contains("Scrape This Game...", lines);
            }
        }, default);
    }
}
