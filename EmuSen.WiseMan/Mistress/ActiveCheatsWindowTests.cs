using System.Linq;
using EmuSen.LunaP.Controls;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Interactivity;
using Avalonia.LogicalTree;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress;
using EmuSen.WiseMan.LunaP;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // The GUI half of `cheat list`/`enable`/`disable`/`remove`/`master` - see `man cheat` and EmuSen_Backend.md §14.13.
    [Collection(TestCollections.ProcessGlobals)]
    public class ActiveCheatsWindowTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ActiveCheatsWindowTests).GetTypeInfo().Assembly);

        private static ActiveCheatsWindow Open(CheatList list, string console = "SNES")
        {
            var window = new ActiveCheatsWindow(list, console: console);
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

        private static LunaTable<CheatRow> Table(ActiveCheatsWindow w) =>
            w.GetControl<LunaTable<CheatRow>>("CheatsList");

        private static CheatRow[] Rows(ActiveCheatsWindow w) => Table(w).Models.ToArray();

        // Through the table's own row list, so this is the selection a click makes -
        // LunaTable.Select is deliberately silent and would not enable Remove. See §4.14a.
        private static void SelectRow(ActiveCheatsWindow w, int index) =>
            Table(w).FindPart<ListBox>()!.SelectedIndex = index;

        private static TextBlock Status(ActiveCheatsWindow w) => w.GetControl<TextBlock>("StatusText");

        private static CheatFileEntry Poke(string description) => new() { Kind = "RamPoke", Description = description, Enabled = false, Writes = { new CheatFileWrite { Space = "WRAM", Address = "9C", Value = "63" } } };

        private static CheatFileEntry Patch(string description) => new() { Kind = "RomPatch", Description = description, Enabled = false, Writes = { new CheatFileWrite { Address = "008000", Value = "EA" } } };

        // Two cheats held with no game, as a list loaded from the database before one starts is.
        private static CheatList WithTwoCheats()
        {
            var list = new CheatList();
            list.Import(new CheatFile { Cheats = { Poke("infinite lives"), Patch("no music") } });
            return list;
        }

        private static void Add(ActiveCheatsWindow window, string code, string? description = null)
        {
            CodeBox(window).Text = code;
            if (description is not null) DescriptionBox(window).Text = description;
            AddButton(window).RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
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
            var window = Open(new CheatList());

            Assert.Empty(Rows(window));
            Assert.Contains("No cheats loaded", window.GetControl<TextBlock>("ListHeaderText").Text!);
            Assert.Contains("Cheat Database", Status(window).Text!);

            window.Close();
        }, default);

        // The row's own box is the deactivate control the shell spells `cheat disable <id>`, on the running session's list.
        [Fact]
        public Task Ticking_a_row_enables_that_cheat_in_the_session() => Session.Dispatch(() =>
        {
            using var game = new RuntimeGame("SNES");
            game.Cheats.Import(new CheatFile { Cheats = { Poke("infinite lives"), Patch("no music") } });
            var window = Open(game.Cheats);

            Rows(window)[0].Enabled = true;

            Assert.Equal(new[] { true, false }, game.Cheats.Cheats.Select(c => c.Enabled));

            Rows(window)[0].Enabled = false;
            Assert.False(game.Cheats.Cheats[0].Enabled);

            window.Close();
        }, default);

        [Fact]
        public Task The_master_switch_starts_on_and_writes_through() => Session.Dispatch(() =>
        {
            CheatList list = WithTwoCheats();
            var window = Open(list);
            var master = window.GetControl<LunaSwitch>("MasterSwitch");

            Assert.True(master.IsChecked);

            master.IsChecked = false;
            Assert.False(list.Master);
            Assert.Contains("off", Status(window).Text!);

            master.IsChecked = true;
            Assert.True(list.Master);

            window.Close();
        }, default);

        // Flipping the master switch must not rewrite each cheat's own box.
        [Fact]
        public Task The_master_switch_leaves_each_rows_own_box_alone() => Session.Dispatch(() =>
        {
            CheatList list = WithTwoCheats();
            var window = Open(list);
            Rows(window)[0].Enabled = true;

            window.GetControl<LunaSwitch>("MasterSwitch").IsChecked = false;

            Assert.True(Rows(window)[0].Enabled);
            Assert.False(Rows(window)[1].Enabled);

            window.Close();
        }, default);

        // A list that arrives with the switch already off must show that, not a ticked box over a dead list.
        [Fact]
        public Task An_already_off_list_opens_showing_it_off() => Session.Dispatch(() =>
        {
            CheatList list = WithTwoCheats();
            list.Master = false;

            var window = Open(list);

            Assert.False(window.GetControl<LunaSwitch>("MasterSwitch").IsChecked);
            Assert.False(list.Master, "opening the window must not flip the switch back on");

            window.Close();
        }, default);

        [Fact]
        public Task A_pro_action_replay_code_can_be_added_by_hand() => Session.Dispatch(() =>
        {
            using var game = new RuntimeGame("SNES");
            var window = Open(game.Cheats);

            Add(window, "7E0DBF63", "max coins");

            CheatLine added = game.Cheats.Cheats.Single();
            Assert.Equal("max coins", added.Description);
            Assert.False(added.RomPatch);
            Assert.True(added.Enabled);
            Assert.Empty(CodeBox(window).Text!);

            window.Close();
        }, default);

        // The 4-then-4 punctuation is the Game Genie convention `cheat add` guesses on - see `man cheat`.
        [Fact]
        public Task A_game_genie_code_is_added_as_a_rom_patch() => Session.Dispatch(() =>
        {
            using var game = new RuntimeGame("SNES");
            var window = Open(game.Cheats);

            Add(window, "DD82-64DD");

            Assert.True(game.Cheats.Cheats.Single().RomPatch);

            window.Close();
        }, default);

        // NES codes carry no punctuation to guess from, so the runtime's codecs are asked which one claims the code - see EmuSen_Cheats.md §2.
        [Fact]
        public Task An_unpunctuated_nes_game_genie_code_is_added_as_a_rom_patch() => Session.Dispatch(() =>
        {
            using var game = new RuntimeGame("NES");
            var window = Open(game.Cheats, "NES");

            Add(window, "SXIOPO");

            CheatFileEntry added = game.Cheats.ToFile().Cheats.Single();
            Assert.True(added.IsRomPatch);
            Assert.Equal(0x91D9u, Address(added.Writes[0]));

            window.Close();
        }, default);

        private static uint Address(CheatFileWrite write) => CheatFileWrite.TryHex(write.Address, out uint address) ? address : uint.MaxValue;

        // The compare is what makes an 8-letter code target one bank.
        [Fact]
        public Task An_eight_letter_nes_code_keeps_its_compare_byte() => Session.Dispatch(() =>
        {
            using var game = new RuntimeGame("NES");
            var window = Open(game.Cheats, "NES");

            Add(window, "SLXPLOVS");

            Assert.Equal("DE", game.Cheats.ToFile().Cheats.Single().Compare);

            window.Close();
        }, default);

        [Fact]
        public Task An_nes_address_value_code_is_added_as_a_ram_poke() => Session.Dispatch(() =>
        {
            using var game = new RuntimeGame("NES");
            var window = Open(game.Cheats, "NES");

            Add(window, "0075:09");

            CheatFileEntry added = game.Cheats.ToFile().Cheats.Single();
            Assert.False(added.IsRomPatch);
            Assert.Equal("CPUBUS", added.Writes[0].Space);

            window.Close();
        }, default);

        // A GameShark code is lines, a test and the write it guards, so the window adds it whole - see EmuSen_Cheats.md §7.
        [Fact]
        public Task An_n64_code_of_several_lines_is_added_as_one_cheat() => Session.Dispatch(() =>
        {
            using var game = new RuntimeGame("N64");
            var window = Open(game.Cheats, "N64");

            Add(window, "D0000200 0005+81000100 1234");

            CheatFileEntry added = game.Cheats.ToFile().Cheats.Single();
            Assert.False(added.IsRomPatch);
            Assert.Equal(new[] { "IfEqual", "Set" }, added.Writes.Select(w => w.Type));
            Assert.True(CheatFileWrite.TryHex(added.Writes[1].Value, out uint value) && value == 0x1234);
            Assert.Equal("RDRAM", added.Writes[1].Space);
            Assert.Contains("as GameShark", Status(window).Text!);

            window.Close();
        }, default);

        [Fact]
        public Task An_n64_code_mars_refuses_says_why_and_adds_nothing() => Session.Dispatch(() =>
        {
            using var game = new RuntimeGame("N64");
            var window = Open(game.Cheats, "N64");

            Add(window, "F1000318 0040");

            Assert.Empty(game.Cheats.Cheats);
            Assert.Contains("boot-time write", Status(window).Text!);

            window.Close();
        }, default);

        // With no ROM-patch codec to guess, a mistyped code reaches the GameShark, whose error says what is wrong - see EmuSen_Cheats.md §7.
        [Fact]
        public Task A_malformed_n64_code_is_answered_by_the_gameshark() => Session.Dispatch(() =>
        {
            using var game = new RuntimeGame("N64");
            var window = Open(game.Cheats, "N64");

            Add(window, "8033-B21E");

            Assert.Empty(game.Cheats.Cheats);
            Assert.Contains("isn't an N64 GameShark code", Status(window).Text!);

            window.Close();
        }, default);

        [Fact]
        public Task An_undecodable_code_reports_it_and_adds_nothing() => Session.Dispatch(() =>
        {
            using var game = new RuntimeGame("SNES");
            var window = Open(game.Cheats);

            Add(window, "not-a-code");

            Assert.Empty(game.Cheats.Cheats);
            Assert.Contains("Couldn't decode", Status(window).Text!);

            window.Close();
        }, default);

        [Fact]
        public Task An_empty_code_box_says_so_rather_than_doing_nothing() => Session.Dispatch(() =>
        {
            using var game = new RuntimeGame("SNES");
            var window = Open(game.Cheats);

            AddButton(window).RaiseEvent(new RoutedEventArgs(Button.ClickEvent));

            Assert.Empty(game.Cheats.Cheats);
            Assert.Contains("Type a cheat code", Status(window).Text!);

            window.Close();
        }, default);

        [Fact]
        public Task Remove_takes_out_only_the_selected_cheat() => Session.Dispatch(() =>
        {
            CheatList list = WithTwoCheats();
            var window = Open(list);
            Assert.False(window.GetControl<Button>("RemoveButton").IsEnabled);

            SelectRow(window, 0);
            Assert.True(window.GetControl<Button>("RemoveButton").IsEnabled);
            window.GetControl<Button>("RemoveButton").RaiseEvent(new RoutedEventArgs(Button.ClickEvent));

            Assert.Equal("no music", list.Cheats.Single().Description);
            Assert.Single(Rows(window));

            window.Close();
        }, default);

        [Fact]
        public Task Remove_all_empties_the_list() => Session.Dispatch(() =>
        {
            CheatList list = WithTwoCheats();
            var window = Open(list);

            window.GetControl<Button>("ClearButton").RaiseEvent(new RoutedEventArgs(Button.ClickEvent));

            Assert.Empty(list.Cheats);
            Assert.Empty(Rows(window));
            Assert.False(window.GetControl<Button>("ClearButton").IsEnabled);

            window.Close();
        }, default);

        // What the Cheat Database window's callback triggers - see §4.14.
        [Fact]
        public Task Refresh_picks_up_cheats_added_from_outside_the_window() => Session.Dispatch(() =>
        {
            var list = new CheatList();
            var window = Open(list);
            Assert.Empty(Rows(window));

            list.Import(new CheatFile { Cheats = { Poke("loaded elsewhere") } });
            window.Refresh();

            Assert.Equal("loaded elsewhere", Rows(window).Single().Description);

            window.Close();
        }, default);

        // A code is read by the running game's codecs, so only that game's tab takes one; another console's tab, and every tab with no game, says a game of it must run (decided 2026-10-18; EmuSen_Backend.md §14.13).
        [Fact]
        public Task Only_the_running_game_s_tab_takes_a_code() => Session.Dispatch(() =>
        {
            var held = new ActiveCheatsWindow(new CheatList());
            held.Show();
            var heldTabs = held.GetControl<TabControl>("Tabs");
            foreach (TabItem item in heldTabs.Items.OfType<TabItem>().Skip(1))
            {
                heldTabs.SelectedItem = item;
                Assert.False(AddButton(held).IsEnabled, $"{item.Header}: a code was offered with no game running");
            }
            held.Close();

            using var game = new RuntimeGame("SNES");
            var window = Open(game.Cheats);
            var tabs = window.GetControl<TabControl>("Tabs");
            tabs.SelectedItem = tabs.Items.OfType<TabItem>().First(t => (string)t.Header! == "NES");
            Assert.False(AddButton(window).IsEnabled);
            Assert.Contains(((Control)((TabItem)tabs.SelectedItem!).Content!).GetLogicalDescendants().OfType<TextBlock>(), t => t.Text?.StartsWith("Start a NES game", System.StringComparison.Ordinal) == true);
            tabs.SelectedItem = tabs.Items.OfType<TabItem>().First(t => (string)t.Header! == "SNES");
            Assert.True(AddButton(window).IsEnabled);
            Add(window, "7E0DBF63");
            Assert.Single(game.Cheats.Cheats);

            window.Close();
        }, default);

        // A list is a list: every tab shows the one list.
        [Fact]
        public Task Every_tab_shows_the_same_live_list() => Session.Dispatch(() =>
        {
            var window = new ActiveCheatsWindow(WithTwoCheats());
            window.Show();
            var tabs = window.GetControl<TabControl>("Tabs");

            foreach (TabItem item in tabs.Items.OfType<TabItem>())
            {
                tabs.SelectedItem = item;
                Assert.Equal(2, Rows(window).Length);
            }

            window.Close();
        }, default);

        // Every console of the runtime's shelves gets a tab, and the running game's names the formats its codecs read - see §4.14.
        [Theory]
        [InlineData("SNES", "Game Genie")]
        [InlineData("NES", "Game Genie")]
        [InlineData("N64", "GameShark")]
        public Task The_running_game_s_tab_offers_its_codecs_formats(string console, string format) => Session.Dispatch(() =>
        {
            using var game = new RuntimeGame(console);
            var window = Open(game.Cheats, console);

            var tabs = window.GetControl<TabControl>("Tabs");
            var headers = tabs.Items.OfType<TabItem>().Select(t => (string)t.Header!).ToArray();
            Assert.Equal(new[] { "General", "NES", "GB", "SNES", "N64", "SG-1000", "Master System", "Genesis", "Sega CD", "PlayStation" }, headers);

            var item = (TabItem)tabs.SelectedItem!;
            Assert.Equal(console, item.Header);
            string[] texts = ((Control)item.Content!).GetLogicalDescendants().OfType<TextBlock>().Select(t => t.Text ?? "").ToArray();
            Assert.True(AddButton(window).IsEnabled, $"{console} should accept a typed code.");
            Assert.Contains(format, texts.First(t => t.StartsWith("Accepts:")));

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

                var item = MainWindowMenu.Find(main, "_Settings", "_Active Cheats...");

                Assert.True(item.IsEnabled);
                item.Invoke();

                main.Close();
            }
            finally
            {
                ConfigStore.OverrideDirectory = null;
            }
        }, default);
    }
}
