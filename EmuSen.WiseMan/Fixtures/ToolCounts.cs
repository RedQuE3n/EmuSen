using System;
using System.Collections.Generic;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using EmuSen.Galaxia.Native;
using EmuSen.Mistress.Pages;
using EmuSen.Mistress.Views;

namespace EmuSen.WiseMan.Fixtures
{
    // The tools' one-line states set for a test through the window's seams, read in place of each tool's own read - see EmuSen_Interface.md §19.
    public static class ToolCounts
    {
        private static FieldInfo Seam(string name) => typeof(MainWindow).GetField(name, BindingFlags.Static | BindingFlags.NonPublic)!;

        // The mockups' lines: 41 games without art, 3 packages changed, 2 suspect dumps, 1 core to approve, 12 cheats on in 3 games.
        public static readonly IReadOnlyDictionary<string, ToolSummary?> Mockups = new Dictionary<string, ToolSummary?>
        {
            [HomeScreen.Scraper] = new("41 without art", "41 no art", Sentence: "41 games have no art yet; the Scraper fetches it from ScreenScraper."),
            [HomeScreen.Packages] = new("3 changed", Sentence: "3 packages no longer match their records: each moved, went missing or changed."),
            [HomeScreen.Library] = new("2 suspect dumps", "2 suspect", NeedsAttention: true, Sentence: "2 files are suspect dumps: each matches no known dump, or a part of it is missing."),
            [HomeScreen.Cores] = new("1 to approve", NeedsAttention: true, Sentence: "PCEE2, a PlayStation 2 core from libretro's buildbot, waits for your approval. Nothing downloaded runs until you approve it."),
            [HomeScreen.Cheats] = new("12 on · 3 games", Sentence: "12 cheats are on, in 3 games."),
        };

        public static void Answer(IReadOnlyDictionary<string, ToolSummary?> read)
        {
            Seam("ToolSummariesQuery").SetValue(null, (Func<IReadOnlyCollection<string>, CancellationToken, IReadOnlyDictionary<string, ToolSummary?>>)((_, _) => read));
            Seam("ToolSummariesBackground").SetValue(null, (Func<Action, Task>)(work => { work(); return Task.CompletedTask; }));
        }

        public static void Answer() => Answer(Mockups);

        public static void Quiet() => Answer(new Dictionary<string, ToolSummary?>());
    }
}
