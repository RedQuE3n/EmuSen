using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Interactivity;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;

namespace EmuSen.WiseMan.Mistress
{
    // The Apply and Save buttons - see EmuSen_Settings_Reference.md §4.15.
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

        private static CheatRegistry WithOneCheat(bool enabled = true)
        {
            var registry = new CheatRegistry();
            registry.AddRamPoke("CpuBus", 0x7E005E, 0x10, "fast walk", enabled);
            return registry;
        }

        private static ActiveCheatsWindow Open(CheatRegistry registry, Func<bool>? applyNow = null, Func<string?>? saveName = null)
        {
            var window = new ActiveCheatsWindow(registry, null, null, applyNow, saveName);
            window.Show();
            return window;
        }

        private static void Click(ActiveCheatsWindow w, string name) =>
            w.GetControl<Button>(name).RaiseEvent(new RoutedEventArgs(Button.ClickEvent));

        private static string Status(ActiveCheatsWindow w) => w.GetControl<TextBlock>("StatusText").Text!;

        [Fact]
        public Task Apply_pokes_the_running_core_and_says_how_many() => Session.Dispatch(() =>
        {
            int applies = 0;
            var window = Open(WithOneCheat(), applyNow: () => { applies++; return true; });

            Click(window, "ApplyButton");

            Assert.Equal(1, applies);
            Assert.Contains("Applied 1 cheat(s)", Status(window));

            window.Close();
        }, default);

        // Cheats can be arranged before a game starts, so Apply has to say
        // what it did rather than pretend it poked something.
        [Fact]
        public Task Apply_with_no_game_running_says_the_cheats_are_armed() => Session.Dispatch(() =>
        {
            var window = Open(WithOneCheat(), applyNow: () => false);

            Click(window, "ApplyButton");

            Assert.Contains("armed", Status(window));

            window.Close();
        }, default);

        [Fact]
        public Task Apply_with_nothing_ticked_says_so_rather_than_claiming_success() => Session.Dispatch(() =>
        {
            int applies = 0;
            var window = Open(WithOneCheat(enabled: false), applyNow: () => { applies++; return true; });

            Click(window, "ApplyButton");

            Assert.Equal(0, applies);
            Assert.Contains("nothing to apply", Status(window));

            window.Close();
        }, default);

        // Otherwise "Applied 1 cheat(s)" would be a lie for as long as the
        // master switch stayed off.
        [Fact]
        public Task Apply_turns_the_master_switch_back_on() => Session.Dispatch(() =>
        {
            CheatRegistry registry = WithOneCheat();
            registry.MasterEnabled = false;
            var window = Open(registry, applyNow: () => true);

            Click(window, "ApplyButton");

            Assert.True(registry.MasterEnabled);
            Assert.True(window.GetControl<CheckBox>("MasterSwitch").IsChecked);

            window.Close();
        }, default);

        [Fact]
        public Task Apply_and_save_are_off_while_the_list_is_empty() => Session.Dispatch(() =>
        {
            var window = Open(new CheatRegistry());

            Assert.False(window.GetControl<Button>("ApplyButton").IsEnabled);
            Assert.False(window.GetControl<Button>("SaveButton").IsEnabled);

            window.Close();
        }, default);

        [Fact]
        public Task Save_writes_a_cheat_file_named_after_the_game() => Session.Dispatch(() =>
        {
            var window = Open(WithOneCheat(), saveName: () => "Alpha");

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
            var registry = new CheatRegistry();
            registry.AddRamPoke("CpuBus", 0x7E005E, 0x10, "on one", enabled: true);
            registry.AddRamPoke("CpuBus", 0x7E005F, 0x11, "off one", enabled: false);
            var window = Open(registry, saveName: () => "Alpha");

            Click(window, "SaveButton");

            var restored = new CheatRegistry();
            restored.LoadFrom(CheatFile.For("Alpha").Load()!);

            Assert.Equal(new[] { true, false }, restored.GetCheats().Select(c => c.Enabled));

            window.Close();
        }, default);

        [Fact]
        public Task Save_with_no_game_running_says_what_it_needs() => Session.Dispatch(() =>
        {
            var window = Open(WithOneCheat(), saveName: () => null);

            Click(window, "SaveButton");

            Assert.Contains("Start a game first", Status(window));

            window.Close();
        }, default);
    }
}
