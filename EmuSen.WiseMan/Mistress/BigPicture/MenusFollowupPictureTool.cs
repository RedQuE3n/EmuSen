using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Library;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    public sealed class MenusFollowupPngFactAttribute : FactAttribute
    {
        public MenusFollowupPngFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_PNG") != "1")
                Skip = "Writes the menus follow-ups' pictures to ~/.cache/emusen/bigpicture/png/menus-followups/; set EMUSEN_BIGPICTURE_PNG=1 - see EmuSen_BigPicture.md §40";
        }
    }

    // The lettered help glyphs, the editor's subtitle and per-row help, the scroll indicator and the text popup, at 1280 by 800 and 1920 by 1200 on both themes; written outside the repository.
    [Collection(TestCollections.ProcessGlobals)]
    public class MenusFollowupPictureTool
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(MenusFollowupPictureTool).GetTypeInfo().Assembly);

        public static readonly string PngFolder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture", "png", "menus-followups");

        private readonly ITestOutputHelper _out;

        public MenusFollowupPictureTool(ITestOutputHelper output) => _out = output;

        private void Save(ThemedSession s, string name)
        {
            s.Settle();
            Directory.CreateDirectory(PngFolder);
            string path = Path.Combine(PngFolder, name + ".png");
            s.Capture().SavePng(path);
            _out.WriteLine(path);
        }

        private void Walk(ThemedSession s, string prefix)
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Run(500);
            Save(s, $"{prefix}-gamelist");
            s.Pad.Chord(SDL3.SDL.GamepadButton.Back, SDL3.SDL.GamepadButton.Start);
            Save(s, $"{prefix}-start-menu");
            s.Pad.Up();
            Save(s, $"{prefix}-start-menu-last");
            s.Pad.B();
            s.Settle();

            ThemedGameOptionsTests.OpenEditor(s);
            Save(s, $"{prefix}-editor");
            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.Rating);
            s.Pad.A();
            s.Pad.A();
            s.Pad.A();
            Save(s, $"{prefix}-editor-rating");
            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.Completed);
            Save(s, $"{prefix}-editor-switch");
            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.HideMetadata);
            Save(s, $"{prefix}-editor-scrolled");
            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.Controller);
            Save(s, $"{prefix}-editor-lower");
            var rows = (ScrollViewer)((EmuSen.Mistress.Views.MetadataEditorWindow)ThemedGameOptionsTests.Sheets(s).Current!).Menu!.Child!;
            rows.Offset = new Vector(0, rows.Extent.Height);
            Save(s, $"{prefix}-editor-bottom");
            ThemedGameOptionsTests.Reach(s, "MetadataSave");
            Save(s, $"{prefix}-editor-buttons");

            // The text popup a physical keyboard gets, and the one Steam's keyboard would type into.
            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.Developer);
            s.Pad.Unplug();
            s.Pad.Tick();
            EsdeMenusTests.Press(s, Avalonia.Input.Key.Enter);
            s.Window.KeyTextInput("Halcyon Works");
            Save(s, $"{prefix}-editor-field");
            EsdeMenusTests.Press(s, Avalonia.Input.Key.Escape);
            s.Pad.Replug();
            s.Pad.Tick();
            s.Settings.OnScreenKeyboard = AppSettings.OnScreenKeyboardSteam;
            s.Pad.A();
            Save(s, $"{prefix}-editor-steam");
            s.Pad.B();
            s.Settings.OnScreenKeyboard = AppSettings.OnScreenKeyboardAutomatic;
            ThemedSwitchesTests.PutAway(s);
            if (ThemedGameOptionsTests.Sheets(s).IsPresenting) ThemedGameOptionsTests.Answer(s, "Discard");

            ThemedSwitchesTests.OpenInterface(s);
            Save(s, $"{prefix}-interface");
            ThemedSwitchesTests.PutAway(s);
        }

        // The help bar in each family the setting can choose, over the gamelist and in the editor.
        private void Families(ThemedSession s, string prefix)
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Run(500);
            foreach (string family in new[] { "Xbox", "PlayStation", "Nintendo", "Generic" })
            {
                s.Settings.ControllerType = family;
                s.Pad.Tick();
                s.Run(100);
                Save(s, $"{prefix}-gamelist-{family.ToLowerInvariant()}");
            }
            s.Settings.SwapPadButtons = true;
            s.Settings.ControllerType = "Xbox";
            s.Pad.Tick();
            s.Run(100);
            Save(s, $"{prefix}-gamelist-xbox-swapped");
            s.Settings.SwapPadButtons = false;
            s.Pad.Tick();
        }

        private static ThemedSession ArtBookNext(double width, double height)
        {
            Assert.True(File.Exists(Path.Combine(ArtBookNextFactAttribute.Folder, "capabilities.xml")), "Art Book Next is needed for these pictures");
            string media = Path.Combine(Path.GetTempPath(), "EmuSenMenusFollowupMedia");
            SyntheticLibrary.WriteMedia(media);
            return new ThemedSession(width, height, a => { a.EsdeMediaDirectory = media; a.ControllerNotifications = false; }, themeDirectory: ArtBookNextFactAttribute.Folder);
        }

        [MenusFollowupPngFact]
        public Task Synthetic_theme() => Session.Dispatch(() =>
        {
            foreach ((int w, int h) in new[] { (1280, 800), (1920, 1200) })
            {
                using (var s = new ThemedSession(w, h, a => a.ControllerNotifications = false)) Walk(s, $"synthetic-{w}x{h}");
                using (var s = new ThemedSession(w, h, a => a.ControllerNotifications = false)) Families(s, $"synthetic-{w}x{h}");
            }
        }, default);

        [MenusFollowupPngFact]
        public Task Art_book_next() => Session.Dispatch(() =>
        {
            foreach ((int w, int h) in new[] { (1280, 800), (1920, 1200) })
            {
                using (ThemedSession s = ArtBookNext(w, h)) Walk(s, $"artbooknext-{w}x{h}");
                using (ThemedSession s = ArtBookNext(w, h)) Families(s, $"artbooknext-{w}x{h}");
            }
        }, default);

        // Every button of every family, filled and outlined, large: the set as drawn, for looking at.
        [MenusFollowupPngFact]
        public Task Glyph_sheet() => Session.Dispatch(() =>
        {
            PadGlyphButton[] buttons = Enum.GetValues<PadGlyphButton>();
            PadFamily[] families = Enum.GetValues<PadFamily>();
            const double side = 44, gap = 8;
            var canvas = new Canvas { Background = new SolidColorBrush(Color.FromRgb(0x16, 0x16, 0x18)) };
            int row = 0;
            foreach (PadGlyphStyle style in new[] { PadGlyphStyle.Filled, PadGlyphStyle.Outline })
                foreach (PadFamily family in families)
                {
                    for (int i = 0; i < buttons.Length; i++)
                    {
                        var glyph = new PadGlyph { Family = family, Button = buttons[i], Style = style, GlyphSize = side, Color = Color.FromRgb(0xD2, 0xD2, 0xD6) };
                        Canvas.SetLeft(glyph, gap + i * (side + gap));
                        Canvas.SetTop(glyph, gap + row * (side + gap));
                        canvas.Children.Add(glyph);
                    }
                    row++;
                }
            var window = new Window { Width = gap + buttons.Length * (side + gap), Height = gap + row * (side + gap), Content = canvas };
            window.Show();
            window.CaptureRenderedFrame()?.Dispose();
            using WriteableBitmap bitmap = window.CaptureRenderedFrame() ?? throw new InvalidOperationException("no frame");
            Directory.CreateDirectory(PngFolder);
            string path = Path.Combine(PngFolder, "glyph-sheet.png");
            bitmap.Save(path);
            _out.WriteLine(path);
            window.Close();
        }, default);
    }
}
