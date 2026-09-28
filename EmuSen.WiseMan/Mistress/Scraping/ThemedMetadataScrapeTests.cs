using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using EmuSen.Galaxia.Library;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Mistress.BigPicture;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // The player's edits against ScreenScraper's answers: a re-scrape never overwrites one, the editor's own scrape fills its fields unsaved, and Clear removes only what Mistress keeps - see EmuSen_Settings_Reference.md §4.59.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemedMetadataScrapeTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ThemedMetadataScrapeTests).GetTypeInfo().Assembly);

        private static readonly FieldInfo ConfirmField = typeof(MainWindow).GetField("ConfirmScrape", BindingFlags.Static | BindingFlags.NonPublic)!;

        private const string Scraped = "Scraped description.";

        // The fake's name for the file's region (us), which ScrapeRules takes before its own (ss).
        private const string ScrapedName = "Aurora Drift (US title)";

        private readonly FakeScreenScraper _server = new();
        private readonly object _realConfirm = ConfirmField.GetValue(null)!;

        public ThemedMetadataScrapeTests()
        {
            NoNetwork.HttpFactory.SetValue(null, (Func<HttpClient>)(() => new HttpClient(_server)));
            NoNetwork.ScrapeClock.SetValue(null, new FakeScrapeClock(new DateTimeOffset(2026, 7, 14, 10, 0, 0, TimeSpan.Zero)));
            NoNetwork.DeveloperSource.SetValue(null, (Func<DeveloperCredentials?>)(() => FakeScreenScraper.Developer));
            ConfirmField.SetValue(null, (Func<MainWindow, string, string, Task<bool>>)((_, _, _) => Task.FromResult(true)));
            _server.Games.Add(new FakeGame(9, "Aurora Drift", RomHashes.Of(SyntheticRom.BuildBlank()).Md5) { Synopsis = Scraped });
        }

        public void Dispose()
        {
            ConfirmField.SetValue(null, _realConfirm);
            NoNetwork.Refuse();
            NoNetwork.ScrapeClock.SetValue(null, SystemScrapeClock.Instance);
        }

        private static string File0(ThemedSession s) => Path.Combine(s.RomDirectory, ThemedSession.SnesGames[0] + ".sfc");

        private static SceneGame Shown(ThemedSession s) => s.Themed.Stage!.Current.Data.System.Games.Single(g => g.File == File0(s));

        private static void Until(ThemedSession s, Func<bool> done, string what)
        {
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (!done())
            {
                s.Settle();
                if (clock.ElapsedMilliseconds > 20_000) Assert.Fail(what);
                Thread.Sleep(5);
            }
            s.Settle();
        }

        // The options menu's Scrape This Game, then the run waited out.
        private static void ScrapeFromTheOptions(ThemedSession s)
        {
            ThemedGameOptionsTests.Choose(s, "Scrape This Game...");
            Until(s, () => !s.Window.ScrapeRunning && Shown(s).Publisher == "Synthetic Publisher", "the scrape never finished");
            PutAwayTheStatus(s);
        }

        // Every run shows stage (d)'s status sheet; B puts it away, back to whatever was under it.
        private static void PutAwayTheStatus(ThemedSession s)
        {
            Assert.IsType<ScrapeStatusWindow>(ThemedGameOptionsTests.Sheets(s).Current);
            s.Pad.B();
            s.Settle();
            Assert.False(ThemedGameOptionsTests.Sheets(s).Current is ScrapeStatusWindow);
        }

        private static void EditDescription(ThemedSession s, string text)
        {
            ThemedGameOptionsTests.OpenEditor(s);
            ThemedGameOptionsTests.Type(s, "Meta_" + GameMetadata.Description, text);
            ThemedGameOptionsTests.Reach(s, "MetadataSave");
            s.Pad.A();
            s.Settle();
        }

        [Fact]
        public Task An_edit_survives_a_rescrape_and_reset_returns_the_scraped_value() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            ThemedLibraryPadTests.Enter(s, "snes");
            EditDescription(s, "mine");
            Assert.Equal("mine", Shown(s).Description);

            ScrapeFromTheOptions(s);
            Assert.Equal(("mine", "Synthetic Developer", "Synthetic Publisher"), (Shown(s).Description, Shown(s).Developer, Shown(s).Publisher));
            Assert.Equal(Scraped, s.Window.ScrapedNow(File0(s))!.Description);
            Assert.Single(_server.JeuInfos);

            // A second run, from the pad menu this time, leaves the edit too.
            var entries = (List<PadMenuEntry>)typeof(MainWindow).GetField("_padMenuEntries", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(s.Window)!;
            s.Pad.Start();
            s.Pad.Down(entries.FindIndex(e => e.Text() == "Scrape This Game..."));
            s.Pad.A();
            Until(s, () => !s.Window.ScrapeRunning, "the second scrape never finished");
            PutAwayTheStatus(s);
            Assert.Equal("mine", Shown(s).Description);
            Assert.Equal("mine", ThemedGameOptionsTests.StoredEdits(s, ThemedSession.SnesGames[0] + ".sfc")[GameMetadata.Description]);

            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
            Assert.Equal("Your edit, shown in place of ScreenScraper's.", editor.HintOf(GameMetadata.Description));
            Assert.Equal("From ScreenScraper.", editor.HintOf(GameMetadata.Developer));
            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.Description);
            s.Pad.X();
            Assert.Equal(Scraped, ((TextBox)editor.EditorOf(GameMetadata.Description)).Text);
            ThemedGameOptionsTests.Reach(s, "MetadataSave");
            s.Pad.A();
            s.Settle();
            Assert.Equal(Scraped, Shown(s).Description);
            Assert.Empty(ThemedGameOptionsTests.StoredEdits(s, ThemedSession.SnesGames[0] + ".sfc"));
        }, default);

        // ES-DE's editor fills its fields from its own scrape and saves them only on Save.
        [Fact]
        public Task The_editor_s_own_scrape_fills_its_fields_unsaved_so_cancel_keeps_the_edit_and_save_takes_the_answer() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            ThemedLibraryPadTests.Enter(s, "snes");
            EditDescription(s, "mine");

            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
            s.Pad.Y();
            Until(s, () => editor.Status.StartsWith("ScreenScraper's answer", StringComparison.Ordinal), "the editor's scrape never filled its fields");
            PutAwayTheStatus(s);
            Assert.Same(editor, ThemedGameOptionsTests.Sheets(s).Current);
            Assert.Equal(Scraped, ((TextBox)editor.EditorOf(GameMetadata.Description)).Text);
            Assert.Equal("Synthetic Developer", ((TextBox)editor.EditorOf(GameMetadata.Developer)).Text);
            Assert.Equal("From this scrape; Save keeps it.", editor.HintOf(GameMetadata.Description));
            ThemedGameOptionsTests.Reach(s, "MetadataCancel");
            s.Pad.A();
            s.Settle();
            Assert.Equal("mine", Shown(s).Description);
            Assert.Equal("Synthetic Developer", Shown(s).Developer);

            editor = ThemedGameOptionsTests.OpenEditor(s);
            s.Pad.Y();
            Until(s, () => editor.Status.StartsWith("ScreenScraper's answer", StringComparison.Ordinal), "the second scrape never filled the fields");
            PutAwayTheStatus(s);
            ThemedGameOptionsTests.Reach(s, "MetadataSave");
            s.Pad.A();
            s.Settle();
            Assert.Equal(Scraped, Shown(s).Description);
            Assert.Empty(ThemedGameOptionsTests.StoredEdits(s, ThemedSession.SnesGames[0] + ".sfc"));
        }, default);

        // The editor's own scrape (Y), waited out and its status sheet put away, back on the editor.
        private static void ScrapeInTheEditor(ThemedSession s, MetadataEditorWindow editor)
        {
            s.Pad.Y();
            Until(s, () => editor.Status.StartsWith("ScreenScraper's answer", StringComparison.Ordinal), "the editor's scrape never answered");
            PutAwayTheStatus(s);
            Assert.Same(editor, ThemedGameOptionsTests.Sheets(s).Current);
        }

        private static string NameBox(MetadataEditorWindow editor) => ((TextBox)editor.EditorOf(GameMetadata.Name)).Text ?? "";

        private static Control NameOffer(ThemedSession s) =>
            Avalonia.VisualTree.VisualExtensions.GetVisualDescendants(ThemedGameOptionsTests.Sheet(s)).OfType<Control>().Single(c => c.Name == "MetadataNameOffer");

        // Q18: ScreenScraper's name is offered, never put in the field by the scrape, and a Save without taking it stores no name.
        [Fact]
        public Task The_editor_s_scrape_offers_screen_scraper_s_name_and_never_puts_it_in_the_field() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            ThemedLibraryPadTests.Enter(s, "snes");
            string stem = ThemedSession.SnesGames[0];
            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
            Assert.False(NameOffer(s).IsVisible);
            Assert.Null(editor.OfferedName);

            ScrapeInTheEditor(s, editor);
            Assert.Equal(stem, NameBox(editor));
            Assert.Equal("From the file name.", editor.HintOf(GameMetadata.Name));
            Assert.Equal(Scraped, ((TextBox)editor.EditorOf(GameMetadata.Description)).Text);
            Assert.True(NameOffer(s).IsVisible);
            Assert.Equal(ScrapedName, editor.OfferedName);
            Assert.Contains("offered under Name", editor.Status);

            // Every control reached by the pad with the offer shown, its two buttons among them.
            s.Window.UpdateLayout();
            HashSet<Avalonia.Input.InputElement> reached = PadAudit.Reachable(ThemedGameOptionsTests.Sheet(s), s.Pad);
            List<Avalonia.Input.InputElement> operable = PadAudit.Operable(ThemedGameOptionsTests.Sheet(s));
            Assert.Contains(operable, c => c is Control { Name: "MetadataUseScrapedName" });
            Assert.Contains(operable, c => c is Control { Name: "MetadataKeepName" });
            Assert.Empty(operable.Where(c => !reached.Contains(c)).Select(PadAudit.Describe));

            ThemedGameOptionsTests.Reach(s, "MetadataSave");
            s.Pad.A();
            s.Settle();
            Assert.DoesNotContain(GameMetadata.Name, ThemedGameOptionsTests.StoredEdits(s, stem + ".sfc").Keys);
            Assert.Equal(stem, Shown(s).Name);
            Assert.Equal("Synthetic Developer", Shown(s).Developer);

            // Opened again on a game already scraped, the editor offers the name again; the field is still the file's.
            editor = ThemedGameOptionsTests.OpenEditor(s);
            Assert.Equal(ScrapedName, editor.OfferedName);
            Assert.Equal(stem, NameBox(editor));
        }, default);

        // Q18: one press takes the offer; Save stores it as the player's edit, and every view shows it.
        [Fact]
        public Task Taking_the_offered_name_stores_it_as_an_edit() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            ThemedLibraryPadTests.Enter(s, "snes");
            string stem = ThemedSession.SnesGames[0];
            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
            ScrapeInTheEditor(s, editor);

            ThemedGameOptionsTests.Reach(s, "MetadataUseScrapedName");
            s.Pad.A();
            s.Settle();
            Assert.Equal(ScrapedName, NameBox(editor));
            Assert.Equal("Your edit.", editor.HintOf(GameMetadata.Name));
            Assert.False(NameOffer(s).IsVisible);
            Assert.True(editor.CanReset(GameMetadata.Name));

            ThemedGameOptionsTests.Reach(s, "MetadataSave");
            s.Pad.A();
            s.Settle();
            Assert.Equal(ScrapedName, ThemedGameOptionsTests.StoredEdits(s, stem + ".sfc")[GameMetadata.Name]);
            Assert.Equal(ScrapedName, Shown(s).Name);
            var list = (EmuSen.LunaP.Controls.LunaList<RomEntry>)s.Window.GetControl<ListBox>("LibraryList");
            Assert.Contains(list.Models.Select(list.Label), l => l.StartsWith(ScrapedName, StringComparison.Ordinal));
        }, default);

        // Q18: Keep Current Name puts the offer away and the player's own name stays, before and after Save.
        [Fact]
        public Task Declining_the_offered_name_leaves_the_name_unchanged() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            ThemedLibraryPadTests.Enter(s, "snes");
            string stem = ThemedSession.SnesGames[0];
            ThemedGameOptionsTests.OpenEditor(s);
            ThemedGameOptionsTests.Type(s, "Meta_" + GameMetadata.Name, "mine");
            ThemedGameOptionsTests.Reach(s, "MetadataSave");
            s.Pad.A();
            s.Settle();
            Assert.Equal("mine", Shown(s).Name);

            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
            ScrapeInTheEditor(s, editor);
            Assert.Equal("mine", NameBox(editor));
            Assert.Equal(ScrapedName, editor.OfferedName);

            ThemedGameOptionsTests.Reach(s, "MetadataKeepName");
            s.Pad.A();
            s.Settle();
            Assert.False(NameOffer(s).IsVisible);
            Assert.Null(editor.OfferedName);
            Assert.Equal("mine", NameBox(editor));
            Assert.False(editor.Draft.Changes().ContainsKey(GameMetadata.Name));

            ThemedGameOptionsTests.Reach(s, "MetadataSave");
            s.Pad.A();
            s.Settle();
            Assert.Equal("mine", ThemedGameOptionsTests.StoredEdits(s, stem + ".sfc")[GameMetadata.Name]);
            Assert.Equal("mine", Shown(s).Name);
        }, default);

        // ES-DE's Clear removes the metadata and media; here only Mistress's own store is touched, never the ROM.
        [Fact]
        public Task Clear_removes_the_edits_and_the_scraped_text_and_pictures_and_touches_no_rom() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            ThemedLibraryPadTests.Enter(s, "snes");
            ScrapeFromTheOptions(s);
            string stem = ThemedSession.SnesGames[0];
            string[] pictures = Directory.EnumerateFiles(DataStore.Media, stem + ".*", SearchOption.AllDirectories).ToArray();
            Assert.NotEmpty(pictures);
            ThemedGameOptionsTests.Choose(s, "Add to Favourites");
            EditDescription(s, "mine");
            SortedDictionary<string, string> roms = ThemedGameOptionsTests.Fingerprint(s.RomDirectory);

            ThemedGameOptionsTests.OpenEditor(s);
            ThemedGameOptionsTests.Reach(s, "MetadataClear");
            s.Pad.A();
            ThemedGameOptionsTests.Answer(s, "Clear");
            Assert.False(ThemedGameOptionsTests.Sheets(s).IsPresenting);

            Assert.Empty(ThemedGameOptionsTests.StoredEdits(s, stem + ".sfc"));
            Assert.Null(s.Window.ScrapedNow(File0(s)));
            Assert.All(pictures, p => Assert.False(File.Exists(p), p));
            Assert.Null(Shown(s).Description);
            Assert.True(Shown(s).Favorite);
            Assert.Equal(roms, ThemedGameOptionsTests.Fingerprint(s.RomDirectory));
        }, default);

        // ES-DE's red for a value its scrape put in, drawn only on the fields the scrape filled; the name stays grey, its offer red (§34).
        [Fact]
        public Task The_editor_draws_what_its_scrape_filled_in_red() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            ThemedLibraryPadTests.Enter(s, "snes");
            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
            Avalonia.Rect Value(Control editorOf)
            {
                var row = Avalonia.VisualTree.VisualExtensions.GetVisualDescendants(editorOf).OfType<EmuSen.LunaP.Controls.MenuRow>().First();
                return EsdeMenusTests.InWindow(row, row.Layout(row.Bounds.Size).Value, s.Window);
            }
            int Red(RenderedFrame f, Avalonia.Rect box)
            {
                int n = 0;
                for (int y = (int)box.Top; y < (int)box.Bottom; y++)
                    for (int x = (int)box.Left; x < (int)box.Right; x++)
                    {
                        (byte r, byte g, byte b) = EsdeMenusTests.At(f, x, y);
                        Avalonia.Media.Color c = MetadataEditorWindow.ScrapedColor;
                        if (Math.Abs(r - c.R) <= 14 && Math.Abs(g - c.G) <= 14 && Math.Abs(b - c.B) <= 14) n++;
                    }
                return n;
            }
            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.SortName);
            Assert.True(Red(s.Capture(), Value(editor.EditorOf(GameMetadata.Description))) < 5);

            ScrapeInTheEditor(s, editor);
            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.SortName);
            RenderedFrame after = s.Capture();
            Assert.True(Red(after, Value(editor.EditorOf(GameMetadata.Description))) > 40, "the scraped description is not red");
            Assert.True(Red(after, Value(editor.EditorOf(GameMetadata.Name))) < 5, "the name, which the scrape only offers, is red");
            Assert.True(Red(after, Value(ThemedCollectionsTests.Named<Button>(s, "MetadataUseScrapedName"))) > 40, "the offered name is not red");
        }, default);

        // The big-screen editor after its own scrape: the values it filled in ES-DE's red and ScreenScraper's name offered under Name (§34); written outside the repository.
        [EmuSen.WiseMan.Mistress.BigPicture.MetadataEditorPngFact]
        public Task Pictures_of_the_editor_after_its_scrape() => Session.Dispatch(() =>
        {
            foreach ((int w, int h) in new[] { (1280, 800), (1920, 1200) })
            {
                using var s = new ThemedSession(w, h);
                ThemedLibraryPadTests.Enter(s, "snes");
                MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
                ScrapeInTheEditor(s, editor);
                ThemedGameOptionsTests.Reach(s, "MetadataUseScrapedName");
                s.Settle();
                Directory.CreateDirectory(EmuSen.WiseMan.Mistress.BigPicture.MetadataEditorPictureTool.PngFolder);
                s.Capture().SavePng(Path.Combine(EmuSen.WiseMan.Mistress.BigPicture.MetadataEditorPictureTool.PngFolder, $"synthetic-{w}x{h}-editor-after-scrape.png"));
            }
        }, default);
    }
}
