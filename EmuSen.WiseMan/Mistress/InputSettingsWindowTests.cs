using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.VisualTree;
using EmuSen.Cores;
using EmuSen.Mistress.Input;
using EmuSen.Endymion.Input;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;
using EmuSen.Galaxia.Input;

namespace EmuSen.WiseMan.Mistress
{
    // Rebind capture, driven through real Avalonia key events - see EmuSen_Settings_Reference.md §4.7.
    public class InputSettingsWindowTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(InputSettingsWindowTests).GetTypeInfo().Assembly);

        private readonly string _configDir;

        public InputSettingsWindowTests()
        {
            // Rebinding saves, so redirect it - see EmuSen_Settings_Reference.md §4.7.
            _configDir = Path.Combine(Path.GetTempPath(), "EmuSenInputSettingsTests", Guid.NewGuid().ToString("N"));
            ConfigStore.OverrideDirectory = _configDir;
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            try { if (Directory.Exists(_configDir)) Directory.Delete(_configDir, recursive: true); } catch { }
        }

        private static readonly string[] Consoles = { "NES", "SNES" };

        private sealed class Harness
        {
            public required InputSettingsWindow Window { get; init; }
            public required ControllerKeyBindings Bindings { get; init; }
            // The tab NewWindow opens on, so the button assertions below read as they always did.
            public required ControllerKeyMap Keys { get; init; }
            public required HotkeyBindingMap Hotkeys { get; init; }
        }

        // Opens on the SNES tab, the only one with all twelve buttons. Pass null
        // for General, which is where the hotkey rows live.
        private static Harness NewWindow(string? console = "SNES")
        {
            var keys = new ControllerKeyBindings(Consoles);
            var hotkeys = new HotkeyBindingMap();
            var window = new InputSettingsWindow(keys, new GamepadBindings(Consoles), null!, new AppSettings(), hotkeys, console);
            window.Show();
            window.UpdateLayout();
            return new Harness { Window = window, Bindings = keys, Keys = keys.For(console ?? "SNES"), Hotkeys = hotkeys };
        }

        // Column 0 only, or the match picks the wrong row - see EmuSen_Settings_Reference.md §4.7.
        private static bool IsRowNamed(Grid row, string rowLabel) =>
            row.Children.OfType<TextBlock>().Any(t => Grid.GetColumn(t) == 0 && t.Text == rowLabel);

        // Found by row text, the way a user sees them.
        private static Button RebindButtonFor(InputSettingsWindow window, string rowLabel, string buttonText)
        {
            foreach (Grid row in window.GetVisualDescendants().OfType<Grid>())
            {
                if (!IsRowNamed(row, rowLabel)) continue;

                Button? match = row.Children.OfType<Button>().FirstOrDefault(b => (b.Content as string) == buttonText);
                if (match is not null) return match;
            }
            throw new InvalidOperationException($"No '{buttonText}' button on a row labelled '{rowLabel}'.");
        }

        private static Button ButtonByContent(InputSettingsWindow window, string content) =>
            window.GetVisualDescendants().OfType<Button>().First(b => (b.Content as string) == content);

        private static string LabelTextFor(InputSettingsWindow window, string rowLabel, int columnIndex)
        {
            foreach (Grid row in window.GetVisualDescendants().OfType<Grid>())
            {
                if (!IsRowNamed(row, rowLabel)) continue;

                TextBlock? cell = row.Children.OfType<TextBlock>().FirstOrDefault(t => Grid.GetColumn(t) == columnIndex);
                if (cell is not null) return cell.Text ?? "";
            }
            throw new InvalidOperationException($"No column {columnIndex} label on a row labelled '{rowLabel}'.");
        }

        // The window only reads e.Key, so PhysicalKey.None is fine.
        private static void Press(InputSettingsWindow window, Key key) =>
            window.KeyPress(key, RawInputModifiers.None, PhysicalKey.None, null);

        // The focus a real click leaves behind is what swallowed the key.
        private static void ClickAsUser(Button button)
        {
            button.Focus();
            button.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
        }

        [Theory]
        // Keys a focused Button or the focus manager claims first.
        [InlineData(Key.Enter)]
        [InlineData(Key.Space)]
        [InlineData(Key.Up)]
        [InlineData(Key.Left)]
        // A plain letter never broke - guards against a wider regression.
        [InlineData(Key.K)]
        public Task Rebinding_a_key_updates_the_row_and_the_map(Key key) => Session.Dispatch(() =>
        {
            var h = NewWindow();

            ClickAsUser(RebindButtonFor(h.Window, "Up", "Rebind Key"));
            Press(h.Window, key);

            Assert.Equal(key, h.Keys.ButtonToKey[PadButton.Up]);
            Assert.Equal(key.ToString(), LabelTextFor(h.Window, "Up", 1));
        }, default);

        [Fact]
        public Task Escape_cancels_a_rebind_and_restores_the_prompt() => Session.Dispatch(() =>
        {
            var h = NewWindow();
            Key before = h.Keys.ButtonToKey[PadButton.A];

            Button rebind = RebindButtonFor(h.Window, "A", "Rebind Key");
            ClickAsUser(rebind);
            Press(h.Window, Key.Escape);

            Assert.Equal(before, h.Keys.ButtonToKey[PadButton.A]);
            Assert.Equal("Rebind Key", rebind.Content as string);
        }, default);

        // The captured key must not re-activate the focused button.
        [Fact]
        public Task Capturing_a_key_does_not_re_arm_the_rebind_button() => Session.Dispatch(() =>
        {
            var h = NewWindow();

            Button rebind = RebindButtonFor(h.Window, "Start", "Rebind Key");
            ClickAsUser(rebind);
            Press(h.Window, Key.Space);

            Assert.Equal("Rebind Key", rebind.Content as string);
            Assert.Equal(Key.Space, h.Keys.ButtonToKey[PadButton.Start]);
        }, default);

        // Two SNES buttons must never share one key.
        [Fact]
        public Task Taking_a_key_from_another_button_unbinds_that_button() => Session.Dispatch(() =>
        {
            var h = NewWindow();

            ClickAsUser(RebindButtonFor(h.Window, "A", "Rebind Key"));
            Press(h.Window, Key.Z); // Z was B's default

            Assert.Equal(Key.Z, h.Keys.ButtonToKey[PadButton.A]);
            Assert.False(h.Keys.ButtonToKey.ContainsKey(PadButton.B));
            Assert.Equal("(unbound)", LabelTextFor(h.Window, "B", 1));
        }, default);

        [Fact]
        public Task Clearing_a_binding_unbinds_it() => Session.Dispatch(() =>
        {
            var h = NewWindow();

            ClickAsUser(RebindButtonFor(h.Window, "Select", "Clear"));

            Assert.False(h.Keys.ButtonToKey.ContainsKey(PadButton.Select));
            Assert.Equal("(unbound)", LabelTextFor(h.Window, "Select", 1));
        }, default);

        // Hotkeys share the game-button listener.
        [Fact]
        public Task Rebinding_a_hotkey_updates_the_row_and_the_map() => Session.Dispatch(() =>
        {
            var h = NewWindow(null); // hotkeys are on General

            ClickAsUser(RebindButtonFor(h.Window, "Fast Forward", "Rebind Key"));
            Press(h.Window, Key.F5);

            Assert.Equal(Key.F5, h.Hotkeys.ActionToKey[HotkeyAction.FastForward]);
            Assert.Equal("F5", LabelTextFor(h.Window, "Fast Forward", 1));
        }, default);

        // The two maps must police each other, not just themselves.
        [Fact]
        // A hotkey is global, so it must clear that key on every console.
        public Task A_hotkey_taking_a_game_buttons_key_unbinds_the_game_button() => Session.Dispatch(() =>
        {
            var h = NewWindow(null); // hotkeys are on General

            ClickAsUser(RebindButtonFor(h.Window, "Rewind", "Rebind Key"));
            Press(h.Window, Key.X); // X was A's default on both consoles

            Assert.Equal(Key.X, h.Hotkeys.ActionToKey[HotkeyAction.Rewind]);
            Assert.False(h.Bindings.For("SNES").ButtonToKey.ContainsKey(PadButton.A));
            Assert.False(h.Bindings.For("NES").ButtonToKey.ContainsKey(PadButton.A));
        }, default);

        // A file written before an action existed leaves it unbound - see EmuSen_Settings_Reference.md §4.18.
        [Fact]
        public Task An_older_hotkey_file_gains_the_defaults_it_predates() => Session.Dispatch(() =>
        {
            var stale = new HotkeyBindingMap();
            stale.ActionToKey.Remove(HotkeyAction.ExitToLibrary);
            stale.Rebind(HotkeyAction.TogglePause, Key.F2); // a real preference, which must survive
            stale.Save();

            var loaded = HotkeyBindingMap.Load();

            Assert.Equal(Key.Escape, loaded.ActionToKey[HotkeyAction.ExitToLibrary]);
            Assert.Equal(Key.F2, loaded.ActionToKey[HotkeyAction.TogglePause]);
        }, default);

        // ...but not by stealing a key the user has since bound elsewhere.
        [Fact]
        public Task A_backfilled_default_never_overwrites_an_existing_binding() => Session.Dispatch(() =>
        {
            var stale = new HotkeyBindingMap();
            stale.ActionToKey.Remove(HotkeyAction.ExitToLibrary);
            stale.Rebind(HotkeyAction.SaveState, Key.Escape);
            stale.Save();

            var loaded = HotkeyBindingMap.Load();

            Assert.Equal(Key.Escape, loaded.ActionToKey[HotkeyAction.SaveState]);
            Assert.False(loaded.ActionToKey.ContainsKey(HotkeyAction.ExitToLibrary));
            Assert.True(loaded.TryGetAction(Key.Escape, out HotkeyAction owner));
            Assert.Equal(HotkeyAction.SaveState, owner);
        }, default);

        // A hand-edited config can still arrive with a clash.
        [Fact]
        public Task A_config_that_already_conflicts_is_reported() => Session.Dispatch(() =>
        {
            var keys = new ControllerKeyBindings(Consoles);
            var hotkeys = new HotkeyBindingMap();
            // Straight into the dictionary, bypassing Rebind's guard.
            keys.For("SNES").ButtonToKey[PadButton.A] = keys.For("SNES").ButtonToKey[PadButton.B];

            var window = new InputSettingsWindow(keys, new GamepadBindings(Consoles), null!, new AppSettings(), hotkeys, "SNES");
            window.Show();

            TextBlock conflict = window.GetVisualDescendants().OfType<TextBlock>()
                .First(t => t.Name == "ConflictText");

            Assert.Contains("Conflict", conflict.Text ?? "");
            Assert.Contains("A", conflict.Text ?? "");
            Assert.Contains("B", conflict.Text ?? "");
        }, default);

        [Fact]
        public Task No_conflict_message_on_a_clean_default_config() => Session.Dispatch(() =>
        {
            var h = NewWindow();

            TextBlock conflict = h.Window.GetVisualDescendants().OfType<TextBlock>()
                .First(t => t.Name == "ConflictText");

            Assert.True(string.IsNullOrEmpty(conflict.Text));
        }, default);

        // The NES tab must not offer X/Y/L/R - see EmuSen_Input.md §5.1.
        [Fact]
        public Task The_NES_tab_offers_only_the_eight_buttons_that_pad_has() => Session.Dispatch(() =>
        {
            var h = NewWindow("NES");

            Assert.NotNull(RebindButtonFor(h.Window, "A", "Rebind Key"));
            Assert.NotNull(RebindButtonFor(h.Window, "Select", "Rebind Key"));
            Assert.Throws<InvalidOperationException>(() => RebindButtonFor(h.Window, "X", "Rebind Key"));
            Assert.Throws<InvalidOperationException>(() => RebindButtonFor(h.Window, "L", "Rebind Key"));
        }, default);

        [Fact]
        public Task The_SNES_tab_offers_all_twelve() => Session.Dispatch(() =>
        {
            var h = NewWindow("SNES");

            Assert.NotNull(RebindButtonFor(h.Window, "X", "Rebind Key"));
            Assert.NotNull(RebindButtonFor(h.Window, "L", "Rebind Key"));
            Assert.NotNull(RebindButtonFor(h.Window, "R", "Rebind Key"));
        }, default);

        // The whole point of separate tabs - see EmuSen_Input.md §5.1.
        [Fact]
        public Task Rebinding_on_one_console_leaves_the_other_alone() => Session.Dispatch(() =>
        {
            var h = NewWindow("NES");
            Key snesBefore = h.Bindings.For("SNES").ButtonToKey[PadButton.A];

            ClickAsUser(RebindButtonFor(h.Window, "A", "Rebind Key"));
            Press(h.Window, Key.K);

            Assert.Equal(Key.K, h.Bindings.For("NES").ButtonToKey[PadButton.A]);
            Assert.Equal(snesBefore, h.Bindings.For("SNES").ButtonToKey[PadButton.A]);
        }, default);

        // Two consoles may share a key, so a clash on one must not paint the other.
        [Fact]
        public Task The_same_key_on_two_consoles_is_not_a_conflict() => Session.Dispatch(() =>
        {
            var h = NewWindow("NES");

            TextBlock conflict = h.Window.GetVisualDescendants().OfType<TextBlock>()
                .First(t => t.Name == "ConflictText");

            // Every NES button defaults to the same key its SNES twin uses.
            Assert.True(string.IsNullOrEmpty(conflict.Text));
        }, default);

        // General is index 0, then one tab per console oldest-first - see EmuSen_Input.md §5.1.
        [Fact]
        public Task Tabs_are_General_then_the_consoles_oldest_first() => Session.Dispatch(() =>
        {
            var h = NewWindow();

            TabControl tabs = h.Window.GetVisualDescendants().OfType<TabControl>().First();
            string?[] headers = tabs.Items.OfType<TabItem>().Select(t => t.Header as string).ToArray();

            Assert.Equal(new[] { "General", "NES", "SNES" }, headers);
        }, default);

        // With no ROM loaded there is no console to prefer, so General stays selected.
        [Fact]
        public Task No_loaded_console_opens_on_General() => Session.Dispatch(() =>
        {
            var window = new InputSettingsWindow(new ControllerKeyBindings(Consoles), new GamepadBindings(Consoles),
                null!, new AppSettings(), new HotkeyBindingMap());
            window.Show();

            TabControl tabs = window.GetVisualDescendants().OfType<TabControl>().First();
            Assert.Equal(0, tabs.SelectedIndex);
        }, default);

        [Fact]
        public Task Reset_to_defaults_restores_every_row() => Session.Dispatch(() =>
        {
            var h = NewWindow();

            ClickAsUser(RebindButtonFor(h.Window, "Up", "Rebind Key"));
            Press(h.Window, Key.K);
            Assert.Equal(Key.K, h.Keys.ButtonToKey[PadButton.Up]);

            ClickAsUser(ButtonByContent(h.Window, "Reset to Defaults"));

            Assert.Equal(Key.Up, h.Keys.ButtonToKey[PadButton.Up]);
            Assert.Equal("Up", LabelTextFor(h.Window, "Up", 1));
            Assert.Equal(Key.Tab, h.Hotkeys.ActionToKey[HotkeyAction.FastForward]);
        }, default);
    }
}
