using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using EmuSen.LunaP.Controls;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.LunaP
{
    // Dropdowns, switches, tabs and filter bars - see EmuSen_LunaP.md §14.
    public class WidgetTests
    {
        private static Task Realised<T>(Func<T> make, Action<T> assert) where T : Control => UiTest.Run(() =>
        {
            T control = make();
            var window = new Window { Width = 500, Height = 300, Content = control };
            window.Show();
            assert(control);
            window.Close();
        });

        [Fact]
        public Task A_switch_shows_its_label_and_reports_its_state() =>
            Realised(() => new LunaSwitch { Label = "Enable Logging", IsChecked = true }, toggle =>
            {
                // Beside the knob and identical in both states, which is the line a CheckBox drew - see EmuSen_LunaP.md §14.1.
                Assert.Equal("Enable Logging", toggle.OnContent);
                Assert.Equal("Enable Logging", toggle.OffContent);
                Assert.Null(toggle.Content);
                Assert.True(toggle.IsChecked);

                toggle.IsChecked = false;
                Assert.False(toggle.IsChecked);
            });

        // The reason Fill exists: setting ItemsSource then SelectedItem raises SelectionChanged, and a naive
        // wiring would treat that as the user picking something and write it straight back to config.
        [Fact]
        public Task Filling_a_dropdown_does_not_look_like_a_user_choice() =>
            Realised(() => new Dropdown(), drop =>
            {
                var chosen = new List<object?>();
                drop.Chose += v => chosen.Add(v);

                drop.Fill(new[] { "All consoles", "NES", "SNES" }, "SNES");
                Assert.Empty(chosen);
                Assert.Equal("SNES", drop.SelectedItem);

                drop.SelectedItem = "NES";
                Assert.Equal(new object?[] { "NES" }, chosen);
            });

        [Fact]
        public Task Tabs_append_and_truncate() =>
            Realised(() => new Tabs(), tabs =>
            {
                tabs.Add("General", new TextBlock { Text = "general" });
                tabs.Add("NES", new TextBlock { Text = "nes" });
                tabs.Add("SNES", new TextBlock { Text = "snes" });
                Assert.Equal(3, tabs.Items.Count);

                // What a console-set change does: keep the declared tabs, drop the generated ones.
                tabs.RemoveFrom(1);
                Assert.Single(tabs.Items);
                Assert.Equal("General", ((TabItem)tabs.Items[0]!).Header);
            });

        [Fact]
        public Task A_filter_bar_reports_typing() =>
            Realised(() => new FilterBar { Placeholder = "Search titles" }, bar =>
            {
                int changes = 0;
                bar.Changed += () => changes++;

                TextBox search = bar.FindPart<TextBox>()!;
                search.Text = "zel";

                Assert.Equal(1, changes);
                Assert.Equal("zel", bar.SearchText);
            });

        // The hard-won detail both hand-written copies carried: a Text set from code has to count too,
        // which TextChanged would miss and the property change does not.
        [Fact]
        public Task A_filter_bar_reports_a_search_set_from_code() =>
            Realised(() => new FilterBar(), bar =>
            {
                int changes = 0;
                bar.Changed += () => changes++;

                bar.FindPart<TextBox>()!.Text = "typed";
                Assert.Equal(1, changes);

                // Clearing it programmatically is exactly the case a TextChanged handler would drop.
                bar.FindPart<TextBox>()!.Text = "";
                Assert.Equal(2, changes);
                Assert.Equal("", bar.SearchText);
            });

        [Fact]
        public Task A_filter_bar_hides_its_facet_until_asked() =>
            Realised(() => new FilterBar(), bar =>
            {
                Assert.False(bar.FindPart<Dropdown>()!.IsVisible);

                bar.ShowFacet = true;
                bar.FacetLabel = "Console:";
                Assert.True(bar.FindPart<Dropdown>()!.IsVisible);
            });

        // Facets are set from a caller's constructor, before any template exists.
        [Fact]
        public Task Facets_set_before_the_template_still_arrive() => UiTest.Run(() =>
        {
            var bar = new FilterBar { ShowFacet = true };
            bar.SetFacets(new[] { "All consoles", "SNES" }, "SNES");

            var window = new Window { Width = 500, Height = 300, Content = bar };
            window.Show();

            Assert.Equal("SNES", bar.Facet);
            Assert.Equal("SNES", bar.FindPart<Dropdown>()!.SelectedItem);
            window.Close();
        });

        [Fact]
        public Task Choosing_a_facet_reports_a_change_but_filling_does_not() =>
            Realised(() => new FilterBar { ShowFacet = true }, bar =>
            {
                int changes = 0;
                bar.SetFacets(new[] { "All consoles", "NES", "SNES" }, "All consoles");
                bar.Changed += () => changes++;

                bar.FindPart<Dropdown>()!.SelectedItem = "NES";

                Assert.Equal(1, changes);
                Assert.Equal("NES", bar.Facet);
            });

        [Theory]
        [InlineData("", "Zelda", true)]
        [InlineData("   ", "Zelda", true)]
        [InlineData("zel", "Zelda", true)]
        [InlineData("ZEL", "Zelda", true)]
        [InlineData(" zel ", "Zelda", true)]
        [InlineData("link", "Zelda", false)]
        public void The_match_is_case_insensitive_and_empty_matches_everything(string search, string candidate, bool expected) =>
            Assert.Equal(expected, FilterBar.Matches(search, candidate));

        [Fact]
        public Task Enter_in_the_search_box_is_a_submit() =>
            Realised(() => new FilterBar(), bar =>
            {
                int submits = 0;
                bar.Submitted += () => submits++;

                TextBox search = bar.FindPart<TextBox>()!;
                search.RaiseEvent(new Avalonia.Input.KeyEventArgs
                {
                    RoutedEvent = Avalonia.Input.InputElement.KeyDownEvent,
                    Key = Avalonia.Input.Key.Enter,
                });

                Assert.Equal(1, submits);
            });
    }
}
