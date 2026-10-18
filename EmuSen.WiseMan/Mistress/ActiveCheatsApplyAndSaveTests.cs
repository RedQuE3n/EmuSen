using System;
using EmuSen.LunaP.Controls;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Interactivity;
using EmuSen.Mistress;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // The Apply and Save buttons - see EmuSen_Settings_Reference.md §4.15.
    [Collection(TestCollections.ProcessGlobals)]
    public class ActiveCheatsApplyAndSaveTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ActiveCheatsApplyAndSaveTests).GetTypeInfo().Assembly);

        private readonly string _root;

        public ActiveCheatsApplyAndSaveTests()
        {
            _root = Path.Combine(Path.GetTempPath(), "EmuSenCheatApplySave", Guid.NewGuid().ToString("N"));
            ConfigStore.OverrideDirectory = _root;
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private static CheatFileEntry Poke(string description, uint address = 0x7E005E, bool enabled = true) =>
            new() { Kind = "RamPoke", Description = description, Enabled = enabled, Writes = { new CheatFileWrite { Space = "CpuBus", Address = address.ToString("X"), Value = "10" } } };

        // A list of one held with no game, as one arranged before a game starts is.
        private static CheatList WithOneCheat(bool enabled = true)
        {
            var list = new CheatList();
            list.Import(new CheatFile { Cheats = { Poke("fast walk", enabled: enabled) } });
            return list;
        }

        private static ActiveCheatsWindow Open(CheatList list, Func<GameCheats?>? saveTarget = null)
        {
            var window = new ActiveCheatsWindow(list, saveTarget);
            window.Show();
            return window;
        }

        private static void Click(ActiveCheatsWindow w, string name) =>
            w.GetControl<Button>(name).RaiseEvent(new RoutedEventArgs(Button.ClickEvent));

        private static string Status(ActiveCheatsWindow w) => w.GetControl<TextBlock>("StatusText").Text!;

        // Save's counterpart: pulls the running game's saved list back - see §4.15.
        [Fact]
        public Task Load_restores_the_saved_list_over_whatever_is_live() => Session.Dispatch(() =>
        {
            Assert.True(CheatFile.For("Alpha").Save(new CheatFile { Cheats = { Poke("fast walk") } }));

            var live = new CheatList();
            live.Import(new CheatFile { Cheats = { Poke("something else", 0x7E0001) } });
            var window = Open(live, saveTarget: () => GameCheats.For(null, "Alpha.smc"));

            Click(window, "LoadButton");

            Assert.Equal("fast walk", live.Cheats.Single().Description);
            Assert.Contains("Loaded 1 cheat(s)", Status(window));

            window.Close();
        }, default);

        [Fact]
        public Task Load_is_off_until_there_is_a_file_to_load() => Session.Dispatch(() =>
        {
            var window = Open(WithOneCheat(), saveTarget: () => GameCheats.For(null, "NeverSaved.smc"));
            Assert.False(window.GetControl<Button>("LoadButton").IsEnabled);

            Click(window, "SaveButton");

            Assert.True(window.GetControl<Button>("LoadButton").IsEnabled);
            window.Close();
        }, default);

        [Fact]
        public Task Load_with_no_game_running_says_what_it_needs() => Session.Dispatch(() =>
        {
            var window = Open(WithOneCheat());

            Click(window, "LoadButton");

            Assert.Contains("Start a game first", Status(window));
            window.Close();
        }, default);

        // Save As writes the same shape, so a Load From round-trips it - see §4.15.
        [Fact]
        public Task A_save_as_file_round_trips_through_load_from() => Session.Dispatch(() =>
        {
            string path = Path.Combine(_root, "elsewhere", "mycheats.json");
            var source = WithOneCheat();
            Assert.True(CheatFile.SaveTo(path, source.ToFile()));

            CheatFile? read = CheatFile.LoadFrom(path);
            Assert.NotNull(read);

            using var game = new RuntimeGame("SNES");
            (int loaded, int skipped) = game.Cheats.Import(read!);

            Assert.Equal(1, loaded);
            Assert.Equal(0, skipped);
            Assert.Equal("fast walk", game.Cheats.Cheats.Single().Description);
            return Task.CompletedTask;
        }, default);

        [Fact]
        public void Load_from_a_path_that_is_not_a_cheat_list_returns_null()
        {
            string path = Path.Combine(_root, "junk.json");
            Directory.CreateDirectory(_root);
            File.WriteAllText(path, "this is not json");

            Assert.Null(CheatFile.LoadFrom(path));
        }

        // A running session applies its list at every frame boundary, so Apply says so.
        [Fact]
        public Task Apply_on_a_running_game_says_how_many() => Session.Dispatch(() =>
        {
            using var game = new RuntimeGame("SNES");
            game.Cheats.Import(new CheatFile { Cheats = { Poke("fast walk") } });
            var window = Open(game.Cheats);

            Click(window, "ApplyButton");

            Assert.Contains("Applied 1 cheat(s)", Status(window));

            window.Close();
        }, default);

        // Cheats can be arranged before a game starts, so Apply has to say
        // what it did rather than pretend it poked something.
        [Fact]
        public Task Apply_with_no_game_running_says_the_cheats_are_armed() => Session.Dispatch(() =>
        {
            var window = Open(WithOneCheat());

            Click(window, "ApplyButton");

            Assert.Contains("armed", Status(window));

            window.Close();
        }, default);

        [Fact]
        public Task Apply_with_nothing_ticked_says_so_rather_than_claiming_success() => Session.Dispatch(() =>
        {
            var window = Open(WithOneCheat(enabled: false));

            Click(window, "ApplyButton");

            Assert.Contains("nothing to apply", Status(window));

            window.Close();
        }, default);

        // Otherwise "Applied 1 cheat(s)" would be a lie for as long as the
        // master switch stayed off.
        [Fact]
        public Task Apply_turns_the_master_switch_back_on() => Session.Dispatch(() =>
        {
            CheatList list = WithOneCheat();
            list.Master = false;
            var window = Open(list);

            Click(window, "ApplyButton");

            Assert.True(list.Master);
            Assert.True(window.GetControl<LunaSwitch>("MasterSwitch").IsChecked);

            window.Close();
        }, default);

        [Fact]
        public Task Apply_and_save_are_off_while_the_list_is_empty() => Session.Dispatch(() =>
        {
            var window = Open(new CheatList());

            Assert.False(window.GetControl<Button>("ApplyButton").IsEnabled);
            Assert.False(window.GetControl<Button>("SaveButton").IsEnabled);

            window.Close();
        }, default);

        [Fact]
        public Task Save_writes_a_cheat_file_named_after_the_game() => Session.Dispatch(() =>
        {
            var window = Open(WithOneCheat(), saveTarget: () => GameCheats.For(null, "Alpha.smc"));

            Click(window, "SaveButton");

            CheatFile? saved = CheatFile.For("Alpha").Load();
            Assert.NotNull(saved);
            Assert.Equal("fast walk", saved!.Cheats.Single().Description);
            Assert.Contains("Saved 1 cheat(s)", Status(window));

            window.Close();
        }, default);

        // Each cheat's own state is part of what a saved list is for - a
        // player wants the arrangement back, not just the codes.
        [Fact]
        public Task A_saved_list_keeps_which_cheats_were_on() => Session.Dispatch(() =>
        {
            var list = new CheatList();
            list.Import(new CheatFile { Cheats = { Poke("on one", enabled: true), Poke("off one", 0x7E005F, enabled: false) } });
            var window = Open(list, saveTarget: () => GameCheats.For(null, "Alpha.smc"));

            Click(window, "SaveButton");

            Assert.Equal(new[] { true, false }, CheatFile.For("Alpha").Load()!.Cheats.Select(c => c.Enabled));

            window.Close();
        }, default);

        [Fact]
        public Task Save_with_no_game_running_says_what_it_needs() => Session.Dispatch(() =>
        {
            var window = Open(WithOneCheat(), saveTarget: () => null);

            Click(window, "SaveButton");

            Assert.Contains("Start a game first", Status(window));

            window.Close();
        }, default);
    }
}
