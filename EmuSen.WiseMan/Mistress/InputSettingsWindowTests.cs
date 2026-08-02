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
using EmuSen.Cores.Nintendo.Venus.Controllers;
using EmuSen.Mistress.Input;
using EmuSen.Nehellania.Input;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;

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

        private sealed class Harness
        {
            public required InputSettingsWindow Window { get; init; }
            public required ControllerKeyMap Keys { get; init; }
            public required HotkeyBindingMap Hotkeys { get; init; }
        }

        private static Harness NewWindow()
        {
            var keys = new ControllerKeyMap();
            var hotkeys = new HotkeyBindingMap();
            var window = new InputSettingsWindow(keys, new GamepadBindingMap(), null!, new AppSettings(), hotkeys);
            window.Show();
            return new Harness { Window = window, Keys = keys, Hotkeys = hotkeys };
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

            Assert.Equal(key, h.Keys.ButtonToKey[SnesButton.Up]);
            Assert.Equal(key.ToString(), LabelTextFor(h.Window, "Up", 1));
        }, default);

        [Fact]
        public Task Escape_cancels_a_rebind_and_restores_the_prompt() => Session.Dispatch(() =>
        {
            var h = NewWindow();
            Key before = h.Keys.ButtonToKey[SnesButton.A];

            Button rebind = RebindButtonFor(h.Window, "A", "Rebind Key");
            ClickAsUser(rebind);
            Press(h.Window, Key.Escape);

            Assert.Equal(before, h.Keys.ButtonToKey[SnesButton.A]);
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
            Assert.Equal(Key.Space, h.Keys.ButtonToKey[SnesButton.Start]);
        }, default);

        // Two SNES buttons must never share one key.
        [Fact]
        public Task Taking_a_key_from_another_button_unbinds_that_button() => Session.Dispatch(() =>
        {
            var h = NewWindow();

            ClickAsUser(RebindButtonFor(h.Window, "A", "Rebind Key"));
            Press(h.Window, Key.Z); // Z was B's default

            Assert.Equal(Key.Z, h.Keys.ButtonToKey[SnesButton.A]);
            Assert.False(h.Keys.ButtonToKey.ContainsKey(SnesButton.B));
            Assert.Equal("(unbound)", LabelTextFor(h.Window, "B", 1));
        }, default);

        [Fact]
        public Task Clearing_a_binding_unbinds_it() => Session.Dispatch(() =>
        {
            var h = NewWindow();

            ClickAsUser(RebindButtonFor(h.Window, "Select", "Clear"));

            Assert.False(h.Keys.ButtonToKey.ContainsKey(SnesButton.Select));
            Assert.Equal("(unbound)", LabelTextFor(h.Window, "Select", 1));
        }, default);

        // Hotkeys share the game-button listener.
        [Fact]
        public Task Rebinding_a_hotkey_updates_the_row_and_the_map() => Session.Dispatch(() =>
        {
            var h = NewWindow();

            ClickAsUser(RebindButtonFor(h.Window, "Fast Forward", "Rebind Key"));
            Press(h.Window, Key.F5);

            Assert.Equal(Key.F5, h.Hotkeys.ActionToKey[HotkeyAction.FastForward]);
            Assert.Equal("F5", LabelTextFor(h.Window, "Fast Forward", 1));
        }, default);

        // The two maps must police each other, not just themselves.
        [Fact]
        public Task A_hotkey_taking_a_game_buttons_key_unbinds_the_game_button() => Session.Dispatch(() =>
        {
            var h = NewWindow();

            ClickAsUser(RebindButtonFor(h.Window, "Rewind", "Rebind Key"));
            Press(h.Window, Key.X); // X was A's default

            Assert.Equal(Key.X, h.Hotkeys.ActionToKey[HotkeyAction.Rewind]);
            Assert.False(h.Keys.ButtonToKey.ContainsKey(SnesButton.A));
        }, default);

        // A hand-edited config can still arrive with a clash.
        [Fact]
        public Task A_config_that_already_conflicts_is_reported() => Session.Dispatch(() =>
        {
            var keys = new ControllerKeyMap();
            var hotkeys = new HotkeyBindingMap();
            // Straight into the dictionary, bypassing Rebind's guard.
            keys.ButtonToKey[SnesButton.A] = keys.ButtonToKey[SnesButton.B];

            var window = new InputSettingsWindow(keys, new GamepadBindingMap(), null!, new AppSettings(), hotkeys);
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

        [Fact]
        public Task Reset_to_defaults_restores_every_row() => Session.Dispatch(() =>
        {
            var h = NewWindow();

            ClickAsUser(RebindButtonFor(h.Window, "Up", "Rebind Key"));
            Press(h.Window, Key.K);
            Assert.Equal(Key.K, h.Keys.ButtonToKey[SnesButton.Up]);

            ClickAsUser(ButtonByContent(h.Window, "Reset to Defaults"));

            Assert.Equal(Key.Up, h.Keys.ButtonToKey[SnesButton.Up]);
            Assert.Equal("Up", LabelTextFor(h.Window, "Up", 1));
            Assert.Equal(Key.Tab, h.Hotkeys.ActionToKey[HotkeyAction.FastForward]);
        }, default);
    }
}
