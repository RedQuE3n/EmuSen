using System;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.Media.Imaging;
using Avalonia.VisualTree;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Mistress.BigPicture;
using EmuSen.WiseMan.Mistress.Scraping;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress
{
    public sealed class DesktopOptionsPngFactAttribute : FactAttribute
    {
        public DesktopOptionsPngFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_PNG") != "1")
                Skip = "Writes the desktop game options' pictures to ~/.cache/emusen/bigpicture/png/desktop-options/; set EMUSEN_BIGPICTURE_PNG=1 - see EmuSen_BigPicture.md §27";
        }
    }

    // Q18's offer and Q19's menu and editor at 1280 by 800; a desktop window or menu is drawn over the main window where it opens.
    [Collection(TestCollections.ProcessGlobals)]
    public class DesktopOptionsPictureTool : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(DesktopOptionsPictureTool).GetTypeInfo().Assembly);

        private static readonly FieldInfo ConfirmField = typeof(MainWindow).GetField("ConfirmScrape", BindingFlags.Static | BindingFlags.NonPublic)!;

        public static readonly string PngFolder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture", "png", "desktop-options");

        private readonly ITestOutputHelper _out;
        private readonly FakeScreenScraper _server = new();
        private readonly object _realConfirm = ConfirmField.GetValue(null)!;

        public DesktopOptionsPictureTool(ITestOutputHelper output)
        {
            _out = output;
            NoNetwork.HttpFactory.SetValue(null, (Func<HttpClient>)(() => new HttpClient(_server)));
            NoNetwork.ScrapeClock.SetValue(null, new FakeScrapeClock(new DateTimeOffset(2026, 7, 14, 10, 0, 0, TimeSpan.Zero)));
            NoNetwork.DeveloperSource.SetValue(null, (Func<DeveloperCredentials?>)(() => FakeScreenScraper.Developer));
            ConfirmField.SetValue(null, (Func<MainWindow, string, string, Task<bool>>)((_, _, _) => Task.FromResult(true)));
            _server.Games.Add(new FakeGame(9, "Aurora Drift", RomHashes.Of(SyntheticRom.BuildBlank()).Md5) { Synopsis = "Scraped description." });
        }

        public void Dispose()
        {
            ConfirmField.SetValue(null, _realConfirm);
            NoNetwork.Refuse();
            NoNetwork.ScrapeClock.SetValue(null, SystemScrapeClock.Instance);
        }

        private void Save(RenderedFrame frame, string name)
        {
            Assert.Equal((1280, 800), (frame.Width, frame.Height));
            Directory.CreateDirectory(PngFolder);
            string path = Path.Combine(PngFolder, name + ".png");
            frame.SavePng(path);
            _out.WriteLine(path);
        }

        // A top level's frame pasted into the main window's at a point, as the desktop would show it there.
        private static RenderedFrame Over(RenderedFrame main, RenderedFrame child, int left, int top)
        {
            byte[] pixels = (byte[])main.Rgba.Clone();
            for (int y = 0; y < child.Height; y++)
            {
                int ty = top + y;
                if (ty < 0 || ty >= main.Height) continue;
                for (int x = 0; x < child.Width; x++)
                {
                    int tx = left + x;
                    if (tx < 0 || tx >= main.Width) continue;
                    Array.Copy(child.Rgba, (y * child.Width + x) * 4, pixels, (ty * main.Width + tx) * 4, 4);
                }
            }
            return new RenderedFrame(pixels, main.Width, main.Height);
        }

        private static RenderedFrame Frame(TopLevel top)
        {
            top.CaptureRenderedFrame()?.Dispose();
            foreach (Visual v in top.GetSelfAndVisualDescendants()) v.InvalidateVisual();
            using WriteableBitmap bitmap = top.CaptureRenderedFrame() ?? throw new InvalidOperationException("no frame");
            return UiTest.Capture(bitmap);
        }

        private static RenderedFrame Centred(ThemedSession s, Window child)
        {
            RenderedFrame main = s.Capture();
            RenderedFrame over = Frame(child);
            return Over(main, over, (main.Width - over.Width) / 2, (main.Height - over.Height) / 2);
        }

        private static void Until(ThemedSession s, Func<bool> done)
        {
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (!done())
            {
                s.Settle();
                if (clock.ElapsedMilliseconds > 20_000) Assert.Fail("waited too long");
                Thread.Sleep(5);
            }
            s.Settle();
        }

        [DesktopOptionsPngFact]
        public Task Desktop_grid_menu_options_and_editor() => Session.Dispatch(() =>
        {
            using ThemedSession s = DesktopGameOptionsTests.Desktop(AppSettings.LibraryGrid);
            string title = ArtworkIndex.Untagged(ThemedSession.SnesGames[2]);
            Control tile = s.Window.GetVisualDescendants().OfType<EmuSen.Mistress.Views.Covers.CoverTile>().Single(t => t.IsEffectivelyVisible && t.Title == title);
            Point at = tile.TranslatePoint(new Point(tile.Bounds.Width / 2, tile.Bounds.Height / 2), s.Window)!.Value;
            s.Window.MouseDown(at, MouseButton.Right);
            s.Window.MouseUp(at, MouseButton.Right);
            s.Settle();
            ContextMenu menu = s.Window.GetControl<TileGrid<RomEntry>>("LibraryGrid").ContextMenu!;
            Assert.True(menu.IsOpen);
            // Headless draws the menu in the window's overlay layer.
            Save(s.Capture(), "desktop-grid-context-menu");

            DesktopGameOptionsTests.ChooseFromContextMenu(s, s.Window.GetControl<TileGrid<RomEntry>>("LibraryGrid"), "Game _Options...");
            GameOptionsWindow options = s.Window.GameOptionsShown!;
            Save(Centred(s, options), "desktop-options-window");

            DesktopGameOptionsTests.Click(s, options.Entries.Single(b => (string?)b.Content == "Edit This Game's Metadata"));
            MetadataEditorWindow editor = s.Window.MetadataEditorShown!;
            Save(Centred(s, editor), "desktop-editor-window");
            ((TextBox)editor.EditorOf(GameMetadata.Name)).Text = "Cobalt Harbor, renamed on the desktop";
            s.Settle();
            Save(Centred(s, editor), "desktop-editor-window-edited");
            DesktopGameOptionsTests.Click(s, DesktopGameOptionsTests.Named<Button>(editor, "MetadataSave"));
            Save(s.Capture(), "desktop-grid-after-save");

            BigPictureSwitchTests.Choose(s, "_Big Picture");
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.Down(2);
            s.Run(500);
            Save(s.Capture(), "bigpicture-shows-the-desktop-edit");
        }, default);

        [DesktopOptionsPngFact]
        public Task Built_in_big_screen_options_and_editor_sheets() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(settings: a => a.LibraryStyle = AppSettings.LibraryStyleMistress);
            DesktopGameOptionsTests.List(s.Window).Select(DesktopGameOptionsTests.List(s.Window).Models.Single(e => e.FullPath == DesktopGameOptionsTests.SnesPath(s, 1)));
            s.Settle();
            s.Pad.Start();
            Save(s.Capture(), "builtin-pad-menu");
            s.Pad.B();
            BigPictureSwitchTests.ChooseFromPadMenu(s, "Game Options...");
            Save(s.Capture(), "builtin-options-sheet");
            SheetLayer sheets = s.Window.GetControl<SheetLayer>("Sheets");
            PadAudit.Reach(sheets.SheetOf(sheets.Current!)!, s.Pad, e => e is Button { Content: "Edit This Game's Metadata" });
            s.Pad.A();
            s.Settle();
            Save(s.Capture(), "builtin-editor-sheet");
        }, default);

        [DesktopOptionsPngFact]
        public Task The_name_offered_after_the_editor_s_scrape() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            ThemedLibraryPadTests.Enter(s, "snes");
            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
            s.Pad.Y();
            Until(s, () => editor.Status.StartsWith("ScreenScraper's answer", StringComparison.Ordinal));
            s.Pad.B();
            s.Settle();
            Save(s.Capture(), "themed-editor-name-offered");
            ThemedGameOptionsTests.Reach(s, "MetadataUseScrapedName");
            s.Pad.A();
            s.Settle();
            Save(s.Capture(), "themed-editor-name-taken");
            ThemedGameOptionsTests.Reach(s, "MetadataSave");
            s.Pad.A();
            s.Settle();
            s.Run(500);
            Save(s.Capture(), "themed-gamelist-name-taken");
        }, default);
    }
}
