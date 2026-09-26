using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Security.Cryptography;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.VisualTree;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The themed gamelist's game options and metadata editor, driven by the pad alone - see EmuSen_Settings_Reference.md §4.59 and EmuSen_BigPicture.md §23.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemedGameOptionsTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ThemedGameOptionsTests).GetTypeInfo().Assembly);

        private readonly ITestOutputHelper _out;

        public ThemedGameOptionsTests(ITestOutputHelper output) => _out = output;

        internal static SheetLayer Sheets(ThemedSession s) => s.Window.GetControl<SheetLayer>("Sheets");

        internal static Control Sheet(ThemedSession s) => Sheets(s).SheetOf(Sheets(s).Current!)!;

        internal static GameOptionsWindow OpenOptions(ThemedSession s)
        {
            s.Pad.Select();
            return Assert.IsType<GameOptionsWindow>(Sheets(s).Current);
        }

        // Select, then the pad walked to the entry and A pressed on it.
        internal static void Choose(ThemedSession s, string label)
        {
            OpenOptions(s);
            PadAudit.Reach(Sheet(s), s.Pad, e => e is Button { Content: string text } && text == label);
            s.Pad.A();
            s.Settle();
        }

        internal static MetadataEditorWindow OpenEditor(ThemedSession s)
        {
            Choose(s, "Edit This Game's Metadata");
            return Assert.IsType<MetadataEditorWindow>(Sheets(s).Current);
        }

        internal static void Reach(ThemedSession s, string name) => PadAudit.Reach(Sheet(s), s.Pad, e => e is Control { Name: { } n } && n == name);

        // The box reached and opened on the keyboard, what it held erased with B, the text typed and the keyboard put away with Start.
        internal static void Type(ThemedSession s, string box, string text)
        {
            Reach(s, box);
            s.Pad.A();
            OnScreenKeyboard keyboard = OnScreenKeyboard.OpenOver(s.Window)!;
            Assert.NotNull(keyboard);
            var target = (TextBox)Sheet(s).GetVisualDescendants().OfType<Control>().Single(c => c.Name == box);
            for (int n = target.Text?.Length ?? 0; n > 0; n--) s.Pad.B();
            PadCheatsTests.TypeByPad(s.Pad, keyboard, text);
            s.Pad.Start();
            Assert.Null(OnScreenKeyboard.OpenOver(s.Window));
        }

        // A dialog's button, reached and pressed by the pad.
        internal static void Answer(ThemedSession s, string button)
        {
            Assert.Equal("DialogWindow", Sheets(s).Current!.GetType().Name);
            PadAudit.Reach(Sheet(s), s.Pad, e => e is Button { Content: string text } && text == button);
            s.Pad.A();
            s.Settle();
        }

        internal static IReadOnlyDictionary<string, string> StoredEdits(ThemedSession s, string game)
        {
            using GameRecords records = GameRecords.Open(GameRecords.DefaultPath);
            return records.Edits(Path.Combine(s.RomDirectory, game));
        }

        internal static IReadOnlyList<string> ListedTitles(ThemedSession s) =>
            s.Themed.Stage!.Current.Data.System.Games.Select(g => g.Name).ToList();

        [Fact]
        public Task Select_opens_the_game_options_as_a_sheet_and_select_again_puts_them_away() => ThemedLibraryPadTests.Run(s =>
        {
            s.Pad.Select();
            Assert.False(Sheets(s).IsPresenting);

            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.Down();
            GameOptionsWindow options = OpenOptions(s);
            Assert.Empty(s.Window.OwnedWindows);
            Assert.Equal(ThemedSession.SnesGames[1], ((TextBlock)Sheet(s).GetVisualDescendants().OfType<Control>().Single(c => c.Name == "GameOptionsTitle")).Text);
            Assert.Equal(new[] { "Add to Favourites", "Edit This Game's Metadata", "Scrape This Game..." }, options.Entries.Select(b => b.Content as string));

            // The view hears nothing under the sheet.
            s.Pad.Down();
            Assert.Equal(ThemedSession.SnesGames[1], s.Game);
            s.Pad.Select();
            Assert.False(Sheets(s).IsPresenting);
            Assert.Null(s.Window.GameOptionsShown);
            s.Pad.Down();
            Assert.Equal(ThemedSession.SnesGames[2], s.Game);

            OpenOptions(s);
            s.Pad.B();
            Assert.False(Sheets(s).IsPresenting);
            Assert.Equal("gamelist", s.View);
        });

        // P84: the menu and the editor leave nothing unreachable to the pad at 1280 by 800.
        [Fact]
        public Task Every_control_of_the_options_and_the_editor_is_reached_by_the_pad() => ThemedLibraryPadTests.Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            OpenOptions(s);
            var missing = new List<string>();
            void Audit(string what)
            {
                s.Window.UpdateLayout();
                Control sheet = Sheet(s);
                HashSet<InputElement> reached = PadAudit.Reachable(sheet, s.Pad);
                List<InputElement> operable = PadAudit.Operable(sheet);
                _out.WriteLine($"{what}: {operable.Count} controls, {operable.Count(reached.Contains)} reached");
                missing.AddRange(operable.Where(c => !reached.Contains(c)).Select(c => $"[{what}] {PadAudit.Describe(c)}"));
            }
            Audit("options");
            s.Pad.Select();
            MetadataEditorWindow editor = OpenEditor(s);

            // Every Reset shows once its field is edited, so the audit sees them all.
            Reach(s, "Meta_" + GameMetadata.Rating);
            s.Pad.Right();
            foreach (MetadataField field in GameMetadata.Fields.Where(f => f.Kind is MetadataKind.Text or MetadataKind.LongText))
                ((TextBox)editor.EditorOf(field.Key)).Text = "x";
            foreach (MetadataField field in GameMetadata.Fields.Where(f => f.Kind == MetadataKind.Flag))
                ((LunaSwitch)editor.EditorOf(field.Key)).IsChecked = true;
            Reach(s, "Meta_" + GameMetadata.ReleaseDate);
            s.Pad.Right();
            Assert.All(GameMetadata.Fields, f => Assert.True(editor.ResetOf(f.Key).IsVisible, f.Key));
            Audit("editor");
            foreach (string m in missing) _out.WriteLine("unreachable: " + m);
            Assert.Empty(missing);
        });

        [Fact]
        public Task The_editor_sets_every_kind_of_field_by_pad_and_save_shows_them_in_the_view_and_in_mistress_s_library() => ThemedLibraryPadTests.Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            MetadataEditorWindow editor = OpenEditor(s);
            Assert.Equal("From the file name.", editor.RowOf(GameMetadata.Name).Hint);

            Type(s, "Meta_" + GameMetadata.Name, "zeta 2");
            Type(s, "Meta_" + GameMetadata.Developer, "my studio");
            Reach(s, "Meta_" + GameMetadata.Rating);
            s.Pad.Right(7);
            Reach(s, "Meta_" + GameMetadata.ReleaseDate);
            s.Pad.Right();
            s.Pad.A();
            s.Pad.Right(2);
            Reach(s, "Meta_" + GameMetadata.Completed);
            s.Pad.A();
            Reach(s, "Meta_favourite");
            s.Pad.A();
            Assert.Equal("Your edit.", editor.RowOf(GameMetadata.Name).Hint);
            Assert.True(editor.ResetOf(GameMetadata.Name).IsVisible);
            Assert.False(editor.ResetOf(GameMetadata.Publisher).IsVisible);

            Reach(s, "MetadataSave");
            s.Pad.A();
            s.Settle();
            Assert.False(Sheets(s).IsPresenting);

            string file = Path.Combine(s.RomDirectory, ThemedSession.SnesGames[0] + ".sfc");
            SceneGame game = s.Themed.Stage!.Current.Data.System.Games.Single(g => g.File == file);
            Assert.Equal(("zeta 2", "my studio", 0.7f, new DateTime(1990, 3, 1), true, true),
                (game.Name, game.Developer, game.Rating, game.ReleaseDate, game.Completed, game.Favorite));
            Assert.Equal(file, s.Themed.SelectedGame!.File);
            IReadOnlyDictionary<string, string> stored = StoredEdits(s, ThemedSession.SnesGames[0] + ".sfc");
            Assert.Equal(new[] { GameMetadata.Completed, GameMetadata.Developer, GameMetadata.Name, GameMetadata.Rating, GameMetadata.ReleaseDate }, stored.Keys.Order());

            // Mistress's own library shows the name too, and its search finds it.
            var list = s.Window.GetControl<LunaList<RomEntry>>("LibraryList");
            RomEntry entry = list.Models.Single(e => e.FullPath == file);
            Assert.StartsWith("zeta 2", list.Label(entry));
        });

        [Fact]
        public Task A_sort_name_orders_the_gamelist_and_the_name_is_what_is_shown() => ThemedLibraryPadTests.Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            OpenEditor(s);
            Type(s, "Meta_" + GameMetadata.SortName, "zz");
            Reach(s, "MetadataSave");
            s.Pad.A();
            s.Settle();
            Assert.Equal(ThemedSession.SnesGames.Skip(1).Append(ThemedSession.SnesGames[0]), ListedTitles(s));
            Assert.Equal(ThemedSession.SnesGames[0], s.Game);
        });

        [Fact]
        public Task Cancel_and_leaving_without_saving_discard_every_change() => ThemedLibraryPadTests.Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            OpenEditor(s);
            Type(s, "Meta_" + GameMetadata.Name, "gone");
            Reach(s, "Meta_" + GameMetadata.Rating);
            s.Pad.Right(3);
            Reach(s, "Meta_favourite");
            s.Pad.A();
            Reach(s, "MetadataCancel");
            s.Pad.A();
            s.Settle();
            Assert.False(Sheets(s).IsPresenting);
            Assert.Empty(StoredEdits(s, ThemedSession.SnesGames[0] + ".sfc"));
            Assert.Equal(ThemedSession.SnesGames[0], s.Game);
            Assert.False(s.Themed.SelectedGame!.Favorite);

            // B with changes asks, as ES-DE does; Discard keeps nothing and Save keeps them.
            OpenEditor(s);
            Type(s, "Meta_" + GameMetadata.Name, "asked");
            s.Pad.B();
            Answer(s, "Discard");
            Assert.False(Sheets(s).IsPresenting);
            Assert.Empty(StoredEdits(s, ThemedSession.SnesGames[0] + ".sfc"));

            OpenEditor(s);
            Type(s, "Meta_" + GameMetadata.Name, "kept");
            s.Pad.B();
            Answer(s, "Save");
            Assert.False(Sheets(s).IsPresenting);
            Assert.Equal("kept", s.Game);

            // B with nothing changed closes at once.
            OpenEditor(s);
            s.Pad.B();
            s.Settle();
            Assert.False(Sheets(s).IsPresenting);
        });

        [Fact]
        public Task Reset_returns_a_field_to_its_default_and_save_removes_the_edit() => ThemedLibraryPadTests.Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            OpenEditor(s);
            Type(s, "Meta_" + GameMetadata.Name, "renamed");
            Reach(s, "Meta_" + GameMetadata.Broken);
            s.Pad.A();
            Reach(s, "MetadataSave");
            s.Pad.A();
            s.Settle();
            Assert.Equal("renamed", s.Game);
            Assert.True(s.Themed.SelectedGame!.Broken);

            MetadataEditorWindow editor = OpenEditor(s);
            Reach(s, "MetaReset_" + GameMetadata.Name);
            s.Pad.A();
            Assert.Equal(ThemedSession.SnesGames[0], ((TextBox)editor.EditorOf(GameMetadata.Name)).Text);
            Assert.False(editor.ResetOf(GameMetadata.Name).IsVisible);
            Reach(s, "MetaReset_" + GameMetadata.Broken);
            s.Pad.A();
            Reach(s, "MetadataSave");
            s.Pad.A();
            s.Settle();
            Assert.Empty(StoredEdits(s, ThemedSession.SnesGames[0] + ".sfc"));
            Assert.Equal(ThemedSession.SnesGames[0], s.Game);
            Assert.False(s.Themed.SelectedGame!.Broken);
        });

        // Every file under a folder with its size, write time and SHA-256.
        internal static SortedDictionary<string, string> Fingerprint(string folder)
        {
            var all = new SortedDictionary<string, string>(StringComparer.Ordinal);
            if (!Directory.Exists(folder)) return all;
            foreach (string file in Directory.EnumerateFiles(folder, "*", SearchOption.AllDirectories))
            {
                var info = new FileInfo(file);
                all[Path.GetRelativePath(folder, file)] = $"{info.Length}|{info.LastWriteTimeUtc.Ticks}|{Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(file)))}";
            }
            return all;
        }

        // The user's rule: a ROM file is never deleted or moved; ES-DE's Delete becomes Hide from Library.
        [Fact]
        public Task Hide_from_library_takes_the_game_out_of_every_view_and_touches_no_file() => ThemedLibraryPadTests.Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.Down(2);
            string hidden = ThemedSession.SnesGames[2];
            SortedDictionary<string, string> before = Fingerprint(s.RomDirectory);
            Assert.Equal(ThemedSession.SnesGames.Length + ThemedSession.NesGames.Length + ThemedSession.GbGames.Length, before.Count);

            MetadataEditorWindow editor = OpenEditor(s);
            Assert.DoesNotContain(Sheet(s).GetVisualDescendants().OfType<Button>(), b => b.Content is string t && t.StartsWith("Delete", StringComparison.Ordinal));
            Reach(s, "MetadataHide");
            s.Pad.A();
            Answer(s, "Hide");
            Assert.False(Sheets(s).IsPresenting);

            Assert.DoesNotContain(hidden, ListedTitles(s));
            Assert.Equal(ThemedSession.SnesGames.Length - 1, ListedTitles(s).Count);
            var list = s.Window.GetControl<LunaList<RomEntry>>("LibraryList");
            Assert.DoesNotContain(list.Models, e => e.Title == hidden);
            Assert.Equal(before, Fingerprint(s.RomDirectory));

            // Listed again once Preferences asks for hidden games, and unhidden in the editor.
            ((AppSettings)typeof(MainWindow).GetField("_appSettings", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(s.Window)!).ShowHiddenGames = true;
            typeof(MainWindow).GetMethod("ShowLibraryEntries", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(s.Window, null);
            s.Settle();
            Assert.Contains(hidden, ListedTitles(s));
            for (int guard = 0; guard < 8 && s.Game != hidden; guard++) s.Pad.Down();
            Assert.True(s.Themed.SelectedGame!.Hidden);
            OpenEditor(s);
            Reach(s, "Meta_" + GameMetadata.Hidden);
            s.Pad.A();
            Reach(s, "MetadataSave");
            s.Pad.A();
            s.Settle();
            Assert.Empty(StoredEdits(s, hidden + ".sfc"));
            Assert.Equal(before, Fingerprint(s.RomDirectory));
        });

        // §15.14's class of defect: a sheet is not an owned window, so Mistress closing must close it and its subscription.
        [Fact]
        public Task Closing_mistress_closes_the_editor_and_its_subscription_to_scraping() => Session.Dispatch(() =>
        {
            var s = new ThemedSession();
            try
            {
                ThemedLibraryPadTests.Enter(s, "snes");
                MetadataEditorWindow editor = OpenEditor(s);
                Assert.True(editor.Subscribed);
                s.Window.Close();
                s.Settle();
                Assert.False(editor.Subscribed);
                Assert.Null(s.Window.MetadataEditorShown);
                Assert.False(Sheets(s).IsPresenting);
            }
            finally { s.Dispose(); }
        }, default);
    }
}
