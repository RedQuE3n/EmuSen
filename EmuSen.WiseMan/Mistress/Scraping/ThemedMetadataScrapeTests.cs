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
            Assert.Equal("Your edit, shown in place of ScreenScraper's.", editor.RowOf(GameMetadata.Description).Hint);
            Assert.Equal("From ScreenScraper.", editor.RowOf(GameMetadata.Developer).Hint);
            ThemedGameOptionsTests.Reach(s, "MetaReset_" + GameMetadata.Description);
            s.Pad.A();
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
            Assert.Equal("From this scrape; Save keeps it.", editor.RowOf(GameMetadata.Description).Hint);
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
    }
}
