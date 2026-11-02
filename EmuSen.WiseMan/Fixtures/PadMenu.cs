using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using Avalonia.Controls;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Views;
using Xunit;

namespace EmuSen.WiseMan.Fixtures
{
    // The pad menu read and walked as a player walks it: a row on the page shown, or one submenu down, reached with Down and A - see EmuSen_Settings_Reference.md §4.69.8.
    public static class PadMenu
    {
        private const BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;

        public static List<PadMenuEntry> Entries(MainWindow w) => (List<PadMenuEntry>)typeof(MainWindow).GetField("_padMenuEntries", Hidden)!.GetValue(w)!;

        // The rows of the page shown, as their text reads.
        public static string[] Lines(MainWindow w) => Entries(w).Select(e => e.Text()).ToArray();

        // Every row the page shown offers: its own, then each of its submenus' rows.
        public static string[] AllLines(MainWindow w) => All(Entries(w)).ToArray();

        private static IEnumerable<string> All(List<PadMenuEntry> rows) =>
            rows.SelectMany(e => e.Submenu is { } sub ? new[] { e.Text() }.Concat(All(sub())) : [e.Text()]);

        public static bool IsOpen(MainWindow w) => w.GetControl<Control>("PadMenuPanel").IsVisible;

        public static ListBox List(MainWindow w) => w.GetControl<ListBox>("PadMenuList");

        public static int Selected(MainWindow w) => List(w).SelectedIndex;

        public static string SelectedLine(MainWindow w) => Lines(w)[Selected(w)];

        // The card's panel: the Pause Card's over a game, the Home card's over the library - see EmuSen_BigPicture.md §48 and §49.
        public static MenuPanel Big(MainWindow w) => Card(w).CardPanel;

        public static AdaptiveLayout Card(MainWindow w) => w.GetControl<AdaptiveLayout>("PadCard");

        public static PauseCardPane Pane(MainWindow w) => (PauseCardPane)Card(w).Detail!;

        // The desktop's title line: the pages' titles joined into a breadcrumb.
        public static string DeskTitle(MainWindow w) => w.GetControl<TextBlock>("PadMenuTitle").Text ?? "";

        // How many pages deep the menu is: 1 on its first page.
        public static int Depth(MainWindow w) => ((System.Collections.ICollection)typeof(MainWindow).GetField("_padMenuPages", Hidden)!.GetValue(w)!).Count;

        private static bool Matches(PadMenuEntry e, string entry, bool exact) =>
            exact ? e.Text() == entry : e.Text().StartsWith(entry, StringComparison.Ordinal);

        // The rows to choose to reach a row whose text is or starts with the words: on this page, else the shallowest submenu under it holding one.
        public static int[]? PathTo(MainWindow w, string entry, bool exact = false) => PathIn(Entries(w), entry, exact, 3);

        private static int[]? PathIn(List<PadMenuEntry> rows, string entry, bool exact, int depth)
        {
            int here = rows.FindIndex(e => Matches(e, entry, exact));
            if (here >= 0) return [here];
            if (depth <= 1) return null;
            for (int d = 2; d <= depth; d++)
                for (int i = 0; i < rows.Count; i++)
                    if (rows[i].Submenu is { } sub && PathIn(sub(), entry, exact, d - 1) is { } below) return [i, .. below];
            return null;
        }

        // Every row reachable from the page shown, with the submenu each is in, for a failure's message.
        public static string Describe(MainWindow w) => Describe(Entries(w));

        private static string Describe(List<PadMenuEntry> rows) =>
            string.Join(", ", rows.Select(e => e.Submenu is { } sub ? $"{e.Text()} ▸ [{Describe(sub())}]" : e.Text()));

        // Down from the row the selection is on to each row of the path, then A, as a player would.
        public static void Choose(MainWindow w, PadDriver pad, string entry, bool exact = false)
        {
            Assert.True(IsOpen(w), $"the pad menu is not open to choose '{entry}'");
            int[]? path = PathTo(w, entry, exact);
            Assert.True(path is not null, $"No '{entry}' in the pad menu: {Describe(w)}");
            foreach (int at in path!)
            {
                int count = Entries(w).Count;
                pad.Down((at - Math.Max(Selected(w), 0) + count) % count);
                pad.A();
            }
        }

        // The yes-or-no answered: Down to Yes, or A on No where the selection starts.
        public static void Answer(MainWindow w, PadDriver pad, bool yes)
        {
            Assert.Equal([MainWindow.QuestionNo, MainWindow.QuestionYes], Lines(w));
            Assert.Equal(0, Selected(w));
            if (yes) pad.Down();
            pad.A();
        }
    }
}
