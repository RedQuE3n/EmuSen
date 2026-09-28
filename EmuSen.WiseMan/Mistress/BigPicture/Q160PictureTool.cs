using System;
using System.IO;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Headless;
using Avalonia.Input;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    public sealed class Q160PngFactAttribute : FactAttribute
    {
        public Q160PngFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_PNG") != "1")
                Skip = "Writes the swap's and the keyboard's pictures to ~/.cache/emusen/bigpicture/png/q160/; set EMUSEN_BIGPICTURE_PNG=1 - see EmuSen_BigPicture.md §40.19";
        }
    }

    // The help bars with the swap on, and the text popups, at 1280 by 800 and 1920 by 1200; EMUSEN_Q160_TAG names the build, so one build's pictures can sit beside another's.
    [Collection(TestCollections.ProcessGlobals)]
    public class Q160PictureTool
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(Q160PictureTool).GetTypeInfo().Assembly);

        public static readonly string PngFolder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture", "png", "q160");

        private static string Tag => Environment.GetEnvironmentVariable("EMUSEN_Q160_TAG") is { Length: > 0 } t ? t : "after";

        private readonly ITestOutputHelper _out;

        public Q160PictureTool(ITestOutputHelper output) => _out = output;

        private void Save(ThemedSession s, string name)
        {
            s.Settle();
            Directory.CreateDirectory(PngFolder);
            string path = Path.Combine(PngFolder, $"{Tag}-{name}.png");
            s.Capture().SavePng(path);
            _out.WriteLine(path);
        }

        // With the swap on, East is Accept: into a system with B, and a menu entry chosen with B.
        private void Swapped(ThemedSession s, string size)
        {
            s.Run(500);
            Save(s, $"{size}-system-swapped");
            for (int guard = 0; guard < 6 && s.System != "snes"; guard++) s.Pad.Right();
            s.Pad.B();
            s.Run(500);
            Save(s, $"{size}-gamelist-swapped");

            ThemedGameOptionsTests.OpenOptions(s);
            PadAudit.Reach(ThemedGameOptionsTests.Sheet(s), s.Pad, e => e is Avalonia.Controls.Button { Content: "Edit This Game's Metadata" });
            s.Pad.B();
            s.Settle();
            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.Completed);
            s.Pad.B();
            Save(s, $"{size}-editor-swapped");
            for (int guard = 0; guard < 4 && ThemedGameOptionsTests.Sheets(s).IsPresenting; guard++)
            {
                s.Pad.A();
                if (ThemedGameOptionsTests.Sheets(s).Current?.GetType().Name == "DialogWindow")
                {
                    PadAudit.Reach(ThemedGameOptionsTests.Sheet(s), s.Pad, e => e is Avalonia.Controls.Button { Content: "Discard" });
                    s.Pad.B();
                }
            }

            typeof(MainWindow).GetMethod("ShowPreferences", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(s.Window, null);
            s.Settle();
            ThemedGameOptionsTests.Reach(s, "BigMenuPage4");
            s.Pad.B();
            ThemedGameOptionsTests.Reach(s, "SwapPadButtonsSwitch");
            Save(s, $"{size}-controllers-swapped");
            for (int guard = 0; guard < 4 && ThemedGameOptionsTests.Sheets(s).IsPresenting; guard++) s.Pad.A();
        }

        // The text popups: from a keyboard, centred; with Steam's keyboard asked for, in the upper part of the screen; and the themed search's.
        private void Popups(ThemedSession s, string size)
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
            s.Settings.OnScreenKeyboard = AppSettings.OnScreenKeyboardSteam;
            s.Pad.A();
            s.Window.KeyTextInput(" Deluxe");
            Save(s, $"{size}-editor-steam");
            s.Pad.B();
            s.Settings.OnScreenKeyboard = AppSettings.OnScreenKeyboardAutomatic;
            s.Pad.Unplug();
            s.Pad.Tick();
            EsdeMenusTests.Press(s, Key.Enter);
            s.Window.KeyTextInput(" Deluxe");
            Save(s, $"{size}-editor-field");
            EsdeMenusTests.Press(s, Key.Escape);
            s.Pad.Replug();
            s.Pad.Tick();
            ThemedSwitchesTests.PutAway(s);

            ThemedGameOptionsTests.OpenOptions(s);
            PadAudit.Reach(ThemedGameOptionsTests.Sheet(s), s.Pad, e => e is Avalonia.Controls.Button { Content: "Search..." });
            s.Pad.Unplug();
            s.Pad.Tick();
            EsdeMenusTests.Press(s, Key.Enter);
            s.Window.KeyTextInput("cob");
            Save(s, $"{size}-search-field");
            EsdeMenusTests.Press(s, Key.Enter);
            Save(s, $"{size}-search-filtered");
            _ = editor;
        }

        [Q160PngFact]
        public Task Pictures() => Session.Dispatch(() =>
        {
            foreach ((int w, int h) in new[] { (1280, 800), (1920, 1200) })
            {
                using (var s = ScreensaverTests.Open(i => i.ScreensaverTimer = 0, w, h, a => { a.SwapPadButtons = true; a.ControllerType = "Xbox"; a.ControllerNotifications = false; }))
                    Swapped(s, $"{w}x{h}");
                using (var s = new ThemedSession(w, h, a => a.ControllerNotifications = false))
                    Popups(s, $"{w}x{h}");
            }
        }, default);
    }
}
