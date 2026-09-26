using System;
using System.IO;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    public sealed class DesktopButtonPngFactAttribute : FactAttribute
    {
        public DesktopButtonPngFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_PNG") != "1")
                Skip = "Writes the desktop button's pictures to ~/.cache/emusen/bigpicture/png/desktop-button/; set EMUSEN_BIGPICTURE_PNG=1 - see EmuSen_Settings_Reference.md §4.54";
        }
    }

    // The desktop window with its Fullscreen button, big picture after it, and the desktop after leaving, at 1280 by 800; written outside the repository.
    [Collection(TestCollections.ProcessGlobals)]
    public class DesktopButtonPictureTool
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(DesktopButtonPictureTool).GetTypeInfo().Assembly);

        public static readonly string PngFolder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture", "png", "desktop-button");

        private static readonly string MediaRoot = Path.Combine(Path.GetTempPath(), "EmuSenDesktopButtonMedia");

        private readonly ITestOutputHelper _out;

        public DesktopButtonPictureTool(ITestOutputHelper output) => _out = output;

        private void Save(RenderedFrame frame, string name)
        {
            string path = Path.Combine(PngFolder, name + ".png");
            frame.SavePng(path);
            _out.WriteLine(path);
        }

        [DesktopButtonPngFact]
        public Task Desktop_button_pictures() => Session.Dispatch(() =>
        {
            Directory.CreateDirectory(PngFolder);
            bool artBook = File.Exists(Path.Combine(ArtBookNextFactAttribute.Folder, "capabilities.xml"));
            if (artBook) SyntheticLibrary.WriteMedia(MediaRoot);

            foreach (string style in new[] { "synthetic", "artbooknext", "mistress" })
            {
                if (style == "artbooknext" && !artBook) continue;
                using var s = new ThemedSession(1280, 800, a =>
                {
                    a.BigScreen = false;
                    a.LibraryView = AppSettings.LibraryGrid;
                    if (style == "mistress") a.BigPictureTheme = null;
                    if (style == "artbooknext") a.EsdeMediaDirectory = MediaRoot;
                }, themeDirectory: style == "artbooknext" ? ArtBookNextFactAttribute.Folder : null);

                if (style == "synthetic")
                {
                    Save(s.Capture(), "desktop-1280x800");
                    ActionButton button = BigPictureSwitchTests.Button(s.Window);
                    ToolTip.SetIsOpen(button, true);
                    s.Settle();
                    Save(s.Capture(), "desktop-tooltip-1280x800");
                    ToolTip.SetIsOpen(button, false);
                }

                BigPictureSwitchTests.Click(s.Window, BigPictureSwitchTests.Button(s.Window));
                s.Run(500);
                Save(s.Capture(), $"{style}-bigpicture-1280x800");
                s.Pad.Start();
                Save(s.Capture(), $"{style}-padmenu-1280x800");
                s.Pad.B();

                BigPictureSwitchTests.Press(s, Key.F11);
                Save(s.Capture(), $"{style}-desktop-after-1280x800");
            }
        }, default);
    }
}
