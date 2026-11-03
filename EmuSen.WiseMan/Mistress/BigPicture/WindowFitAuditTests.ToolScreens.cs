using System;
using System.Linq;
using System.Threading.Tasks;
using EmuSen.Mistress.Views;
using Avalonia.Controls;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // A theory that runs only when the big-picture pictures are asked for, as WindowLookPngFact does for a fact.
    public sealed class WindowLookPngTheoryAttribute : TheoryAttribute
    {
        public WindowLookPngTheoryAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_PNG") != "1")
                Skip = "Pictures the tool screens at the mockups' sizes under EMUSEN_WINDOW_FIT_PNG; set EMUSEN_BIGPICTURE_PNG=1 - see EmuSen_Interface.md §15.2";
        }
    }

    // The four tools as screens at 1280 by 800 and 1920 by 1080, the mockups' sizes, audited and pictured beside them - see EmuSen_Interface.md §15.2.
    public partial class WindowFitAuditTests
    {
        public static TheoryData<string, int, int> ToolScreenCases()
        {
            var data = new TheoryData<string, int, int>();
            foreach (string which in new[] { "Cores", "CoresDetail", "Packages", "ScraperDetail", "ScraperPlan", "LibraryDetail", "LibraryPlan" })
                foreach ((int w, int h) in new[] { (1280, 800), (1920, 1080) }) data.Add(which, w, h);
            return data;
        }

        // The form the window shows its tools in.
        private static EmuSen.LunaP.Controls.AdaptiveForm FormOf(MainWindow window) => Field(window, "_bigScreen") is true ? EmuSen.LunaP.Controls.AdaptiveForm.BigScreen : EmuSen.LunaP.Controls.AdaptiveForm.Desktop;

        // The tool the window showed: the big screen's sheet, or on the desktop the window it owns.
        private static T Shown<T>(MainWindow window) where T : Window =>
            Sheets(window).Current as T ?? window.OwnedWindows.OfType<T>().Last();

        public static TheoryData<string> DesktopToolCases() => new() { "Cores", "Packages", "Scraper", "Library", "CheatDatabase" };

        // The tools on the desktop at the mockups' 1280 by 800, each its own window, pictured beside the desktop mockups (EmuSen_Interface.md §19).
        [WindowLookPngTheory]
        [MemberData(nameof(DesktopToolCases))]
        public Task A_tool_on_the_desktop_is_pictured_at_the_mockups_size(string which) => Session.Dispatch(() =>
        {
            ToolCounts.Answer();
            (MainWindow window, PadDriver pad) = InGame(1280, 800, bigScreen: false);
            try
            {
                if (which == "CheatDatabase")
                {
                    CheatFiles();
                    Call(window, "ShowCheatDatabase");
                }
                else OpenInGame(window, pad, which);
                Settle(window);
                Window tool = Shown<Window>(window);
                tool.Width = 1280;
                tool.Height = 800;
                Settle(window);
                Settle(tool);
                SavePicture(tool, $"{which}Desktop-1280x800");
                tool.Close();
            }
            finally
            {
                ToolCounts.Quiet();
                Stop(window);
                window.Close();
            }
        }, default);

        [WindowLookPngTheory]
        [MemberData(nameof(ToolScreenCases))]
        public Task A_tool_screen_fits_and_is_pictured_at_the_mockups_sizes(string which, int width, int height) => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = InGame(width, height);
            try
            {
                OpenInGame(window, pad, which);
                Assert.Empty(Audit(window, $"{which}-{width}x{height}", _out));
            }
            finally
            {
                Stop(window);
                window.Close();
            }
        }, default);
    }
}
