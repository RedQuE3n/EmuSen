using System.Collections.Generic;
using System.Threading.Tasks;
using Avalonia.Controls;
using EmuSen.LunaP.Controls;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.LunaP
{
    // LunaList's measured contract - each a silent behaviour change if wrong. §4.11a.
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

        // Chose is SELECTION, not activation - §4.11a.
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

        // The seam a default selection goes through, without claiming the user did it.
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

        // Restores by Key across rebuilt objects, quietly, so a rescan is not a click.
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
