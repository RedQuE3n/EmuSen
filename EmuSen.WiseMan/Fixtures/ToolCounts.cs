using System;
using System.Reflection;
using System.Threading.Tasks;
using EmuSen.Mistress.Views;

namespace EmuSen.WiseMan.Fixtures
{
    // The Home card's tool counts set for a test, through the window's seams: what the scraper and the runtime answer, asked in place - see EmuSen_BigPicture.md §49.4.
    public static class ToolCounts
    {
        private static FieldInfo Seam(string name) => typeof(MainWindow).GetField(name, BindingFlags.Static | BindingFlags.NonPublic)!;

        public static void Answer(int? missingMedia, (int Listed, int Waiting)? cores)
        {
            Seam("MissingMediaQuery").SetValue(null, (Func<int?>)(() => missingMedia));
            Seam("CoresQuery").SetValue(null, (Func<(int, int)?>)(() => cores));
            Seam("ToolCountsBackground").SetValue(null, (Func<Action, Task>)(work => { work(); return Task.CompletedTask; }));
        }

        public static void Quiet() => Answer(null, null);
    }
}
