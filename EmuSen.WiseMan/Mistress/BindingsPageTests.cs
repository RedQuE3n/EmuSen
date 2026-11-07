using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.Threading;
using EmuSen.Endymion.Input;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Pages;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;
using EmuSen.Mistress.Library;

namespace EmuSen.WiseMan.Mistress
{
    // What the binding window did, through Settings' bindings pages since 2026-11-03: the keyboard's keys, the hotkeys, the players, the drawing lit by what is held, the tester and the conflicts - see EmuSen_Input.md §10.6.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class BindingsPageTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(BindingsPageTests).GetTypeInfo().Assembly);

        private static readonly BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenBindingsPage", Guid.NewGuid().ToString("N"));

        public BindingsPageTests()
        {
            Directory.CreateDirectory(Path.Combine(_root, "Roms"));
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private sealed record Rig(MainWindow Window, PadDriver Pad)
        {
            public SettingsScreen Model => Window.SettingsShown!;
            public SettingsView View => (SettingsView)typeof(MainWindow).GetField("_settings", Hidden)!.GetValue(Window)!;
            public ControllerDiagram Diagram => View.Diagram!;
            public ControllerKeyBindings Keys => (ControllerKeyBindings)typeof(MainWindow).GetField("_keyBindings", Hidden)!.GetValue(Window)!;
            public HotkeyBindingMap Hotkeys => (HotkeyBindingMap)typeof(MainWindow).GetField("_hotkeyBindings", Hidden)!.GetValue(Window)!;
            public GamepadBindings Pads => (GamepadBindings)typeof(MainWindow).GetField("_gamepadBindings", Hidden)!.GetValue(Window)!;
            public string? Pane => string.Join("\n", Model.PaneOf(Window.SettingsLayoutShown!.Content?.Focus ?? Window.SettingsLayoutShown.Focused ?? "")?.Lines.Select(l => l.Text) ?? []);

            public void Key(Key key) => Window.KeyPress(key, RawInputModifiers.None, PhysicalKey.None, null);
            public void KeyUp(Key key) => Window.KeyRelease(key, RawInputModifiers.None, PhysicalKey.None, null);

            public void Focus(PadControl control)
            {
                Model.FocusBinding(control);
                View.Fill();
            }

            // A the way a player chooses: the screen's own A on the focused part.
            public void Choose()
            {
                Window.SettingsLayoutShown!.Press(PadGlyphButton.South);
                Dispatcher.UIThread.RunJobs();
            }
        }

        // The big screen's Settings on a page, the console chosen, as the player reaches it.
        private Rig Open(string page, string console, Action<AppSettings>? settings = null)
        {
            var app = new AppSettings { RomDirectory = Path.Combine(_root, "Roms"), LibraryView = AppSettings.LibraryList, BigScreen = true };
            settings?.Invoke(app);
            app.Save();
            var window = new MainWindow { Width = 1280, Height = 800 };
            window.Show();
            var pad = new PadDriver(window) { Pad = { Name = "Test Pad" } };
            Dispatcher.UIThread.RunJobs();
            window.ShowSettings(SettingsScreen.Controllers, page);
            if (window.SettingsShown!.Binding is not null) window.SettingsShown.ChooseBindingConsole(console);
            var rig = new Rig(window, pad);
            if (rig.Model.Binding is not null) rig.View.Fill();
            window.SettingsLayoutShown!.Focus();
            return rig;
        }

        // --- The keyboard's keys (the binding window's rows and their capture) ---

        [Theory]
        [InlineData(Key.Enter)]
        [InlineData(Key.Space)]
        [InlineData(Key.Up)]
        [InlineData(Key.Left)]
        [InlineData(Key.K)]
        public Task A_key_rebinds_the_focused_control_and_the_pane_says_it(Key key) => Session.Dispatch(() =>
        {
            Rig rig = Open("keys", "SNES");
            rig.Focus(PadControl.Up);
            rig.Choose();
            Assert.True(rig.Model.Binding!.Capturing);
            rig.Key(key);
            Assert.False(rig.Model.Binding!.Capturing, "the key that answered does not start another capture");
            Assert.Equal(key, rig.Keys.For("SNES").ButtonToKey[PadControl.Up]);
            Assert.Equal(key, ControllerKeyBindings.Load(["SNES"]).For("SNES").ButtonToKey[PadControl.Up]);
            Assert.Contains($"Bound to the key {key}.", rig.Pane);
            Assert.Equal(key.ToString(), rig.Diagram.BindingOf("Up").Key);
            rig.Window.Close();
        }, default);

        [Fact]
        public Task Escape_cancels_a_rebind_and_a_bare_modifier_is_no_binding() => Session.Dispatch(() =>
        {
            Rig rig = Open("keys", "SNES");
            rig.Focus(PadControl.A);
            Key before = rig.Keys.For("SNES").ButtonToKey[PadControl.A];
            rig.Choose();
            rig.Key(Key.LeftShift);
            Assert.True(rig.Model.Binding!.Capturing);
            rig.Key(Key.Escape);
            Assert.False(rig.Model.Binding!.Capturing);
            Assert.Equal(before, rig.Keys.For("SNES").ButtonToKey[PadControl.A]);
            Assert.NotNull(rig.Window.SettingsShown);
            rig.Window.Close();
        }, default);

        [Fact]
        public Task Taking_a_key_from_another_button_unbinds_that_button_and_X_clears_one() => Session.Dispatch(() =>
        {
            Rig rig = Open("keys", "SNES");
            rig.Focus(PadControl.A);
            rig.Choose();
            rig.Key(Key.Z);
            Assert.Equal(Key.Z, rig.Keys.For("SNES").ButtonToKey[PadControl.A]);
            Assert.False(rig.Keys.For("SNES").ButtonToKey.ContainsKey(PadControl.B));
            rig.Focus(PadControl.B);
            Assert.Contains("Not bound to a key.", rig.Pane);

            rig.Focus(PadControl.Select);
            Assert.True(rig.Window.SettingsLayoutShown!.Press(PadGlyphButton.West));
            Assert.False(rig.Keys.For("SNES").ButtonToKey.ContainsKey(PadControl.Select));
            Assert.False(ControllerKeyBindings.Load(["SNES"]).For("SNES").ButtonToKey.ContainsKey(PadControl.Select));

            // Y resets the focused control to its key.
            Assert.True(rig.Window.SettingsLayoutShown!.Press(PadGlyphButton.North));
            Assert.Equal(DefaultPadKeyMap.Bindings()[PadControl.Select], rig.Keys.For("SNES").ButtonToKey[PadControl.Select]);
            rig.Window.Close();
        }, default);

        [Fact]
        public Task Each_console_offers_its_own_controls_and_rebinding_one_leaves_the_other_alone() => Session.Dispatch(() =>
        {
            Rig rig = Open("keys", "NES");
            Assert.Contains(PadControl.A, rig.Model.ControlsOf("NES"));
            Assert.DoesNotContain(PadControl.X, rig.Model.ControlsOf("NES"));
            Assert.DoesNotContain(PadControl.L, rig.Model.ControlsOf("NES"));
            Assert.Contains(PadControl.X, rig.Model.ControlsOf("SNES"));
            Assert.Contains(PadControl.R, rig.Model.ControlsOf("SNES"));
            Key snes = rig.Keys.For("SNES").ButtonToKey[PadControl.A];
            rig.Focus(PadControl.A);
            rig.Choose();
            rig.Key(Key.K);
            Assert.Equal(Key.K, rig.Keys.For("NES").ButtonToKey[PadControl.A]);
            Assert.Equal(snes, rig.Keys.For("SNES").ButtonToKey[PadControl.A]);
            rig.Window.Close();
        }, default);

        [Fact]
        public Task A_clash_is_a_notice_in_the_pane_and_the_same_key_on_two_consoles_is_none() => Session.Dispatch(() =>
        {
            Rig rig = Open("keys", "NES");
            Assert.Empty(rig.Model.Conflicts("NES"));
            Assert.Empty(rig.Model.Conflicts("SNES"));
            Assert.DoesNotContain("Conflict", rig.Pane);
            // A hand-edited file can still arrive with a clash, past Rebind's guard.
            rig.Keys.For("NES").ButtonToKey[PadControl.A] = rig.Keys.For("NES").ButtonToKey[PadControl.B];
            rig.View.Fill();
            Assert.Contains("Conflict on the NES", rig.Pane);
            Assert.True(rig.Pane!.Contains("A and B") || rig.Pane.Contains("B and A"), rig.Pane);
            Assert.Equal(SettingsTone.Attention, rig.Model.PaneOf(rig.Window.SettingsLayoutShown!.Content!.Focus)!.Lines.Last().Tone);
            Assert.Empty(rig.Model.Conflicts("SNES"));
            rig.Window.Close();
        }, default);

        [Fact]
        public Task The_consoles_follow_the_player_in_release_order_and_the_page_opens_on_the_running_game_or_the_first() => Session.Dispatch(() =>
        {
            Rig rig = Open("bindings", "NES");
            IReadOnlyList<string> order = rig.Model.BindingTabs(rig.Model.Binding!);
            Assert.Equal("Player 1", order[0]);
            List<string> consoles = RuntimeCatalog.ConsoleNames.ToList();
            Assert.True(consoles.IndexOf("NES") < consoles.IndexOf("SNES"));
            Assert.True(order.ToList().IndexOf("Super Nintendo") > order.ToList().IndexOf("NES"), string.Join(", ", order));
            rig.Window.CloseSettings();
            typeof(MainWindow).GetMethod("ShowControllerBindings", Hidden)!.Invoke(rig.Window, null);
            Assert.NotNull(rig.Window.SettingsShown!.Binding);
            rig.Window.Close();
        }, default);

        [Fact]
        public Task Stick_directions_are_bound_to_keys_and_have_no_pad_button() => Session.Dispatch(() =>
        {
            Rig rig = Open("bindings", "N64");
            rig.Focus(PadControl.LeftStickUp);
            Assert.Contains("Read from the pad's own stick", rig.Pane);
            Assert.Equal("Left stick", rig.Model.BoundPad(rig.Model.Binding!, PadControl.LeftStickUp));
            Assert.Equal("Right stick", rig.Model.BoundPad(rig.Model.Binding!, PadControl.RightStickRight));
            rig.Choose();
            Assert.False(rig.Model.Binding!.Capturing, "a stick direction has no button to listen for");
            rig.Window.CloseSettings();

            rig = Open("keys", "N64");
            rig.Focus(PadControl.LeftStickUp);
            rig.Choose();
            rig.Key(Key.U);
            Assert.Equal(Key.U, rig.Keys.For("N64").ButtonToKey[PadControl.LeftStickUp]);
            rig.Window.Close();
        }, default);

        // --- Hotkeys ---

        [Fact]
        public Task A_hotkey_rebinds_its_row_and_takes_its_key_from_every_consoles_buttons() => Session.Dispatch(() =>
        {
            Rig rig = Open("hotkeys", "SNES");
            Assert.Equal("Hotkeys", rig.Model.Trail[^1]);
            rig.Model.Choose("hk:" + HotkeyAction.FastForward);
            Assert.Equal(HotkeyAction.FastForward, rig.Model.HotkeyListening);
            rig.Key(Key.F5);
            Assert.Equal(Key.F5, rig.Hotkeys.ActionToKey[HotkeyAction.FastForward]);
            Assert.Equal("F5", rig.Model.Rows.Single(r => r.Key == "hk:" + HotkeyAction.FastForward).Status);

            rig.Model.Choose("hk:" + HotkeyAction.Rewind);
            rig.Key(Key.X);
            Assert.Equal(Key.X, rig.Hotkeys.ActionToKey[HotkeyAction.Rewind]);
            Assert.False(rig.Keys.For("SNES").ButtonToKey.ContainsKey(PadControl.A));
            Assert.False(rig.Keys.For("NES").ButtonToKey.ContainsKey(PadControl.A));
            Assert.Equal(Key.X, HotkeyBindingMap.Load().ActionToKey[HotkeyAction.Rewind]);

            // And a game button taking a hotkey's key takes it from the hotkey.
            rig.Window.CloseSettings();
            rig = Open("keys", "SNES");
            rig.Focus(PadControl.Start);
            rig.Choose();
            rig.Key(Key.F5);
            Assert.False(rig.Hotkeys.ActionToKey.ContainsKey(HotkeyAction.FastForward));
            rig.Window.Close();
        }, default);

        // A file written before an action existed leaves it unbound - see EmuSen_Settings_Reference.md §4.18.
        [Fact]
        public Task An_older_hotkey_file_gains_the_defaults_it_predates_but_never_over_a_key_since_bound() => Session.Dispatch(() =>
        {
            var stale = new HotkeyBindingMap();
            stale.ActionToKey.Remove(HotkeyAction.ExitToLibrary);
            stale.Rebind(HotkeyAction.TogglePause, Key.F2);
            stale.Save();
            var loaded = HotkeyBindingMap.Load();
            Assert.Equal(Key.Escape, loaded.ActionToKey[HotkeyAction.ExitToLibrary]);
            Assert.Equal(Key.F2, loaded.ActionToKey[HotkeyAction.TogglePause]);

            stale = new HotkeyBindingMap();
            stale.ActionToKey.Remove(HotkeyAction.ExitToLibrary);
            stale.Rebind(HotkeyAction.SaveState, Key.Escape);
            stale.Save();
            loaded = HotkeyBindingMap.Load();
            Assert.Equal(Key.Escape, loaded.ActionToKey[HotkeyAction.SaveState]);
            Assert.False(loaded.ActionToKey.ContainsKey(HotkeyAction.ExitToLibrary));
            Assert.True(loaded.TryGetAction(Key.Escape, out HotkeyAction owner));
            Assert.Equal(HotkeyAction.SaveState, owner);
        }, default);

        [Fact]
        public Task Reset_every_binding_restores_the_keys_the_pads_and_the_hotkeys() => Session.Dispatch(() =>
        {
            Rig rig = Open("keys", "SNES");
            rig.Focus(PadControl.Up);
            rig.Choose();
            rig.Key(Key.K);
            rig.Pads.For("SNES").Rebind(PadButton.A, SDL.GamepadButton.North);
            rig.Window.CloseSettings();
            rig.Window.ShowSettings(SettingsScreen.Controllers);
            rig.Window.SettingsShown!.Choose("resetbindings");
            Assert.Equal(Key.Up, rig.Keys.For("SNES").ButtonToKey[PadControl.Up]);
            Assert.Equal(SDL.GamepadButton.East, rig.Pads.For("SNES").ButtonToPad[PadButton.A]);
            Assert.Equal(Key.Tab, rig.Hotkeys.ActionToKey[HotkeyAction.FastForward]);
            Assert.Equal(Key.Up, ControllerKeyBindings.Load(["SNES"]).For("SNES").ButtonToKey[PadControl.Up]);
            rig.Window.Close();
        }, default);

        // --- Players and the drawing ---

        [Fact]
        public Task Each_console_offers_its_ports_players_and_player_2_gets_buttons_of_its_own() => Session.Dispatch(() =>
        {
            Rig rig = Open("bindings", "SNES");
            Assert.True(rig.Model.PlayersOf("SNES") >= 2);
            Assert.Equal(1, rig.Model.PlayersOf("GB"));
            rig.Focus(PadControl.B);
            rig.Window.SettingsLayoutShown!.Press(PadGlyphButton.West);
            Assert.Equal(2, rig.Model.Binding!.Player);
            Assert.Equal("Player 2", rig.Window.SettingsLayoutShown!.Content!.Tabs[0]);
            rig.Model.Choose(SettingsScreen.BindingKey(PadControl.B));
            rig.Model.Rebind(SDL.GamepadButton.North);
            Assert.True(rig.Pads.HasOwn("SNES", 2));
            Assert.Equal(SDL.GamepadButton.North, rig.Pads.For("SNES", 2).ButtonToPad[PadButton.B]);
            Assert.Equal(SDL.GamepadButton.South, rig.Pads.For("SNES", 1).ButtonToPad[PadButton.B]);
            rig.Window.Close();
        }, default);

        [Fact]
        public Task An_adapters_players_are_offered_and_each_ones_pad_is_drawn() => Session.Dispatch(() =>
        {
            var config = ConsolesConfig.Load();
            config.SetValue("Genesis", "pad1", "md.teamplayer6");
            config.SetValue("Genesis", "pad2", "md.pad3");
            config.Save();
            Rig rig = Open("bindings", "Genesis");
            Assert.Equal(5, rig.Model.PlayersOf("Genesis"));
            foreach (var (player, layout, regions) in new[] { (3, ControllerLayout.GenesisSixButton, 12), (4, ControllerLayout.GenesisSixButton, 12), (5, ControllerLayout.Genesis, 8) })
            {
                while (rig.Model.Binding!.Player != player) rig.Model.StepBindingPlayer();
                rig.View.Fill();
                Assert.Equal(layout, rig.Diagram.Layout);
                Assert.Equal(regions, rig.Diagram.Regions.Count);
            }
            rig.Window.Close();
        }, default);

        [Fact]
        public Task Every_control_each_console_uses_has_a_region_and_every_region_a_control() => UiTest.Run(() =>
        {
            foreach (string console in RuntimeCatalog.ConsoleNames)
            {
                var diagram = new ControllerDiagram { Layout = ControllerDiagrams.LayoutFor(console) };
                IReadOnlyList<PadControl> controls = RuntimeCatalog.ControlsFor(console);
                var regions = diagram.Regions.Select(r => r.Id).ToHashSet();
                bool knobs = diagram.Layout == ControllerLayout.DualShock;
                foreach (PadControl control in controls.Where(c => !(knobs && c.ToString().Contains("Stick", StringComparison.Ordinal) && c is not (PadControl.L3 or PadControl.R3))))
                    Assert.True(regions.Contains(ControllerDiagrams.RegionFor(console, control)), $"{console} has no region for {control}");
                foreach (string region in regions.Where(r => !(knobs && r == "Analog")))
                    Assert.True(ControllerDiagrams.ControlFor(console, region, controls) is not null, $"{console}'s {region} stands for no control");
            }
        });

        [Theory]
        [InlineData("Saturn", "B=A A=B R=C Y=X X=Y L=Z L2=L R2=R Start=Start Up=Up Down=Down Left=Left Right=Right")]
        [InlineData("Dreamcast", "B=A A=B Y=X X=Y L2=LT R2=RT Start=Start Up=Up Down=Down Left=Left Right=Right")]
        public Task The_hosted_consoles_buttons_are_the_cores_documented_retropad_controls(string console, string table) => UiTest.Run(() =>
        {
            var expected = table.Split(' ').Select(p => p.Split('=')).ToDictionary(p => Enum.Parse<PadControl>(p[0]), p => p[1]);
            foreach (var (control, region) in expected) Assert.Equal(region, ControllerDiagrams.RegionFor(console, control));
            var diagram = new ControllerDiagram { Layout = ControllerDiagrams.LayoutFor(console) };
            Assert.Equal(expected.Values.Order(), diagram.Regions.Select(r => r.Id).Order());
        });

        [Fact]
        public Task The_cross_moves_on_the_drawing_by_where_the_buttons_are_drawn_and_A_listens() => Session.Dispatch(() =>
        {
            Rig rig = Open("bindings", "SNES");
            rig.Focus(PadControl.B);
            rig.Window.SettingsLayoutShown!.Move(0, -1);
            Assert.Equal(PadControl.X, rig.Model.Binding!.Focus);
            rig.Window.SettingsLayoutShown!.Move(-1, 0);
            Assert.Equal(PadControl.Y, rig.Model.Binding!.Focus);
            Assert.Equal("Y", rig.Diagram.SelectedRegion);
            rig.Choose();
            Assert.True(rig.Model.Binding!.Capturing);
            Assert.Contains("Press the button for Y", rig.Pane);
            rig.Key(Key.Escape);
            Assert.False(rig.Model.Binding!.Capturing);
            rig.Window.Close();
        }, default);

        [Fact]
        public Task A_pad_press_or_a_bound_key_lights_its_region_and_letting_go_clears_it_but_not_while_listening() => Session.Dispatch(() =>
        {
            // The tester takes the presses, so none moves the focus or starts a binding.
            Rig rig = Open("bindings", "SNES");
            rig.Model.ToggleTesting();
            rig.View.Fill();
            foreach (PadButton button in RuntimeCatalog.PadOf("SNES").Buttons)
            {
                SDL.GamepadButton bound = rig.Pads.For("SNES").ButtonToPad[button];
                rig.Pad.Pad.Press(bound);
                rig.Pad.Tick();
                Assert.Equal(new[] { button.ToString() }, rig.Diagram.Pressed.ToArray());
                rig.Pad.Pad.Release(bound);
                rig.Pad.Tick();
                Assert.Empty(rig.Diagram.Pressed);
            }
            rig.Key(Key.X);
            Assert.True(rig.Diagram.IsPressed("A"));
            rig.KeyUp(Key.X);
            Assert.False(rig.Diagram.IsPressed("A"));

            rig.Model.StopTesting();
            rig.Focus(PadControl.L);
            rig.Choose();
            rig.Pad.Pad.Press(SDL.GamepadButton.East);
            rig.Pad.Tick();
            Assert.Empty(rig.Diagram.Pressed);
            rig.Window.Close();
        }, default);

        [Fact]
        public Task The_stick_moves_its_knob_and_lights_its_directions_and_a_key_pushes_it_all_the_way() => Session.Dispatch(() =>
        {
            Rig rig = Open("bindings", "N64");
            rig.Model.ToggleTesting();
            rig.View.Fill();
            Dispatcher.UIThread.RunJobs();
            rig.Window.UpdateLayout();
            Point rest = rig.Diagram.StickKnobCentre("Stick")!.Value;
            rig.Pad.Pad.SetAxis(SDL.GamepadAxis.LeftX, 0.8);
            rig.Pad.Pad.SetAxis(SDL.GamepadAxis.LeftY, -0.3);
            rig.Pad.Tick();
            Assert.Equal(0.8, rig.Diagram.StickPosition("Stick").X, 2);
            Assert.True(rig.Diagram.IsPressed("StickRight"));
            Assert.False(rig.Diagram.IsPressed("StickUp"));
            rig.Pad.Pad.SetAxis(SDL.GamepadAxis.LeftX, 0);
            rig.Pad.Pad.SetAxis(SDL.GamepadAxis.LeftY, 0);
            rig.Pad.Tick();
            Assert.Equal(rest, rig.Diagram.StickKnobCentre("Stick")!.Value);
            Key up = rig.Keys.For("N64").ButtonToKey[PadControl.LeftStickUp];
            rig.Key(up);
            rig.Pad.Tick();
            Assert.Equal(new Vector(0, -1), rig.Diagram.StickPosition("Stick"));
            Assert.True(rig.Diagram.IsPressed("StickUp"));
            rig.Window.Close();
        }, default);

        [Fact]
        public Task Z_is_the_left_trigger_and_a_button_bound_beside_it_is_taken_off_by_the_trigger() => Session.Dispatch(() =>
        {
            Rig rig = Open("bindings", "N64");
            Assert.Equal("Left Trigger", rig.Model.BoundPad(rig.Model.Binding!, PadControl.L2));
            rig.Model.ToggleTesting();
            rig.Pad.Pad.SetAxis(SDL.GamepadAxis.LeftTrigger, 0.8);
            rig.Pad.Tick();
            Assert.True(rig.Diagram.IsPressed("Z"));
            rig.Pad.Pad.SetAxis(SDL.GamepadAxis.LeftTrigger, 0);
            rig.Pad.Tick();
            rig.Model.StopTesting();
            rig.Focus(PadControl.L2);
            rig.Choose();
            rig.Pad.Tick();
            rig.Pad.Pad.Press(SDL.GamepadButton.LeftShoulder);
            rig.Pad.Tick();
            rig.Pad.Pad.Release(SDL.GamepadButton.LeftShoulder);
            rig.Pad.Tick();
            Assert.Equal(SDL.GamepadButton.LeftShoulder, rig.Pads.For("N64").ButtonToPad[PadButton.L2]);
            Assert.Equal("Left Shoulder or Left Trigger", rig.Model.BoundPad(rig.Model.Binding!, PadControl.L2));
            rig.Choose();
            rig.Pad.Tick();
            rig.Pad.Pad.SetAxis(SDL.GamepadAxis.LeftTrigger, 1);
            rig.Pad.Tick();
            Assert.False(rig.Pads.For("N64").ButtonToPad.ContainsKey(PadButton.L2));
            Assert.Equal("Left Trigger", rig.Model.BoundPad(rig.Model.Binding!, PadControl.L2));
            Assert.False(rig.Model.Binding!.Capturing);
            rig.Window.Close();
        }, default);

        // Select tries the buttons: every press lights and none moves or rebinds, until B is held for a second or Escape is pressed.
        [Fact]
        public Task Testing_takes_every_press_until_B_is_held() => Session.Dispatch(() =>
        {
            Rig rig = Open("bindings", "SNES");
            var now = TimeSpan.Zero;
            rig.Window.TesterClock = () => now;
            rig.Focus(PadControl.Start);
            Assert.True(rig.Window.SettingsLayoutShown!.Press(PadGlyphButton.Select));
            Assert.True(rig.Model.Binding!.Testing);
            Assert.Contains("Testing", rig.Pane);
            Assert.Equal(["Hold to stop"], rig.Window.SettingsLayoutShown!.Hints.Select(h => h.Label));
            rig.Pad.Tap(SDL.GamepadButton.South);
            rig.Pad.Tap(SDL.GamepadButton.DPadRight);
            Assert.False(rig.Model.Binding!.Capturing);
            Assert.Equal(PadControl.Start, rig.Model.Binding!.Focus);

            rig.Pad.Pad.Press(SDL.GamepadButton.East);
            rig.Pad.Tick();
            Assert.True(rig.Diagram.IsPressed("A"));
            Assert.Contains("Held: A", rig.Pane);
            Assert.NotNull(rig.Window.SettingsShown);
            now += TimeSpan.FromSeconds(0.5);
            rig.Pad.Tick();
            Assert.True(rig.Model.Binding!.Testing);
            now += MainWindow.TestExitHold;
            rig.Pad.Tick();
            Assert.False(rig.Model.Binding!.Testing);
            rig.Pad.Pad.Release(SDL.GamepadButton.East);
            rig.Pad.Tick();

            // The keyboard: Enter is Start's key, so it lights Start and rebinds nothing; Escape stops.
            rig.Model.ToggleTesting();
            rig.Key(Key.Enter);
            Assert.True(rig.Diagram.IsPressed("Start"));
            Assert.False(rig.Model.Binding!.Capturing);
            rig.KeyUp(Key.Enter);
            rig.Key(Key.Escape);
            Assert.False(rig.Model.Binding!.Testing);
            Assert.NotNull(rig.Window.SettingsShown);
            rig.Window.Close();
        }, default);

        [Fact]
        public Task The_deadzone_and_the_mirror_are_controllers_rows_that_reach_the_pads() => Session.Dispatch(() =>
        {
            Rig rig = Open("bindings", "SNES");
            rig.Window.CloseSettings();
            rig.Window.ShowSettings(SettingsScreen.Controllers);
            SettingsScreen s = rig.Window.SettingsShown!;
            var gamepad = (GamepadManager)typeof(MainWindow).GetField("_gamepad", Hidden)!.GetValue(rig.Window)!;
            double before = AppSettings.Load().StickDeadzone;
            s.Step("deadzone", 1);
            Assert.Equal(before + 0.05, AppSettings.Load().StickDeadzone, 3);
            Assert.Equal(before + 0.05, gamepad.StickDeadzone, 3);
            Assert.Equal(SettingsScreen.Off, s.Rows.Single(r => r.Key == "mirror").Status);
            s.Step("mirror", 1);
            Assert.True(AppSettings.Load().MirrorPlayer1ToPlayer2);
            rig.Window.Close();
        }, default);

        // The drawing and the tabs are named for assistive technology, as the binding window's buttons were.
        [Fact]
        public Task The_drawing_and_its_tabs_are_named() => Session.Dispatch(() =>
        {
            Rig rig = Open("bindings", "SNES");
            Assert.Equal("SNES controller", Avalonia.Automation.AutomationProperties.GetName(rig.Diagram));
            Assert.All(rig.Window.SettingsLayoutShown!.Content!.Tabs, t => Assert.False(string.IsNullOrWhiteSpace(t)));
            rig.Window.Close();
        }, default);
    }
}
