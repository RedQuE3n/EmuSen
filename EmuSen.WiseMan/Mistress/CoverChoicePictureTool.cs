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
using Avalonia.Media;
using Avalonia.Media.Imaging;
using Avalonia.VisualTree;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Mistress.BigPicture;
using EmuSen.WiseMan.Mistress.Scraping;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress
{
    public sealed class CoverChoicePngFactAttribute : FactAttribute
    {
        public CoverChoicePngFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_PNG") != "1")
                Skip = "Writes Q42's row and Q45's picker to ~/.cache/emusen/bigpicture/png/q40-q45/; set EMUSEN_BIGPICTURE_PNG=1 - see EmuSen_BigPicture.md §28";
        }
    }

    // Q42's OpenVGDB row and Q45's picker, on the desktop and as a sheet, at 1280 by 800; a desktop window is drawn over the main window where it opens.
    [Collection(TestCollections.ProcessGlobals)]
    public class CoverChoicePictureTool
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(CoverChoicePictureTool).GetTypeInfo().Assembly);

        public static readonly string PngFolder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture", "png", "q40-q45");

        private readonly ITestOutputHelper _out;

        public CoverChoicePictureTool(ITestOutputHelper output) => _out = output;

        private void Save(RenderedFrame frame, string name)
        {
            Assert.Equal((1280, 800), (frame.Width, frame.Height));
            Directory.CreateDirectory(PngFolder);
            string path = Path.Combine(PngFolder, name + ".png");
            frame.SavePng(path);
            _out.WriteLine(path);
        }

        private static RenderedFrame Frame(TopLevel top)
        {
            top.CaptureRenderedFrame()?.Dispose();
            foreach (Visual v in top.GetSelfAndVisualDescendants()) v.InvalidateVisual();
            using WriteableBitmap bitmap = top.CaptureRenderedFrame() ?? throw new InvalidOperationException("no frame");
            return UiTest.Capture(bitmap);
        }

        // A child taller than the window is drawn by its bottom, where the rows these pictures are about are.
        private static RenderedFrame Centred(ThemedSession s, Window child)
        {
            RenderedFrame main = s.Capture();
            RenderedFrame over = Frame(child);
            byte[] pixels = (byte[])main.Rgba.Clone();
            int left = (main.Width - over.Width) / 2, top = over.Height > main.Height ? main.Height - over.Height : (main.Height - over.Height) / 2;
            for (int y = 0; y < over.Height; y++)
                for (int x = 0; x < over.Width; x++)
                {
                    int tx = left + x, ty = top + y;
                    if (tx < 0 || ty < 0 || tx >= main.Width || ty >= main.Height) continue;
                    Array.Copy(over.Rgba, (y * over.Width + x) * 4, pixels, (ty * main.Width + tx) * 4, 4);
                }
            return new RenderedFrame(pixels, main.Width, main.Height);
        }

        private static string Rom(ThemedSession s, int n) => Path.Combine(s.RomDirectory, ThemedSession.SnesGames[n] + ".sfc");

        private static void Art(ThemedSession s)
        {
            (int n, Color a, Color b)[] art = [(0, Colors.Crimson, Colors.Gold), (1, Colors.SteelBlue, Colors.Wheat), (3, Colors.Teal, Colors.White), (4, Colors.DarkOliveGreen, Colors.Orange)];
            foreach ((int n, Color a, Color b) in art)
            {
                string to = Path.Combine(DataStore.Artwork, "SNES", ThemedSession.SnesGames[n] + ".png");
                Directory.CreateDirectory(Path.GetDirectoryName(to)!);
                File.Copy(SceneAssets.Halves($"q45-{n}", 120, 160, a, b), to, overwrite: true);
            }
            typeof(MainWindow).GetMethod("ScanArtwork", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(s.Window, null);
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (CoverChoiceTests.CoverShown(s, 0) is null && clock.ElapsedMilliseconds < 10_000) { s.Settle(); Thread.Sleep(5); }
        }

        private static void Wait(ThemedSession s, int ms = 400)
        {
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (clock.ElapsedMilliseconds < ms) { s.Settle(); Thread.Sleep(10); }
        }

        [CoverChoicePngFact]
        public Task Q42_the_openvgdb_row_downloaded_and_removed() => Session.Dispatch(() =>
        {
            using ThemedSession s = DesktopGameOptionsTests.Desktop(AppSettings.LibraryGrid);
            OnlineCoverTests.BuildOpenVgdb(OpenVgdb.DefaultPath, ("SNES", "Aurora Drift (Synthetic)", "00", null));
            FieldInfo confirm = typeof(MainWindow).GetField("ConfirmRemoveOpenVgdb", BindingFlags.Static | BindingFlags.NonPublic)!;
            object real = confirm.GetValue(null)!;
            confirm.SetValue(null, (Func<MainWindow, Task<bool>>)(_ => Task.FromResult(true)));
            try
            {
                var prefs = new PreferencesWindow(new AppSettings { RomDirectory = s.RomDirectory }, s.Window) { Width = 1000, Height = 700 };
                prefs.Show();
                prefs.ShowTab(PreferencesWindow.ScrapingTab);
                Wait(s);
                Button remove = prefs.GetVisualDescendants().OfType<Button>().Single(b => b.Name == "OpenVgdbRemoveButton");
                foreach (ScrollViewer sv in remove.GetVisualAncestors().OfType<ScrollViewer>()) sv.ScrollToEnd();
                remove.Focus(NavigationMethod.Directional);
                Wait(s);
                Save(Centred(s, prefs), "q42-openvgdb-row-downloaded");
                remove.RaiseEvent(new Avalonia.Interactivity.RoutedEventArgs(Button.ClickEvent));
                Wait(s);
                foreach (ScrollViewer sv in prefs.GetVisualDescendants().OfType<Button>().Single(b => b.Name == "OpenVgdbDownloadButton").GetVisualAncestors().OfType<ScrollViewer>()) sv.ScrollToEnd();
                Wait(s);
                Save(Centred(s, prefs), "q42-openvgdb-row-removed");
                prefs.Close();
            }
            finally { confirm.SetValue(null, real); }
        }, default);

        [CoverChoicePngFact]
        public Task Q45_the_picker_on_the_desktop_and_the_grid_after() => Session.Dispatch(() =>
        {
            using ThemedSession s = DesktopGameOptionsTests.Desktop(AppSettings.LibraryGrid);
            Art(s);
            Wait(s);
            string title = ArtworkIndex.Untagged(ThemedSession.SnesGames[2]);
            Control tile = s.Window.GetVisualDescendants().OfType<EmuSen.Mistress.Views.Covers.CoverTile>().Single(t => t.IsEffectivelyVisible && t.Title == title);
            Point at = tile.TranslatePoint(new Point(tile.Bounds.Width / 2, tile.Bounds.Height / 2), s.Window)!.Value;
            s.Window.MouseDown(at, MouseButton.Right);
            s.Window.MouseUp(at, MouseButton.Right);
            s.Settle();
            Save(s.Capture(), "q45-desktop-context-menu");
            DesktopGameOptionsTests.ChooseFromContextMenu(s, s.Window.GetControl<TileGrid<RomEntry>>("LibraryGrid"), "Use Another _Game's Cover...");
            CoverPickerWindow picker = s.Window.CoverPickerShown!;
            Wait(s);
            Save(Centred(s, picker), "q45-desktop-picker");
            picker.Choose(picker.Listed.Single(c => c.Path == Rom(s, 3)));
            Wait(s, 800);
            Save(s.Capture(), "q45-desktop-grid-after");
        }, default);

        [CoverChoicePngFact]
        public Task Q45_the_picker_as_a_sheet_in_big_picture() => ThemedLibraryPadTests.Run(s =>
        {
            Art(s);
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.Down();
            s.Pad.Down();
            ThemedGameOptionsTests.Choose(s, "Use Another Game's Cover...");
            Wait(s);
            Save(s.Capture(), "q45-sheet-picker");
            ThemedGameOptionsTests.Type(s, "CoverPickerSearch", "dune");
            Wait(s);
            Save(s.Capture(), "q45-sheet-picker-searched");
            PadAudit.Reach(ThemedGameOptionsTests.Sheet(s), s.Pad, e => e is Button { Tag: CoverCandidate c } && c.Path == Rom(s, 3));
            s.Pad.A();
            Wait(s, 800);
            Save(s.Capture(), "q45-themed-gamelist-after");
        });
    }
}
