using System;
using System.Threading.Tasks;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // A theory that runs only when the big-picture pictures are asked for, as WindowLookPngFact does for a fact.
    public sealed class WindowLookPngTheoryAttribute : TheoryAttribute
    {
        public WindowLookPngTheoryAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_PNG") != "1")
                Skip = "Pictures the tool screens at the mockups' sizes under EMUSEN_WINDOW_FIT_PNG; set EMUSEN_BIGPICTURE_PNG=1 - see EmuSen_Interface.md §14.2";
        }
    }

    // The four tools as screens at 1280 by 800 and 1920 by 1080, the mockups' sizes, audited and pictured beside them - see EmuSen_Interface.md §14.2.
    public partial class WindowFitAuditTests
    {
        public static TheoryData<string, int, int> ToolScreenCases()
        {
            var data = new TheoryData<string, int, int>();
            foreach (string which in new[] { "Cores", "CoresDetail", "Packages", "ScraperDetail", "LibraryDetail", "LibraryPlan" })
                foreach ((int w, int h) in new[] { (1280, 800), (1920, 1080) }) data.Add(which, w, h);
            return data;
        }

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
