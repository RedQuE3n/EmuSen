using System.Linq;
using EmuSen.LunaP.Controls;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Interactivity;
using Avalonia.LogicalTree;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;

namespace EmuSen.WiseMan.Mistress
{
    // The GUI half of `cheat list`/`enable`/`disable`/`remove`/`master` -
    // see `man cheat` and EmuSen_Settings_Reference.md §4.14.
    public class ActiveCheatsWindowTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ActiveCheatsWindowTests).GetTypeInfo().Assembly);

        private static ICheatCodeCodec Poke() => new EmuSen.Cores.Nintendo.Venus.Cheats.ActionReplayCheatCodec();
        private static ICheatCodeCodec Patch() => new EmuSen.Cores.Nintendo.Venus.Cheats.GameGenieCheatCodec();

        private static ActiveCheatsWindow Open(CheatRegistry registry)
        {
            var window = new ActiveCheatsWindow(registry, Poke(), Patch(), console: "SNES");
            window.Show();
            return window;
        }

        // The Add box lives on the selected console tab now - see EmuSen_Settings_Reference.md §4.14.
        private static T OnSelectedTab<T>(ActiveCheatsWindow w, string suffix) where T : Control
        {
            var item = (TabItem)w.GetControl<TabControl>("Tabs").SelectedItem!;
            string console = (string)item.Header!;
            return ((Control)item.Content!).GetLogicalDescendants().OfType<T>().First(c => c.Name == console + suffix);
        }

        private static TextBox CodeBox(ActiveCheatsWindow w) => OnSelectedTab<TextBox>(w, "CodeBox");
        private static TextBox DescriptionBox(ActiveCheatsWindow w) => OnSelectedTab<TextBox>(w, "DescriptionBox");
        private static Button AddButton(ActiveCheatsWindow w) => OnSelectedTab<Button>(w, "AddButton");

        private static CheatRow[] Rows(ActiveCheatsWindow w) =>
            w.GetControl<ListBox>("CheatsList").ItemsSource!.Cast<CheatRow>().ToArray();

        private static TextBlock Status(ActiveCheatsWindow w) => w.GetControl<TextBlock>("StatusText");

        private static CheatRegistry WithTwoCheats()
        {
            var registry = new CheatRegistry();
            registry.AddRamPoke("WRAM", 0x9C, 0x63, "infinite lives", enabled: false);
            registry.AddRomPatch(0x008000, 0xEA, null, "no music", enabled: false);
            return registry;
        }

        [Fact]
        public Task Every_loaded_cheat_is_listed_with_its_kind_and_state() => Session.Dispatch(() =>
        {
            var window = Open(WithTwoCheats());

            CheatRow[] rows = Rows(window);
            Assert.Equal(2, rows.Length);
            Assert.Equal(new[] { "infinite lives", "no music" }, rows.Select(r => r.Description));
            Assert.Equal(new[] { "RAM", "ROM" }, rows.Select(r => r.Kind));
            Assert.All(rows, r => Assert.False(r.Enabled));
            Assert.Contains("2 cheat(s), 0 on", window.GetControl<TextBlock>("ListHeaderText").Text!);

            window.Close();
        }, default);

        [Fact]
        public Task An_empty_list_says_where_cheats_come_from() => Session.Dispatch(() =>
        {
            var window = Open(new CheatRegistry());

            Assert.Empty(Rows(window));
            Assert.Contains("No cheats loaded", window.GetControl<TextBlock>("ListHeaderText").Text!);
            Assert.Contains("Cheat Database", Status(window).Text!);

            window.Close();
        }, default);

        // The row's own box is the deactivate control the shell spells
        // `cheat disable <id>`.
        [Fact]
        public Task Ticking_a_row_enables_that_cheat_in_the_registry() => Session.Dispatch(() =>
        {
            CheatRegistry registry = WithTwoCheats();
            var window = Open(registry);

            Rows(window)[0].Enabled = true;

            Assert.True(registry.GetCheats()[0].Enabled);
            Assert.False(registry.GetCheats()[1].Enabled);

            Rows(window)[0].Enabled = false;
            Assert.False(registry.GetCheats()[0].Enabled);

            window.Close();
        }, default);

        [Fact]
        public Task The_master_switch_starts_on_and_writes_through() => Session.Dispatch(() =>
        {
            CheatRegistry registry = WithTwoCheats();
            var window = Open(registry);
            var master = window.GetControl<LunaSwitch>("MasterSwitch");

            Assert.True(master.IsChecked);

            master.IsChecked = false;
            Assert.False(registry.MasterEnabled);
            Assert.Contains("off", Status(window).Text!);

            master.IsChecked = true;
            Assert.True(registry.MasterEnabled);

            window.Close();
        }, default);

        // Flipping the master switch must not rewrite each cheat's own box.
        [Fact]
        public Task The_master_switch_leaves_each_rows_own_box_alone() => Session.Dispatch(() =>
        {
            CheatRegistry registry = WithTwoCheats();
            var window = Open(registry);
            Rows(window)[0].Enabled = true;

            window.GetControl<LunaSwitch>("MasterSwitch").IsChecked = false;

            Assert.True(Rows(window)[0].Enabled);
            Assert.False(Rows(window)[1].Enabled);

            window.Close();
        }, default);

        // A registry that arrives with the switch already off must show that,
        // not a ticked box over a dead list.
        [Fact]
        public Task An_already_off_registry_opens_showing_it_off() => Session.Dispatch(() =>
        {
            CheatRegistry registry = WithTwoCheats();
            registry.MasterEnabled = false;

            var window = Open(registry);

            Assert.False(window.GetControl<LunaSwitch>("MasterSwitch").IsChecked);
            Assert.True(registry.MasterEnabled == false, "opening the window must not flip the switch back on");

            window.Close();
        }, default);

        [Fact]
        public Task A_pro_action_replay_code_can_be_added_by_hand() => Session.Dispatch(() =>
        {
            var registry = new CheatRegistry();
            var window = Open(registry);

            CodeBox(window).Text = "7E0DBF63";
            DescriptionBox(window).Text = "max coins";
            AddButton(window).RaiseEvent(new RoutedEventArgs(Button.ClickEvent));

            CheatInfo added = registry.GetCheats().Single();
            Assert.Equal("max coins", added.Description);
            Assert.Equal(CheatKind.RamPoke, added.Kind);
            Assert.True(added.Enabled);
            Assert.Empty(CodeBox(window).Text!);

            window.Close();
        }, default);

        // The 4-then-4 punctuation is the Game Genie convention `cheat add`
        // guesses on - see `man cheat`.
        [Fact]
        public Task A_game_genie_code_is_added_as_a_rom_patch() => Session.Dispatch(() =>
        {
            var registry = new CheatRegistry();
            var window = Open(registry);

            CodeBox(window).Text = "DD82-64DD";
            AddButton(window).RaiseEvent(new RoutedEventArgs(Button.ClickEvent));

            Assert.Equal(CheatKind.RomPatch, registry.GetCheats().Single().Kind);

            window.Close();
        }, default);

        // NES codes carry no punctuation to guess from, so the window asks the
        // codecs which one claims the code - see EmuSen_Cheats.md §2.
        private static ActiveCheatsWindow OpenNes(CheatRegistry registry)
        {
            var window = new ActiveCheatsWindow(registry,
                new EmuSen.Cores.Nintendo.Moon.Cheats.NesRawCheatCodec(),
                new EmuSen.Cores.Nintendo.Moon.Cheats.NesGameGenieCheatCodec(), console: "NES");
            window.Show();
            return window;
        }

        [Fact]
        public Task An_unpunctuated_nes_game_genie_code_is_added_as_a_rom_patch() => Session.Dispatch(() =>
        {
            var registry = new CheatRegistry();
            var window = OpenNes(registry);

            CodeBox(window).Text = "SXIOPO";
            AddButton(window).RaiseEvent(new RoutedEventArgs(Button.ClickEvent));

            var added = registry.GetCheats().Single();
            Assert.Equal(CheatKind.RomPatch, added.Kind);
            Assert.Equal(0x91D9, added.Writes[0].Address);

            window.Close();
        }, default);

        // The compare is what makes an 8-letter code target one bank.
        [Fact]
        public Task An_eight_letter_nes_code_keeps_its_compare_byte() => Session.Dispatch(() =>
        {
            var registry = new CheatRegistry();
            var window = OpenNes(registry);

            CodeBox(window).Text = "SLXPLOVS";
            AddButton(window).RaiseEvent(new RoutedEventArgs(Button.ClickEvent));

            Assert.Equal((byte)0xDE, registry.GetCheats().Single().Compare);

            window.Close();
        }, default);

        [Fact]
        public Task An_nes_address_value_code_is_added_as_a_ram_poke() => Session.Dispatch(() =>
        {
            var registry = new CheatRegistry();
            var window = OpenNes(registry);

            CodeBox(window).Text = "0075:09";
            AddButton(window).RaiseEvent(new RoutedEventArgs(Button.ClickEvent));

            var added = registry.GetCheats().Single();
            Assert.Equal(CheatKind.RamPoke, added.Kind);
            Assert.Equal("CPUBUS", added.Writes[0].Space);

            window.Close();
        }, default);

        [Fact]
        public Task An_undecodable_code_reports_it_and_adds_nothing() => Session.Dispatch(() =>
        {
            var registry = new CheatRegistry();
            var window = Open(registry);

            CodeBox(window).Text = "not-a-code";
            AddButton(window).RaiseEvent(new RoutedEventArgs(Button.ClickEvent));

            Assert.Empty(registry.GetCheats());
            Assert.Contains("Couldn't decode", Status(window).Text!);

            window.Close();
        }, default);

        [Fact]
        public Task An_empty_code_box_says_so_rather_than_doing_nothing() => Session.Dispatch(() =>
        {
            var registry = new CheatRegistry();
            var window = Open(registry);

            AddButton(window).RaiseEvent(new RoutedEventArgs(Button.ClickEvent));

            Assert.Empty(registry.GetCheats());
            Assert.Contains("Type a cheat code", Status(window).Text!);

            window.Close();
        }, default);

        [Fact]
        public Task Remove_takes_out_only_the_selected_cheat() => Session.Dispatch(() =>
        {
            CheatRegistry registry = WithTwoCheats();
            var window = Open(registry);
            var list = window.GetControl<ListBox>("CheatsList");

            Assert.False(window.GetControl<Button>("RemoveButton").IsEnabled);

            list.SelectedIndex = 0;
            Assert.True(window.GetControl<Button>("RemoveButton").IsEnabled);
            window.GetControl<Button>("RemoveButton").RaiseEvent(new RoutedEventArgs(Button.ClickEvent));

            Assert.Equal("no music", registry.GetCheats().Single().Description);
            Assert.Single(Rows(window));

            window.Close();
        }, default);

        [Fact]
        public Task Remove_all_empties_the_list() => Session.Dispatch(() =>
        {
            CheatRegistry registry = WithTwoCheats();
            var window = Open(registry);

            window.GetControl<Button>("ClearButton").RaiseEvent(new RoutedEventArgs(Button.ClickEvent));

            Assert.Empty(registry.GetCheats());
            Assert.Empty(Rows(window));
            Assert.False(window.GetControl<Button>("ClearButton").IsEnabled);

            window.Close();
        }, default);

        // What the Cheat Database window's callback triggers - see §4.14.
        [Fact]
        public Task Refresh_picks_up_cheats_added_from_outside_the_window() => Session.Dispatch(() =>
        {
            var registry = new CheatRegistry();
            var window = Open(registry);
            Assert.Empty(Rows(window));

            registry.AddRamPoke("WRAM", 0x9C, 0x63, "loaded elsewhere", enabled: false);
            window.Refresh();

            Assert.Equal("loaded elsewhere", Rows(window).Single().Description);

            window.Close();
        }, default);

        // The whole point of the tabs: the same typed text means different things per console.
        [Fact]
        public Task The_tab_a_code_is_typed_under_decides_how_it_is_parsed() => Session.Dispatch(() =>
        {
            var registry = new CheatRegistry();
            var window = new ActiveCheatsWindow(registry);
            window.Show();
            var tabs = window.GetControl<TabControl>("Tabs");

            // Eight hex digits: a Pro Action Replay poke on the SNES.
            tabs.SelectedItem = tabs.Items.OfType<TabItem>().First(t => (string)t.Header! == "SNES");
            CodeBox(window).Text = "7E0DBF63";
            AddButton(window).RaiseEvent(new RoutedEventArgs(Button.ClickEvent));

            // Six letters: a Game Genie ROM patch on the NES.
            tabs.SelectedItem = tabs.Items.OfType<TabItem>().First(t => (string)t.Header! == "NES");
            CodeBox(window).Text = "SXIOPO";
            AddButton(window).RaiseEvent(new RoutedEventArgs(Button.ClickEvent));

            var kinds = registry.GetCheats().Select(c => c.Kind).ToArray();
            Assert.Equal(new[] { CheatKind.RamPoke, CheatKind.RomPatch }, kinds);

            window.Close();
        }, default);

        // A list is a list: both consoles' cheats sit in the one live registry.
        [Fact]
        public Task Every_tab_shows_the_same_live_list() => Session.Dispatch(() =>
        {
            var registry = WithTwoCheats();
            var window = new ActiveCheatsWindow(registry);
            window.Show();
            var tabs = window.GetControl<TabControl>("Tabs");

            foreach (TabItem item in tabs.Items.OfType<TabItem>())
            {
                tabs.SelectedItem = item;
                Assert.Equal(2, Rows(window).Length);
            }

            window.Close();
        }, default);

        // Every console in the catalog gets a tab, and each names the formats it takes - see §4.14.
        [Fact]
        public Task Each_console_tab_offers_its_own_formats() => Session.Dispatch(() =>
        {
            var window = new ActiveCheatsWindow(new CheatRegistry());
            window.Show();

            var tabs = window.GetControl<TabControl>("Tabs");
            var headers = tabs.Items.OfType<TabItem>().Select(t => (string)t.Header!).ToArray();
            Assert.Equal(new[] { "General", "NES", "SNES" }, headers);

            foreach (TabItem item in tabs.Items.OfType<TabItem>().Skip(1))
            {
                tabs.SelectedItem = item;
                string console = (string)item.Header!;
                Assert.True(AddButton(window).IsEnabled, $"{console} should accept a typed code.");

                string formats = ((Control)item.Content!).GetLogicalDescendants().OfType<TextBlock>()
                    .Select(t => t.Text ?? "").First(t => t.StartsWith("Accepts:"));
                Assert.Contains("Game Genie", formats);
            }

            window.Close();
        }, default);

        [Fact]
        public Task The_settings_menu_opens_it_without_needing_a_rom() => Session.Dispatch(() =>
        {
            ConfigStore.OverrideDirectory = System.IO.Path.Combine(
                System.IO.Path.GetTempPath(), "EmuSenActiveCheatsMenu", System.Guid.NewGuid().ToString("N"));

            try
            {
                var main = new MainWindow();
                main.Show();

                main.GetControl<MenuItem>("SettingsMenu").RaiseEvent(new RoutedEventArgs(MenuItem.SubmenuOpenedEvent));

                MenuItem item = main.GetControl<MenuItem>("SettingsMenu").Items
                    .OfType<MenuItem>()
                    .Single(m => (string?)m.Header == "_Active Cheats...");

                Assert.True(item.IsEnabled);
                item.RaiseEvent(new RoutedEventArgs(MenuItem.ClickEvent));

                main.Close();
            }
            finally
            {
                ConfigStore.OverrideDirectory = null;
            }
        }, default);
    }
}
