using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Media;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    public sealed class Pass4PngFactAttribute : FactAttribute
    {
        public Pass4PngFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_PNG") != "1")
                Skip = "Writes pass 4's pictures to ~/.cache/emusen/bigpicture/png/pass4/; set EMUSEN_BIGPICTURE_PNG=1 - see EmuSen_BigPicture.md §29";
        }
    }

    // Pass 4's badges, switches, overlay and sheets at 1280 by 800, written outside the repository to be looked at - see EmuSen_BigPicture.md §29.
    [Collection(TestCollections.ProcessGlobals)]
    public class Pass4PictureTool
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(Pass4PictureTool).GetTypeInfo().Assembly);

        public static readonly string PngFolder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture", "png", "pass4");

        private readonly ITestOutputHelper _out;

        public Pass4PictureTool(ITestOutputHelper output) => _out = output;

        private void Save(RenderedFrame frame, string name)
        {
            Directory.CreateDirectory(PngFolder);
            string path = Path.Combine(PngFolder, name + ".png");
            frame.SavePng(path);
            _out.WriteLine(path);
        }

        private static RenderedFrame Whole(Window window)
        {
            Dispatcher.UIThread.RunJobs();
            window.UpdateLayout();
            window.CaptureRenderedFrame()?.Dispose();
            foreach (Visual v in window.GetSelfAndVisualDescendants()) v.InvalidateVisual();
            return UiTest.Capture(window);
        }

        private static void Place(Canvas canvas, Control c, double x, double y)
        {
            Canvas.SetLeft(c, x);
            Canvas.SetTop(c, y);
            canvas.Children.Add(c);
        }

        // Every badge and controller the toolkit draws, large and at the size a theme's strip gives them, on dark and on light.
        [Pass4PngFact]
        public Task Glyph_sheet() => Session.Dispatch(() =>
        {
            var canvas = new Canvas { Width = 1280, Height = 800, Background = new SolidColorBrush(Color.FromRgb(0x18, 0x1C, 0x24)) };
            var light = new Border { Width = 1280, Height = 200, Background = new SolidColorBrush(Color.FromRgb(0xEE, 0xEA, 0xE0)) };
            Place(canvas, light, 0, 600);
            BadgeKind[] kinds = Enum.GetValues<BadgeKind>();
            for (int i = 0; i < kinds.Length; i++)
            {
                Place(canvas, new BadgeGlyph { Kind = kinds[i], GlyphSize = 104, Color = Colors.White }, 20 + i * 124, 30);
                Place(canvas, new TextBlock { Text = kinds[i].ToString(), Foreground = Brushes.White, FontSize = 14 }, 24 + i * 124, 140);
                Place(canvas, new BadgeGlyph { Kind = kinds[i], GlyphSize = 36, Color = Colors.White }, 54 + i * 124, 170);
                Place(canvas, new BadgeGlyph { Kind = kinds[i], GlyphSize = 48, Color = Color.FromRgb(0x20, 0x20, 0x28) }, 48 + i * 124, 640);
            }
            ControllerShape[] shapes = Enum.GetValues<ControllerShape>();
            for (int i = 0; i < shapes.Length; i++)
            {
                Place(canvas, new ControllerGlyph { Shape = shapes[i], GlyphSize = 170, Color = Colors.White }, 30 + i * 250, 240);
                Place(canvas, new TextBlock { Text = shapes[i].ToString(), Foreground = Brushes.White, FontSize = 14 }, 40 + i * 250, 420);
                Place(canvas, new ControllerGlyph { Shape = shapes[i], GlyphSize = 60, Color = Color.FromRgb(0x20, 0x20, 0x28) }, 80 + i * 250, 720);
            }
            Place(canvas, new BadgeStrip
            {
                Width = 700, Height = 110, ItemsPerLine = 6, ItemMargin = new Size(10, 0),
                Entries = [new BadgeEntry(BadgeKind.Favorite), new BadgeEntry(BadgeKind.Controller) { Controller = ControllerShape.Snes }, new BadgeEntry(BadgeKind.Controller) { Controller = ControllerShape.Nintendo64 },
                    new BadgeEntry(BadgeKind.Folder) { Linked = true }, new BadgeEntry(BadgeKind.Folder), new BadgeEntry(BadgeKind.Manual)],
            }, 20, 460);
            Place(canvas, new ScrollLetterOverlay { Width = 300, Height = 110, Letters = "Co", LetterSize = 80, Foreground = Colors.White }, 900, 460);
            var window = new Window { Width = 1280, Height = 800, Content = canvas };
            window.Show();
            Save(Whole(window), "glyph-sheet");
            window.Close();
        }, default);

        // The synthetic theme with every switch it has an element for, and the badges drawn by Mistress for a theme that names no image.
        [Pass4PngFact]
        public Task Synthetic_theme_switches() => Session.Dispatch(() =>
        {
            const string badges = "<badges name=\"badges\"><pos>0.55 0.82</pos><size>0.4 0.08</size><slots>all</slots><lines>1</lines><itemsPerLine>9</itemsPerLine><itemMargin>0.005 0</itemMargin></badges>";
            using var s = new ThemedSession(extraGamelist: ThemedSwitchesTests.Clock + ThemedSwitchesTests.Status + badges, status: ThemedSwitchesTests.Device);
            s.Themed.LiveClock = false;
            s.Themed.Now = () => new DateTime(2026, 9, 27, 13, 45, 0);
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.Y();
            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
            ((Dropdown)editor.EditorOf(GameMetadata.Controller)).SelectedItem = "Super Nintendo";
            foreach (string flag in new[] { GameMetadata.Completed, GameMetadata.KidGame, GameMetadata.Broken })
                ((LunaSwitch)editor.EditorOf(flag)).IsChecked = true;
            ThemedGameOptionsTests.Reach(s, "MetadataSave");
            s.Pad.A();
            s.Settle();
            Save(s.Capture(), "synthetic-default-badges-clock-off");

            ThemedSwitchesTests.Flip(s, "DisplayClock");
            Save(s.Capture(), "synthetic-clock-on");
            ThemedSwitchesTests.Flip(s, "StatusWifi");
            Save(s.Capture(), "synthetic-status-wifi-off");
            ThemedSwitchesTests.Flip(s, "StatusBatteryPercentage");
            Save(s.Capture(), "synthetic-status-wifi-and-percentage-off");
            ThemedSwitchesTests.Flip(s, "DisplayHelp");
            Save(s.Capture(), "synthetic-help-off");
            ThemedSwitchesTests.Flip(s, "DisplayHelp");
            ThemedSwitchesTests.Choose(s, "QuickSystemSelect", "Shoulders");
            Save(s.Capture(), "synthetic-quick-select-shoulders-help");

            ThemedSwitchesTests.OpenInterface(s);
            Save(s.Capture(), "sheet-interface-tab");
            s.Pad.B();
            ThemedLibraryFlowTests.Choose(s, "Preferences");
            Window prefs = s.Window.GetControl<SheetLayer>("Sheets").Current!;
            TabControl tabs = ThemedSwitchesTests.Named<TabControl>(prefs, "PreferenceTabs");
            while ((tabs.SelectedItem as TabItem)?.Header as string != "Appearance") s.Pad.R1();
            ThemedSwitchesTests.Named<Slider>(prefs, "NavigationVolumeSlider").BringIntoView();
            Save(s.Capture(), "sheet-preferences-volume");
            s.Pad.B();
        }, default);

        // The quick scrolling overlay over a held list, letters and then a star, on the synthetic theme and on Art Book Next.
        [Pass4PngFact]
        public Task Scroll_overlay() => Session.Dispatch(() =>
        {
            foreach ((string name, string? theme) in new[] { ("synthetic", (string?)null), ("artbooknext", File.Exists(Path.Combine(ArtBookNextFactAttribute.Folder, "capabilities.xml")) ? ArtBookNextFactAttribute.Folder : null) })
            {
                if (name == "artbooknext" && theme is null) continue;
                using var s = new ThemedSession(themeDirectory: theme, status: ThemedSwitchesTests.Device, settings: a => a.BigPictureInterface.ListScrollOverlay = true);
                ThemedLibraryPadTests.Enter(s, "snes");
                s.Pad.Y();
                s.Pad.L2();
                s.Pad.Pad.Press(SDL.GamepadButton.DPadDown);
                s.Pad.Tick();
                s.Run(560);
                Save(s.Capture(), $"{name}-overlay-held");
                s.Pad.Pad.Release(SDL.GamepadButton.DPadDown);
                s.Pad.Tick();
                s.Pad.L2();
                for (int i = 0; i < 3; i++)
                {
                    s.Pad.Down();
                    if (!s.Themed.SelectedGame!.Favorite) s.Pad.Y();
                }
                s.Pad.L2();
                s.Pad.Pad.Press(SDL.GamepadButton.DPadDown);
                s.Pad.Tick();
                s.Run(510);
                Save(s.Capture(), $"{name}-overlay-star");
                s.Pad.Pad.Release(SDL.GamepadButton.DPadDown);
                s.Pad.Tick();
            }
        }, default);

        // Art Book Next with its own badge images, which Mistress's drawings leave alone, and its clock turned on.
        [ArtBookNextFact]
        public Task Art_book_next_badges_and_clock() => Session.Dispatch(() =>
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_PNG") != "1") return;
            using var s = new ThemedSession(themeDirectory: ArtBookNextFactAttribute.Folder, status: ThemedSwitchesTests.Device);
            s.Themed.LiveClock = false;
            s.Themed.Now = () => new DateTime(2026, 9, 27, 13, 45, 0);
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.Y();
            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
            ((LunaSwitch)editor.EditorOf(GameMetadata.Completed)).IsChecked = true;
            ThemedGameOptionsTests.Reach(s, "MetadataSave");
            s.Pad.A();
            s.Settle();
            Save(s.Capture(), "artbooknext-badges");
            s.Pad.B();
            Save(s.Capture(), "artbooknext-system-clock-off");
            ThemedSwitchesTests.Flip(s, "DisplayClock");
            Save(s.Capture(), "artbooknext-system-clock-on");
        }, default);
    }
}
