using System.Collections.Generic;
using System.Threading.Tasks;
using Avalonia.Controls;
using EmuSen.LunaP.Controls;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.LunaP
{
    // The LunaList promises the frontends' lists were migrated onto, measured
    // rather than read off the summaries - see EmuSen_Settings_Reference.md §4.11a.
    // Every one of these is a silent behaviour change if it turns out otherwise,
    // not a compile error, which is why they are pinned here.
    public class LunaListContractTests
    {
        private sealed record Row(string Id, string Text);

        private static LunaList<Row> Built(out List<Row> chosen)
        {
            var picked = new List<Row>();
            var list = new LunaList<Row> { Label = r => r.Text, Key = r => r.Id };
            list.Chose += r => { if (r is not null) picked.Add(r); };
            chosen = picked;
            return list;
        }

        private static Window Showing(Control content)
        {
            var window = new Window { Content = content };
            window.Show();
            return window;
        }

        // Refresh selects nothing, so a window wanting a default must say so.
        [Fact]
        public Task Refresh_leaves_nothing_selected() => UiTest.Run(() =>
        {
            var list = Built(out List<Row> chosen);
            var window = Showing(list);

            list.Refresh(new[] { new Row("a", "Alpha"), new Row("b", "Beta") });

            Assert.Null(list.Selected);
            Assert.Equal(-1, list.SelectedIndex);
            Assert.Empty(chosen);

            window.Close();
        });

        // Chose is SELECTION, not activation. A window that closes or launches on
        // Chose does it on one click - which is not what a double-click list did.
        [Fact]
        public Task Chose_fires_on_a_selection_change() => UiTest.Run(() =>
        {
            var list = Built(out List<Row> chosen);
            var window = Showing(list);
            list.Refresh(new[] { new Row("a", "Alpha"), new Row("b", "Beta") });

            list.SelectedIndex = 1;

            Assert.Equal("b", Assert.Single(chosen).Id);

            window.Close();
        });

        // Select() states what is true without pretending the user did it, which
        // is the seam a default selection has to go through.
        [Fact]
        public Task Select_sets_the_selection_without_raising_Chose() => UiTest.Run(() =>
        {
            var list = Built(out List<Row> chosen);
            var window = Showing(list);
            list.Refresh(new[] { new Row("a", "Alpha"), new Row("b", "Beta") });

            list.Select(new Row("a", "Alpha"));

            Assert.Equal("a", list.Selected!.Id);
            Assert.Empty(chosen);

            window.Close();
        });

        // Refresh restores by Key across rebuilt objects, and stays quiet doing it,
        // so a rescan cannot look like a click.
        [Fact]
        public Task Refresh_restores_the_selection_by_key_and_stays_quiet() => UiTest.Run(() =>
        {
            var list = Built(out List<Row> chosen);
            var window = Showing(list);
            list.Refresh(new[] { new Row("a", "Alpha"), new Row("b", "Beta") });
            list.SelectedIndex = 1;
            chosen.Clear();

            list.Refresh(new[] { new Row("a", "Alpha"), new Row("b", "Beta") });

            Assert.Equal("b", list.Selected!.Id);
            Assert.Empty(chosen);

            window.Close();
        });

        // And drops it when the key is gone - the case a narrowing filter produces.
        [Fact]
        public Task Refresh_drops_a_selection_the_new_list_no_longer_holds() => UiTest.Run(() =>
        {
            var list = Built(out List<Row> chosen);
            var window = Showing(list);
            list.Refresh(new[] { new Row("a", "Alpha"), new Row("b", "Beta") });
            list.SelectedIndex = 1;
            chosen.Clear();

            list.Refresh(new[] { new Row("a", "Alpha") });

            Assert.Null(list.Selected);
            Assert.Empty(chosen);

            window.Close();
        });
    }
}
