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
using Avalonia.Media;
using Avalonia.VisualTree;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The big-screen metadata editor in ES-DE's layout, read off the pixels, and its two new fields - see EmuSen_BigPicture.md §34 and the settings reference §4.72.
    [Collection(TestCollections.ProcessGlobals)]
    public class MetadataEditorLayoutTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(MetadataEditorLayoutTests).GetTypeInfo().Assembly);

        public static TheoryData<int, int> Sizes() => EsdeMenusTests.Sizes();

        // ES-DE's user guide's order, with Mistress's Play time where ES-DE keeps its own, and no Delete.
        private static readonly string[] Labels =
        [
            "Name", "Sort name", "Description", "Rating", "Release date", "Developer", "Publisher", "Genre", "Players", "Favourite", "Completed", "Kid game",
            "Hidden", "Broken / not working", "Exclude from game counter", "Exclude from multi-scraper", "Hide metadata fields", "Times played", "Play time",
            "Controller", "Alternative emulator",
        ];

        private static readonly string[] ButtonTexts = ["Scrape", "Save", "Cancel", "Clear", "Hide from Library..."];

        private static MenuPanel Menu(MetadataEditorWindow editor) => editor.Menu!;

        private static MenuRow RowOf(Control c) => c.GetVisualDescendants().OfType<MenuRow>().First();

        // Each line's label as drawn, top to bottom, the name offer's buttons left out while hidden.
        private static List<string> DrawnLabels(ThemedSession s)
        {
            StackPanel rows = ThemedCollectionsTests.Named<StackPanel>(s, "MetadataRows");
            var labels = new List<string>();
            foreach (Control line in rows.Children.Where(c => c.IsVisible))
            {
                Control row = line is Grid g ? g.Children[0] : line;
                labels.Add(row is MenuFieldRow field ? field.Label! : MenuRows.GetLabel(row)!);
            }
            return labels;
        }

        private static int CountNear(RenderedFrame f, Rect box, Color colour, int tolerance = 14)
        {
            int n = 0;
            for (int y = Math.Max(0, (int)box.Top); y < Math.Min(f.Height, (int)box.Bottom); y++)
                for (int x = Math.Max(0, (int)box.Left); x < Math.Min(f.Width, (int)box.Right); x++)
                {
                    (byte r, byte g, byte b) = EsdeMenusTests.At(f, x, y);
                    if (Math.Abs(r - colour.R) <= tolerance && Math.Abs(g - colour.G) <= tolerance && Math.Abs(b - colour.B) <= tolerance) n++;
                }
            return n;
        }

        private static Rect ValueOf(Control editor, Window w)
        {
            MenuRow row = RowOf(editor);
            return EsdeMenusTests.InWindow(row, row.Layout(row.Bounds.Size).Value, w);
        }

        [Theory]
        [MemberData(nameof(Sizes))]
        public Task The_editor_is_a_centred_panel_titled_in_capitals_over_the_game_and_its_file_with_a_row_per_field_and_its_buttons_in_one_row(int width, int height) => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(width, height);
            ThemedLibraryPadTests.Enter(s, "snes");
            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
            RenderedFrame open = s.Capture();

            Assert.True(SheetLayer.GetChromeless(editor));
            MenuPanel menu = Menu(editor);
            Assert.Equal("Edit Metadata", menu.Title);
            Assert.Equal(EmuSen.LunaP.Media.LetterCase.Upper, menu.LetterCase);
            Assert.Equal(ThemedSession.SnesGames[0] + "\n" + ThemedSession.SnesGames[0] + ".sfc", menu.Subtitle);
            Rect panel = EsdeMenusTests.InWindow(menu, menu.PanelBounds, s.Window);
            EsdeMenusTests.AssertCentred(panel, s.Window.GetControl<Control>("ScreenContent").Bounds.Size);

            // One row per field in ES-DE's order; outside a custom collection there is no custom collections sortname.
            Assert.Equal(Labels, DrawnLabels(s));
            Assert.Equal(MenuRowKind.Submenu, MenuRows.GetKind(editor.EditorOf(GameMetadata.Name)));
            Assert.Equal(MenuRowKind.Switch, MenuRows.GetKind(editor.EditorOf(GameMetadata.HideMetadata)));
            Assert.Equal(MenuRowKind.Option, MenuRows.GetKind(editor.EditorOf(GameMetadata.Controller)));
            Assert.IsType<RatingPicker>(editor.EditorOf(GameMetadata.Rating));
            Assert.IsType<MenuFieldRow>(editor.EditorOf(GameMetadata.Rating).Parent);

            // ES-DE's editor's proportions, measured on its own editor at 1280 by 800 (Q104): 42-pixel rows, "unknown" where it shows it.
            double u = height / 800.0;
            Assert.Equal(42 * u, RowOf(editor.EditorOf(GameMetadata.SortName)).Bounds.Height, 1.0);
            Assert.Equal(42 * u, RowOf(editor.EditorOf(GameMetadata.Completed)).Bounds.Height, 1.0);
            Assert.Equal("unknown", RowOf(editor.EditorOf(GameMetadata.Developer)).Value);
            Assert.True(string.IsNullOrEmpty(RowOf(editor.EditorOf(GameMetadata.SortName)).Value));
            Assert.Equal("unknown", ((DateStepper)editor.EditorOf(GameMetadata.ReleaseDate)).NoDateText);
            Assert.Equal(5 * MetadataEditorWindow.StarSize * u, editor.EditorOf(GameMetadata.Rating).Bounds.Width, 2.0);

            // The first row has the focus and the bar across the panel; the footer says where its value comes from.
            Assert.True(editor.EditorOf(GameMetadata.Name).IsFocused);
            MenuRow name = RowOf(editor.EditorOf(GameMetadata.Name));
            Assert.True(name.IsHighlighted);
            EsdeMenusTests.AssertBarSpans(open, EsdeMenusTests.InWindow(name, name.Layout(name.Bounds.Size).Bar, s.Window), panel, name.BarColor);
            Assert.Equal("Name: From the file name.", menu.Footer);

            // The buttons in one row under the rows, in ES-DE's order with Hide from Library where its Delete was, centred on the panel.
            StackPanel bar = ThemedCollectionsTests.Named<StackPanel>(s, "MetadataButtons");
            Button[] buttons = bar.Children.OfType<Button>().ToArray();
            Assert.Equal(ButtonTexts, buttons.Select(b => (string)b.Content!));
            Assert.Single(buttons.Select(b => b.Bounds.Top).Distinct());
            Rect band = EsdeMenusTests.InWindow(menu, menu.ButtonsBounds, s.Window);
            Assert.All(buttons, b => Assert.True(band.Contains(EsdeMenusTests.InWindow(b, new Rect(b.Bounds.Size), s.Window)), (string)b.Content!));
            Assert.Equal(panel.Center.X, EsdeMenusTests.InWindow(bar, new Rect(bar.Bounds.Size), s.Window).Center.X, 1.0);
            Rect rowsBox = EsdeMenusTests.InWindow(menu.Child!, new Rect(menu.Child!.Bounds.Size), s.Window);
            Assert.True(band.Top >= rowsBox.Bottom - 0.5, $"the buttons {band} overlap the rows {rowsBox}");
            Assert.DoesNotContain(ThemedGameOptionsTests.Sheet(s).GetVisualDescendants().OfType<Button>(), b => b.Content is string t && t.StartsWith("Delete", StringComparison.Ordinal));

            // A button describes no field, so the footer is empty while one has the focus.
            ThemedGameOptionsTests.Reach(s, "MetadataSave");
            Assert.Null(menu.Footer);

            // Nothing is drawn outside the panel and its help bar.
            RenderedFrame withMenu = s.Capture();
            Rect help = EsdeMenusTests.InWindow(menu.HelpBar, new Rect(menu.HelpBar.Bounds.Size), s.Window);
            menu.IsVisible = false;
            RenderedFrame without = s.Capture();
            menu.IsVisible = true;
            (int inside, int outside) = EsdeMenusTests.Changed(withMenu, without, panel.Inflate(1), help.Inflate(1));
            Assert.True(inside > 1000, $"only {inside} pixels of the panel were drawn");
            Assert.Equal(0, outside);
        }, default);

        // ES-DE's colours: a value is grey as opened and blue once changed, stars as well as text; Reset shows only beside a changed field.
        [Fact]
        public Task A_value_is_drawn_grey_as_opened_and_blue_once_changed() => ThemedLibraryPadTests.Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
            Control nameBox = editor.EditorOf(GameMetadata.Name);
            var stars = (RatingPicker)editor.EditorOf(GameMetadata.Rating);
            RenderedFrame before = s.Capture();
            Rect starsBox = EsdeMenusTests.InWindow(stars, new Rect(stars.Bounds.Size), s.Window);
            Assert.True(CountNear(before, ValueOf(nameBox, s.Window), MetadataEditorWindow.EditedColor) < 5);
            Assert.True(CountNear(before, starsBox, MetadataEditorWindow.EditedColor) < 5);
            Assert.False(editor.CanReset(GameMetadata.Name));

            // Blue on the focused row as well as off it: the colour says where the value came from, the bar says where the focus is.
            ThemedGameOptionsTests.Type(s, "Meta_" + GameMetadata.Name, "zeta");
            RenderedFrame named = s.Capture();
            Assert.True(CountNear(named, ValueOf(nameBox, s.Window), MetadataEditorWindow.EditedColor) > 40, "the changed name is not blue");
            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.Rating);
            s.Pad.Right(4);
            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.Description);
            RenderedFrame after = s.Capture();
            starsBox = EsdeMenusTests.InWindow(stars, new Rect(stars.Bounds.Size), s.Window);
            Assert.True(CountNear(after, starsBox, MetadataEditorWindow.EditedColor) > 40, "the changed stars are not blue");
            Assert.True(editor.CanReset(GameMetadata.Name));
            Assert.True(editor.CanReset(GameMetadata.Rating));
            Assert.False(editor.CanReset(GameMetadata.Developer));
            Assert.True(CountNear(after, ValueOf(editor.EditorOf(GameMetadata.Description), s.Window), MetadataEditorWindow.EditedColor) < 5);
        });

        // ES-DE's Hide metadata fields: stored as an edit, and the gamelist then hides the game's metadata but keeps its name and description.
        [Fact]
        public Task Hide_metadata_fields_is_stored_as_an_edit_and_hides_the_game_s_metadata_in_the_gamelist() => ThemedLibraryPadTests.Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            string game = ThemedSession.SnesGames[0];
            SceneBuilder scene = s.Themed.Stage!.Current.Scene;
            Assert.Null(scene.Find("rating", "r")!.Skipped);
            Assert.NotNull(scene.Find("text", "dev")!.Control);

            ThemedGameOptionsTests.OpenEditor(s);
            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.HideMetadata);
            s.Pad.A();
            ThemedGameOptionsTests.Reach(s, "MetadataSave");
            s.Pad.A();
            s.Settle();
            Assert.Equal(GameMetadata.Yes, ThemedGameOptionsTests.StoredEdits(s, game + ".sfc")[GameMetadata.HideMetadata]);
            Assert.True(s.Themed.SelectedGame!.HideMetadata);
            scene = s.Themed.Stage!.Current.Scene;
            Assert.Equal("the entry hides its metadata", scene.Find("rating", "r")!.Skipped);
            Assert.Equal("the entry hides its metadata", scene.Find("text", "dev")!.Skipped);
            Assert.NotEqual("the entry hides its metadata", scene.Find("text", "desc")!.Skipped);

            // The next game keeps its metadata.
            s.Pad.Down();
            scene = s.Themed.Stage!.Current.Scene;
            Assert.False(s.Themed.SelectedGame!.HideMetadata);
            Assert.Null(scene.Find("rating", "r")!.Skipped);

            // Turned off again, the edit is gone and the metadata is back.
            s.Pad.Up();
            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
            Assert.True(((LunaSwitch)editor.EditorOf(GameMetadata.HideMetadata)).IsChecked);
            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.HideMetadata);
            s.Pad.A();
            ThemedGameOptionsTests.Reach(s, "MetadataSave");
            s.Pad.A();
            s.Settle();
            Assert.False(ThemedGameOptionsTests.StoredEdits(s, game + ".sfc").ContainsKey(GameMetadata.HideMetadata));
            Assert.Null(s.Themed.Stage!.Current.Scene.Find("rating", "r")!.Skipped);
        }, extraGamelist: ThemedCollectionsTests.MetadataElements);

        // ES-DE's custom collections sortname: offered only from inside a custom collection, and it orders custom collections and nothing else.
        [Fact]
        public Task The_custom_collections_sortname_is_offered_only_inside_a_custom_collection_and_orders_only_custom_collections() => ThemedLibraryPadTests.Run(s =>
        {
            string first = ThemedSession.SnesGames[0];
            ThemedLibraryPadTests.Enter(s, "snes");
            MetadataEditorWindow outside = ThemedGameOptionsTests.OpenEditor(s);
            Assert.False(outside.InCustomCollection);
            Assert.DoesNotContain(outside.Shown, f => f.Key == GameMetadata.CustomSortName);
            Assert.DoesNotContain(ThemedGameOptionsTests.Sheet(s).GetVisualDescendants().OfType<Control>(), c => c.Name == "Meta_" + GameMetadata.CustomSortName);
            ThemedGameOptionsTests.Reach(s, "MetadataCancel");
            s.Pad.A();
            s.Settle();
            s.Pad.B();

            GameRecords records = ThemedCollectionsTests.Records(s);
            long id = records.CreateCollection("Platform", DateTime.Now)!.Value;
            foreach (string game in ThemedSession.SnesGames) records.AddToCollection(id, ThemedCollectionsTests.Rom(s, game));
            ThemedCollectionsTests.Refresh(s);
            ThemedCollectionsTests.Enter(s, "collections");
            s.Pad.A();
            s.Settle();
            Assert.Equal(first, s.Game);
            MetadataEditorWindow inside = ThemedGameOptionsTests.OpenEditor(s);
            Assert.True(inside.InCustomCollection);
            Assert.Equal(["Name", "Sort name", "Custom collections sortname", "Description"], DrawnLabels(s).Take(4));
            ThemedGameOptionsTests.Type(s, "Meta_" + GameMetadata.CustomSortName, "zz");
            ThemedGameOptionsTests.Reach(s, "MetadataSave");
            s.Pad.A();
            s.Settle();
            Assert.Equal("zz", ThemedGameOptionsTests.StoredEdits(s, first + ".sfc")[GameMetadata.CustomSortName]);

            // Last in the collection, still first in the console's own list, and shown by its name in both.
            Assert.Equal(ThemedSession.SnesGames.Skip(1).Append(first), ThemedCollectionsTests.Listed(s));
            s.Pad.B();
            s.Pad.B();
            ThemedLibraryPadTests.Enter(s, "snes");
            Assert.Equal(ThemedSession.SnesGames, ThemedCollectionsTests.Listed(s));
        });

        // Hide from Library asks in a message box over the editor: the editor stays drawn beneath, its help bar put away, and Cancel returns to it.
        [Fact]
        public Task Hide_from_library_asks_in_a_message_box_over_the_editor_which_stays_drawn_beneath() => ThemedLibraryPadTests.Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
            Control editorSheet = ThemedGameOptionsTests.Sheet(s);
            ThemedGameOptionsTests.Reach(s, "MetadataHide");
            s.Pad.A();
            s.Settle();
            Window box = ThemedGameOptionsTests.Sheets(s).Current!;
            Assert.NotSame(editor, box);
            Assert.True(SheetLayer.GetChromeless(box));
            Assert.True(editorSheet.IsVisible);
            Assert.False(editorSheet.IsHitTestVisible);
            Assert.Equal(0, Menu(editor).HelpBar.Opacity);
            Assert.Contains("never deletes", box.Title);

            ThemedGameOptionsTests.Answer(s, "Cancel");
            Assert.Same(editor, ThemedGameOptionsTests.Sheets(s).Current);
            Assert.True(editorSheet.IsHitTestVisible);
            Assert.Equal(1, Menu(editor).HelpBar.Opacity);
        });

        // ES-DE's keys drive a big-screen menu as the pad does, with no pad (Q102): arrows move, Enter chooses, Delete resets, Backspace leaves.
        [Fact]
        public Task ES_DE_s_keys_move_choose_reset_and_leave_in_the_editor_with_no_pad() => ThemedLibraryPadTests.Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
            s.Pad.Unplug();
            s.Pad.Tick();
            Assert.True(editor.EditorOf(GameMetadata.Name).IsFocused);

            // Down past a text row, which it does not type into, to Sort name.
            EsdeMenusTests.Press(s, Key.Down);
            Assert.True(editor.EditorOf(GameMetadata.SortName).IsFocused);
            Assert.Equal("", ((TextBox)editor.EditorOf(GameMetadata.SortName)).Text);
            var kid = (LunaSwitch)editor.EditorOf(GameMetadata.KidGame);
            for (int guard = 0; guard < 20 && !kid.IsFocused; guard++) EsdeMenusTests.Press(s, Key.Down);
            Assert.True(kid.IsFocused);
            EsdeMenusTests.Press(s, Key.Enter);
            Assert.True(kid.IsChecked);
            Assert.True(editor.CanReset(GameMetadata.KidGame));
            EsdeMenusTests.Press(s, Key.Delete);
            Assert.False(kid.IsChecked);
            Assert.False(editor.CanReset(GameMetadata.KidGame));

            // Backspace on an edited game asks, as B does; Enter takes the focused answer, Save.
            EsdeMenusTests.Press(s, Key.Enter);
            EsdeMenusTests.Press(s, Key.Back);
            s.Settle();
            Assert.Equal("DialogWindow", ThemedGameOptionsTests.Sheets(s).Current!.GetType().Name);
            EsdeMenusTests.Press(s, Key.Enter);
            s.Settle();
            Assert.False(ThemedGameOptionsTests.Sheets(s).IsPresenting);
            Assert.Equal(GameMetadata.Yes, ThemedGameOptionsTests.StoredEdits(s, ThemedSession.SnesGames[0] + ".sfc")[GameMetadata.KidGame]);
        });

        // The same keys in stage 1's Gamelist Options, which Q102 also covers: Down moves off Jump To, Backspace applies and closes.
        [Fact]
        public Task ES_DE_s_keys_move_and_leave_in_the_gamelist_options_with_no_pad() => ThemedLibraryPadTests.Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            ThemedGameOptionsTests.OpenOptions(s);
            s.Pad.Unplug();
            s.Pad.Tick();
            s.Settle();
            Assert.True(ThemedCollectionsTests.Named<Dropdown>(s, "GamelistJumpTo").IsFocused);
            EsdeMenusTests.Press(s, Key.Down);
            Assert.False(ThemedCollectionsTests.Named<Dropdown>(s, "GamelistJumpTo").IsFocused);
            EsdeMenusTests.Press(s, Key.Back);
            s.Settle();
            Assert.False(ThemedGameOptionsTests.Sheets(s).IsPresenting);
        });

        // Q101: no Reset button in a row; West resets the focused field, and the help bar names it only while that field holds an edit.
        [Fact]
        public Task West_resets_the_focused_field_and_the_help_bar_offers_it_only_on_an_edited_field() => ThemedLibraryPadTests.Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
            MenuPanel menu = Menu(editor);
            Assert.DoesNotContain(ThemedGameOptionsTests.Sheet(s).GetVisualDescendants().OfType<Button>(), b => b.Name?.StartsWith("MetaReset_", StringComparison.Ordinal) == true);
            Assert.DoesNotContain(menu.Hints!, h => h.Label == "Reset");

            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.Completed);
            s.Pad.A();
            Assert.Contains(menu.Hints!, h => h.Label == "Reset" && h.Button == PadGlyphButton.West);
            // The edited row's bar now spans the panel, as every other row's does.
            RenderedFrame f = s.Capture();
            MenuRow row = RowOf(editor.EditorOf(GameMetadata.Completed));
            EsdeMenusTests.AssertBarSpans(f, EsdeMenusTests.InWindow(row, row.Layout(row.Bounds.Size).Bar, s.Window), EsdeMenusTests.InWindow(menu, menu.PanelBounds, s.Window), row.BarColor);

            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.KidGame);
            Assert.DoesNotContain(menu.Hints!, h => h.Label == "Reset");
            s.Pad.X();
            Assert.False(((LunaSwitch)editor.EditorOf(GameMetadata.KidGame)).IsChecked);
            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.Completed);
            s.Pad.X();
            Assert.False(((LunaSwitch)editor.EditorOf(GameMetadata.Completed)).IsChecked);
            Assert.False(editor.CanReset(GameMetadata.Completed));
            Assert.DoesNotContain(menu.Hints!, h => h.Label == "Reset");
            Assert.True(editor.EditorOf(GameMetadata.Completed).IsFocused);
        });

        // A long controller name has a short form in a big-screen row; the desktop keeps the full one.
        [Fact]
        public Task A_controller_is_shown_by_its_short_name_in_a_big_screen_row() => ThemedLibraryPadTests.Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
            Assert.Equal(["None", "NES", "SNES", "N64", "Gamepad", "Unknown"], editor.ChoicesOf(GameMetadata.Controller).Select(c => c.Text));
            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.Controller);
            s.Pad.Right(2);
            Assert.Equal("SNES", RowOf(editor.EditorOf(GameMetadata.Controller)).Value);
            Assert.Equal("gamepad_nintendo_snes", editor.Draft.Value(GameMetadata.Controller));
            var desktop = new MetadataEditorWindow(null!, editor.GamePath, "x");
            Assert.Equal("Super Nintendo", desktop.ChoicesOf(GameMetadata.Controller)[2].Text);
        });

        // The question no longer names another program.
        [Fact]
        public Task Hide_from_library_s_question_names_no_other_program() => ThemedLibraryPadTests.Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            ThemedGameOptionsTests.OpenEditor(s);
            ThemedGameOptionsTests.Reach(s, "MetadataHide");
            s.Pad.A();
            s.Settle();
            string question = ThemedGameOptionsTests.Sheets(s).Current!.Title!;
            Assert.StartsWith("EmuSen never deletes or moves a game's file. Hide this game from the library instead?", question);
            Assert.DoesNotContain("ES-DE", question);
            ThemedGameOptionsTests.Answer(s, "Cancel");
        });
    }
}
