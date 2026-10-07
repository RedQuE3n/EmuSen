using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Cores;
using EmuSen.Endymion.Input;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;

namespace EmuSen.WiseMan.Mistress
{
    // The controller bindings window's drawings: every control has a region, a click or the pad chooses it, and what is pressed lights up - see EmuSen_Settings_Reference.md §4.81.
    [Collection(TestCollections.ProcessGlobals)]
    public class ControllerBindingsDiagramTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenBindingsDiagramTests", Guid.NewGuid().ToString("N"));

        public ControllerBindingsDiagramTests()
        {
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private static readonly string[] Consoles = CoreCatalog.ConsolesInReleaseOrder.Select(c => c.Console).ToArray();

        private sealed record Rig(InputSettingsWindow Window, SimulatedPad Pad, GamepadManager Gamepad, ControllerKeyBindings Keys, GamepadBindings Pads, ControllerDiagram Diagram)
        {
            public void Poll()
            {
                Gamepad.Poll();
                Dispatcher.UIThread.RunJobs();
            }
        }

        private static Rig Open(string console, double width = 1280, double height = 800)
        {
            var pad = new SimulatedPad { Name = "Test Pad" };
            var gamepad = new GamepadManager(new GamepadBindingMap(), start: true, SimulatedPads.With(pad));
            // The consoles as the catalog lists them now, which a test that shows a development core has changed.
            string[] consoles = CoreCatalog.ConsolesInReleaseOrder.Select(c => c.Console).ToArray();
            var keys = new ControllerKeyBindings(consoles);
            var pads = new GamepadBindings(consoles);
            var window = new InputSettingsWindow(keys, pads, gamepad, new AppSettings(), new HotkeyBindingMap(), console) { Width = width, Height = height };
            window.Show();
            window.CaptureRenderedFrame();
            Dispatcher.UIThread.RunJobs();
            return new Rig(window, pad, gamepad, keys, pads, window.DiagramFor(console)!);
        }

        private static void Capture(InputSettingsWindow window) =>
            typeof(InputSettingsWindow).GetMethod("PollForPadButton", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);

        private static Point InWindow(Rig rig, string region) => rig.Diagram.TranslatePoint(rig.Diagram.PointIn(region)!.Value, rig.Window)!.Value;

        [Fact]
        public Task Every_control_each_console_uses_has_a_region_and_every_region_a_control() => UiTest.Run(() =>
        {
            foreach (string console in Consoles)
            {
                var diagram = new ControllerDiagram { Layout = ControllerDiagrams.LayoutFor(console) };
                IReadOnlyList<PadControl> controls = CoreCatalog.ControlsFor(console);
                var regions = diagram.Regions.Select(r => r.Id).ToHashSet();
                foreach (PadControl control in controls)
                    Assert.True(regions.Contains(ControllerDiagrams.RegionFor(console, control)), $"{console} has no region for {control}");
                foreach (string region in regions)
                    Assert.True(ControllerDiagrams.ControlFor(console, region, controls) is not null, $"{console}'s {region} stands for no control");
                Assert.Equal(regions.Count, diagram.Regions.Count);
            }
        });

        // With an adapter on a port the window offers each player its ports hold, and draws that player's pad - see EmuSen_Settings_Reference.md §4.102.
        [Fact]
        public Task The_genesis_tab_offers_an_adapters_players_and_draws_each_ones_pad() => UiTest.Run(() =>
        {
            EmuSen.Cores.Native.CoreDiscovery.UseDevelopment(false);
            try
            {
                var config = GraphicsConfig.Load();
                config.SetValue("Genesis", "pad1", "md.teamplayer6");
                config.SetValue("Genesis", "pad2", "md.pad3");
                config.Save();
                Rig rig = Open("Genesis");
                Dropdown player = rig.Window.GetVisualDescendants().OfType<Dropdown>().Single(d => d.Name == "PlayerSelector");
                Assert.Equal(new[] { "Player 1", "Player 2", "Player 3", "Player 4", "Player 5" }, player.Items.Cast<string>());
                foreach (var (name, layout, buttons) in new[] { ("Player 3", ControllerLayout.GenesisSixButton, 12), ("Player 4", ControllerLayout.GenesisSixButton, 12), ("Player 5", ControllerLayout.Genesis, 8) })
                {
                    player.SelectedItem = name;
                    Dispatcher.UIThread.RunJobs();
                    Assert.Equal(layout, rig.Diagram.Layout);
                    Assert.Equal(buttons, rig.Diagram.Regions.Count);
                }
                rig.Window.Close();
            }
            finally
            {
                EmuSen.Cores.Native.CoreDiscovery.UseDevelopment(null);
            }
        });

        // The Genesis's drawing is the pad its port is set to: every button of that pad has a region named as the engine names the button, hit where it is drawn and bound by a click, and the drawing follows the player shown - see EmuSen_Settings_Reference.md §4.93.
        [Theory]
        [InlineData("md.pad3", ControllerLayout.Genesis, 8)]
        [InlineData("md.pad6", ControllerLayout.GenesisSixButton, 12)]
        public Task Every_button_of_the_genesis_pad_the_port_is_set_to_has_a_hit_region(string pad, ControllerLayout layout, int buttons) => UiTest.Run(() =>
        {
            EmuSen.Cores.Native.CoreDiscovery.UseDevelopment(false);
            try
            {
                var config = GraphicsConfig.Load();
                config.SetValue("Genesis", "pad1", pad);
                config.SetValue("Genesis", "pad2", pad == "md.pad3" ? "md.pad6" : "md.pad3");
                config.Save();
                Rig rig = Open("Genesis");
                Assert.Equal(layout, rig.Diagram.Layout);
                var controller = CoreCatalog.DiscoveredSystem("Genesis")!.Controllers.Single(c => c.Id == pad);
                Assert.Equal(buttons, controller.Buttons.Count);
                Assert.Equal(controller.Buttons.Select(b => b.Label).Order(), rig.Diagram.Regions.Select(r => r.Id).Order());
                IReadOnlyList<PadControl> controls = CoreCatalog.ControlsFor("Genesis");
                foreach (var button in controller.Buttons)
                {
                    PadControl control = PadControls.For(new[] { button.Control!.Value }, Array.Empty<PadAxis>()).Single();
                    string region = ControllerDiagrams.RegionFor("Genesis", control);
                    Assert.Equal(button.Label, region);
                    Assert.Equal(control, ControllerDiagrams.ControlFor("Genesis", region, controls));
                    Assert.Equal(region, rig.Diagram.RegionAt(rig.Diagram.PointIn(region)!.Value));

                    rig.Window.MouseDown(InWindow(rig, region), MouseButton.Left);
                    rig.Window.MouseUp(InWindow(rig, region), MouseButton.Left);
                    Assert.NotEqual(PadCapture.None, rig.Window.Capturing);
                    Assert.Equal(region, rig.Diagram.SelectedRegion);
                    Assert.Equal("Press a button", rig.Diagram.BindingOf(region).Pad);
                    rig.Window.KeyPress(Key.Escape, RawInputModifiers.None, PhysicalKey.None, null);
                    Assert.Equal(PadCapture.None, rig.Window.Capturing);
                }

                Dropdown player = rig.Window.GetVisualDescendants().OfType<Dropdown>().Single(d => d.Name == "PlayerSelector");
                player.SelectedItem = "Player 2";
                Dispatcher.UIThread.RunJobs();
                Assert.Equal(layout == ControllerLayout.Genesis ? ControllerLayout.GenesisSixButton : ControllerLayout.Genesis, rig.Diagram.Layout);
                rig.Window.Close();
            }
            finally
            {
                EmuSen.Cores.Native.CoreDiscovery.UseDevelopment(null);
            }
        });

        // A click anywhere a region is drawn hits it, and nowhere else does; the click starts that control's capture.
        [Theory]
        [InlineData("NES")]
        [InlineData("GB")]
        [InlineData("SNES")]
        [InlineData("N64")]
        public Task A_click_on_each_region_chooses_that_region(string console) => UiTest.Run(() =>
        {
            Rig rig = Open(console);
            Assert.Null(rig.Diagram.RegionAt(new Point(1, 1)));
            foreach (DiagramRegion region in rig.Diagram.Regions)
            {
                Point at = rig.Diagram.PointIn(region.Id)!.Value;
                Assert.Equal(region.Id, rig.Diagram.RegionAt(at));

                rig.Window.MouseDown(InWindow(rig, region.Id), MouseButton.Left);
                rig.Window.MouseUp(InWindow(rig, region.Id), MouseButton.Left);
                PadControl control = ControllerDiagrams.ControlFor(console, region.Id, CoreCatalog.ControlsFor(console))!.Value;
                Assert.NotEqual(PadCapture.None, rig.Window.Capturing);
                Assert.Equal("Press a key", rig.Diagram.BindingOf(region.Id).Key);
                Assert.Equal(region.Id, rig.Diagram.SelectedRegion);
                string? pad = rig.Diagram.BindingOf(region.Id).Pad;
                if (PadControls.IsButton(control, out _)) Assert.Equal("Press a button", pad);
                else Assert.True(pad is "Left stick" or "Right stick", $"{region.Id} shows {pad}");
                rig.Window.KeyPress(Key.Escape, RawInputModifiers.None, PhysicalKey.None, null);
                Assert.Equal(PadCapture.None, rig.Window.Capturing);
            }
            rig.Window.Close();
        });

        [Fact]
        public Task A_pad_press_lights_its_region_and_letting_go_clears_it() => UiTest.Run(() =>
        {
            Rig rig = Open("SNES");
            foreach (PadButton button in CoreCatalog.ButtonsFor("SNES"))
            {
                SDL.GamepadButton bound = rig.Pads.For("SNES").ButtonToPad[button];
                rig.Pad.Press(bound);
                rig.Poll();
                Assert.Equal(new[] { button.ToString() }, rig.Diagram.Pressed.ToArray());
                rig.Pad.Release(bound);
                rig.Poll();
                Assert.Empty(rig.Diagram.Pressed);
            }

            // The left stick stands in for the cross on a console that reads no stick.
            rig.Pad.SetAxis(SDL.GamepadAxis.LeftX, -0.9);
            rig.Poll();
            Assert.True(rig.Diagram.IsPressed("Left"));
            rig.Pad.SetAxis(SDL.GamepadAxis.LeftX, 0);
            rig.Poll();
            Assert.False(rig.Diagram.IsPressed("Left"));
            rig.Window.Close();
        });

        [Fact]
        public Task A_bound_key_lights_its_region_while_held() => UiTest.Run(() =>
        {
            Rig rig = Open("SNES");
            rig.Window.KeyPress(Key.X, RawInputModifiers.None, PhysicalKey.None, null);
            Assert.True(rig.Diagram.IsPressed("A"));
            rig.Window.KeyRelease(Key.X, RawInputModifiers.None, PhysicalKey.None, null);
            Assert.False(rig.Diagram.IsPressed("A"));

            // A key bound nowhere lights nothing.
            rig.Window.KeyPress(Key.P, RawInputModifiers.None, PhysicalKey.None, null);
            Assert.Empty(rig.Diagram.Pressed);
            rig.Window.Close();
        });

        [Fact]
        public Task The_stick_moves_its_marker_and_lights_its_directions() => UiTest.Run(() =>
        {
            Rig rig = Open("N64");
            Point rest = rig.Diagram.StickKnobCentre("Stick")!.Value;

            rig.Pad.SetAxis(SDL.GamepadAxis.LeftX, 0.8);
            rig.Pad.SetAxis(SDL.GamepadAxis.LeftY, -0.3);
            rig.Poll();
            Vector at = rig.Diagram.StickPosition("Stick");
            Assert.Equal(0.8, at.X, 2);
            Assert.Equal(-0.3, at.Y, 2);
            Point moved = rig.Diagram.StickKnobCentre("Stick")!.Value;
            Assert.True(moved.X > rest.X + 5 && moved.Y < rest.Y, $"the knob went from {rest} to {moved}");
            Assert.True(rig.Diagram.IsPressed("StickRight"));
            Assert.False(rig.Diagram.IsPressed("StickUp"));

            // The right stick is the C buttons, as Mars reads it.
            rig.Pad.SetAxis(SDL.GamepadAxis.RightY, -1);
            rig.Poll();
            Assert.True(rig.Diagram.IsPressed("CUp"));

            // Let go, the knob goes home; a key held for a direction pushes it all the way.
            rig.Pad.SetAxis(SDL.GamepadAxis.LeftX, 0);
            rig.Pad.SetAxis(SDL.GamepadAxis.LeftY, 0);
            rig.Pad.SetAxis(SDL.GamepadAxis.RightY, 0);
            rig.Poll();
            Assert.Equal(rest, rig.Diagram.StickKnobCentre("Stick")!.Value);
            Assert.Empty(rig.Diagram.Pressed);
            rig.Window.KeyPress(Key.I, RawInputModifiers.None, PhysicalKey.None, null);
            Assert.Equal(new Vector(0, -1), rig.Diagram.StickPosition("Stick"));
            Assert.True(rig.Diagram.IsPressed("StickUp"));
            rig.Window.Close();
        });

        [Fact]
        public Task Rebinding_by_the_drawing_updates_the_binding_and_the_labels() => UiTest.Run(() =>
        {
            Rig rig = Open("SNES");

            rig.Window.MouseDown(InWindow(rig, "Start"), MouseButton.Left);
            rig.Window.MouseUp(InWindow(rig, "Start"), MouseButton.Left);
            Assert.Equal(PadCapture.PadButton, rig.Window.Capturing);
            rig.Window.KeyPress(Key.K, RawInputModifiers.None, PhysicalKey.None, null);
            Assert.Equal(Key.K, rig.Keys.For("SNES").ButtonToKey[PadControl.Start]);
            Assert.Equal("K", rig.Diagram.BindingOf("Start").Key);
            Assert.Equal(PadCapture.None, rig.Window.Capturing);

            // The pad answers a capture begun on the drawing once the press that began it is let go.
            rig.Window.MouseDown(InWindow(rig, "X"), MouseButton.Left);
            rig.Window.MouseUp(InWindow(rig, "X"), MouseButton.Left);
            Capture(rig.Window);
            rig.Pad.Press(SDL.GamepadButton.LeftShoulder);
            Capture(rig.Window);
            Assert.Equal(SDL.GamepadButton.LeftShoulder, rig.Pads.For("SNES").ButtonToPad[PadButton.X]);
            Assert.Equal("Left Shoulder", rig.Diagram.BindingOf("X").Pad);
            Assert.NotEqual(PadCapture.Key, rig.Window.Capturing);
            rig.Pad.Release(SDL.GamepadButton.LeftShoulder);
            Capture(rig.Window);
            Assert.Equal(PadCapture.None, rig.Window.Capturing);

            // The list below the drawing shows the same bindings.
            var keyCells = rig.Window.GetVisualDescendants().OfType<TextBlock>().Select(t => t.Text).ToList();
            Assert.Contains("K", keyCells);
            rig.Window.Close();
        });

        // Every region of each drawing is reached from its first by the cross alone, through the router as a pad drives any window.
        [Theory]
        [InlineData("NES")]
        [InlineData("GB")]
        [InlineData("SNES")]
        [InlineData("N64")]
        public Task The_pad_alone_reaches_every_region(string console) => UiTest.Run(() =>
        {
            Rig rig = Open(console);
            rig.Diagram.Select(rig.Diagram.Regions[0].Id, NavigationMethod.Directional);
            Control root = PadWindowRouter.RootOf(rig.Window);
            HashSet<InputElement> reached = PadAudit.Reachable(root, b => PadWindowRouter.Send(rig.Window, b));
            string[] missing = rig.Diagram.Regions.Where(r => !reached.Contains(rig.Diagram.LabelOf(r.Id)!)).Select(r => r.Id).ToArray();
            Assert.Empty(missing);
            rig.Window.Close();
        });

        // On the drawing, the cross moves by where the buttons are drawn, not by where their labels stand, and A rebinds by pad or key.
        [Fact]
        public Task The_cross_moves_on_the_drawing_and_A_rebinds_the_region() => UiTest.Run(() =>
        {
            Rig rig = Open("SNES");
            rig.Diagram.Select("B", NavigationMethod.Directional);
            PadWindowRouter.Send(rig.Window, UiButton.Up);
            Assert.Equal("X", rig.Diagram.SelectedRegion);
            PadWindowRouter.Send(rig.Window, UiButton.Left);
            Assert.Equal("Y", rig.Diagram.SelectedRegion);

            PadWindowRouter.Send(rig.Window, UiButton.Accept);
            Assert.Equal(PadCapture.PadButton, rig.Window.Capturing);
            Assert.Equal("Press a button", rig.Diagram.BindingOf("Y").Pad);
            Assert.Equal("Press a key", rig.Diagram.BindingOf("Y").Key);
            rig.Window.KeyPress(Key.Escape, RawInputModifiers.None, PhysicalKey.None, null);
            Assert.Equal(PadCapture.None, rig.Window.Capturing);
            Assert.Equal("A", rig.Diagram.BindingOf("Y").Key);
            rig.Window.Close();
        });

        // On a big-screen sheet the footer names this window's buttons, then the tester's, and gives the old words back on close.
        [Fact]
        public Task On_a_sheet_the_footer_names_the_window_s_buttons_and_is_given_back() => UiTest.Run(() =>
        {
            Directory.CreateDirectory(Path.Combine(_root, "Roms"));
            new AppSettings { RomDirectory = Path.Combine(_root, "Roms"), LibraryView = AppSettings.LibraryList, BigScreen = true }.Save();
            var main = new MainWindow { Width = 1280, Height = 800 };
            main.Show();
            var layer = main.GetVisualDescendants().OfType<EmuSen.LunaP.Windowing.SheetLayer>().First(l => l.Name == "Sheets");
            string? before = layer.Hint;
            typeof(MainWindow).GetMethod("ShowControllerBindings", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(main, null);
            var bindings = (InputSettingsWindow)layer.Current!;
            Assert.Equal(PadHints.Face(InputSettingsWindow.PadHint), layer.Hint);
            bindings.SetTesting(true);
            Assert.Equal(PadHints.Face(InputSettingsWindow.TestingPadHint), layer.Hint);
            bindings.Close();
            Assert.Equal(before, layer.Hint);
            main.Close();
        });

        // The N64's Z is the left trigger by default, as a trigger is always its L2; a button bound beside it is shown too, and a trigger pulled while capturing takes the button off again.
        [Fact]
        public Task Z_is_the_left_trigger_and_a_button_can_be_bound_beside_it() => UiTest.Run(() =>
        {
            Rig rig = Open("N64");
            Assert.Equal("Left Trigger", rig.Diagram.BindingOf("Z").Pad);
            rig.Pad.SetAxis(SDL.GamepadAxis.LeftTrigger, 0.8);
            rig.Poll();
            Assert.True(rig.Diagram.IsPressed("Z"));
            rig.Pad.SetAxis(SDL.GamepadAxis.LeftTrigger, 0);
            rig.Poll();

            rig.Diagram.Select("Z", NavigationMethod.Directional);
            PadWindowRouter.Send(rig.Window, UiButton.Accept);
            Capture(rig.Window);
            rig.Pad.Press(SDL.GamepadButton.LeftShoulder);
            Capture(rig.Window);
            rig.Pad.Release(SDL.GamepadButton.LeftShoulder);
            Capture(rig.Window);
            Assert.Equal(SDL.GamepadButton.LeftShoulder, rig.Pads.For("N64").ButtonToPad[PadButton.L2]);
            Assert.Equal("Left Shoulder or Left Trigger", rig.Diagram.BindingOf("Z").Pad);

            PadWindowRouter.Send(rig.Window, UiButton.Accept);
            Capture(rig.Window);
            rig.Pad.SetAxis(SDL.GamepadAxis.LeftTrigger, 1);
            Capture(rig.Window);
            Assert.False(rig.Pads.For("N64").ButtonToPad.ContainsKey(PadButton.L2));
            Assert.Equal("Left Trigger", rig.Diagram.BindingOf("Z").Pad);
            Assert.Equal(PadCapture.None, rig.Window.Capturing);
            rig.Window.Close();
        });

        // In ES-DE's look on a big-screen sheet: framed as a menu, its own help bar, no Close or Test Buttons, and no word on any tab cut short, at both sizes.
        [Theory]
        [InlineData(1280, 800)]
        [InlineData(1920, 1200)]
        public Task In_the_look_the_window_is_a_menu_whose_help_bar_is_its_own_and_no_word_is_cut(int w, int h) => UiTest.Run(() =>
        {
            Directory.CreateDirectory(Path.Combine(_root, "Roms"));
            new AppSettings { RomDirectory = Path.Combine(_root, "Roms"), LibraryView = AppSettings.LibraryList, BigScreen = true }.Save();
            var main = new MainWindow { Width = w, Height = h };
            main.Show();
            var layer = main.GetVisualDescendants().OfType<EmuSen.LunaP.Windowing.SheetLayer>().First(l => l.Name == "Sheets");
            typeof(MainWindow).GetMethod("ShowControllerBindings", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(main, null);
            var bindings = (InputSettingsWindow)layer.Current!;
            Assert.True(layer.DrawsMenu(bindings));
            Assert.Equal(new[] { "Rebind", "Test buttons", "Back", "Console" }, EmuSen.LunaP.Controls.MenuLook.GetHints(bindings)!.Select(e => e.Label));
            Control sheet = layer.SheetOf(bindings)!;
            Assert.DoesNotContain(sheet.GetVisualDescendants().OfType<Button>(), b => b.IsEffectivelyVisible && b.Content is "Close" or "Test Buttons");

            TabControl tabs = sheet.GetVisualDescendants().OfType<TabControl>().First();
            var cut = new List<string>();
            for (int tab = 0; tab < tabs.ItemCount; tab++)
            {
                tabs.SelectedIndex = tab;
                main.UpdateLayout();
                UiTest.Capture(main);
                foreach (TextBlock text in sheet.GetVisualDescendants().OfType<TextBlock>().Where(t => t.IsEffectivelyVisible && t.TextWrapping == Avalonia.Media.TextWrapping.NoWrap && !string.IsNullOrEmpty(t.Text)))
                {
                    double shown = text.Bounds.Width;
                    text.Measure(Size.Infinity);
                    if (text.DesiredSize.Width - text.Margin.Left - text.Margin.Right > shown + 1) cut.Add($"{(tabs.SelectedItem as TabItem)?.Header}: '{text.Text}' needs {text.DesiredSize.Width:0} and has {shown:0}");
                }
                main.UpdateLayout();
            }
            Assert.Empty(cut);

            // Reset to Defaults stands at the end of General, and the footer row is gone while there is no conflict.
            tabs.SelectedIndex = 0;
            main.UpdateLayout();
            UiTest.Capture(main);
            Button? reset = sheet.GetVisualDescendants().OfType<Button>().FirstOrDefault(b => b.Content is "Reset to Defaults");
            Assert.True(reset is not null, string.Join(", ", sheet.GetVisualDescendants().OfType<Button>().Select(b => b.Content?.ToString())));
            Assert.True(reset.IsEffectivelyVisible);
            Assert.Same(tabs, reset.FindAncestorOfType<TabControl>());
            Assert.False(sheet.GetVisualDescendants().OfType<Border>().Single(b => b.Name == "FooterBar").IsVisible);

            bindings.SetTesting(true);
            Assert.Equal(new[] { "Hold to stop testing" }, EmuSen.LunaP.Controls.MenuLook.GetHints(bindings)!.Select(e => e.Label));
            bindings.Close();
            main.Close();
        });

        // The General tab's modern pad shows player 1's pad itself, whatever the bindings: its buttons, its triggers' travel, its sticks, and the deadzone as a ring.
        [Fact]
        public Task The_general_tab_shows_the_pad_itself_beside_the_deadzone() => UiTest.Run(() =>
        {
            var pad = new SimulatedPad { Name = "Test Pad" };
            var gamepad = new GamepadManager(new GamepadBindingMap(), start: true, SimulatedPads.With(pad));
            var window = new InputSettingsWindow(new ControllerKeyBindings(Consoles), new GamepadBindings(Consoles), gamepad, new AppSettings(), new HotkeyBindingMap()) { Width = 1280, Height = 800 };
            window.Show();
            window.CaptureRenderedFrame();
            ControllerDiagram raw = window.GetVisualDescendants().OfType<ControllerDiagram>().Single(d => d.Name == "RawDiagram");
            Assert.False(raw.ShowsLabels);

            pad.Press(SDL.GamepadButton.South);
            pad.Press(SDL.GamepadButton.Guide);
            pad.SetAxis(SDL.GamepadAxis.LeftTrigger, 0.3);
            pad.SetAxis(SDL.GamepadAxis.RightX, -0.6);
            gamepad.Poll();
            Dispatcher.UIThread.RunJobs();
            Assert.Equal(new[] { "B", "Guide" }, raw.Pressed.OrderBy(r => r).ToArray());
            Assert.Equal(0.3, raw.TriggerValue("L2"), 2);
            Assert.Equal(-0.6, raw.StickPosition("RightStick").X, 2);

            pad.ReleaseAll();
            pad.SetAxis(SDL.GamepadAxis.LeftTrigger, 0);
            pad.SetAxis(SDL.GamepadAxis.RightX, 0);
            gamepad.Poll();
            Assert.Empty(raw.Pressed);

            var slider = window.GetVisualDescendants().OfType<Slider>().Single(s => s.Name == "DeadzoneSlider");
            slider.Value = 0.3;
            Assert.Equal(0.3, raw.StickRing, 3);
            window.Close();
        });

        // Y tries the buttons: every press lights and none moves or rebinds, until B is held for a second.
        [Fact]
        public Task Testing_takes_every_press_until_B_is_held() => UiTest.Run(() =>
        {
            Rig rig = Open("SNES");
            var now = TimeSpan.Zero;
            rig.Window.Clock = () => now;
            rig.Diagram.Select("Start", NavigationMethod.Directional);

            PadWindowRouter.Send(rig.Window, UiButton.Search);
            Assert.True(rig.Window.IsTesting);
            PadWindowRouter.Send(rig.Window, UiButton.Accept);
            PadWindowRouter.Send(rig.Window, UiButton.Right);
            Assert.Equal(PadCapture.None, rig.Window.Capturing);
            Assert.Equal("Start", rig.Diagram.SelectedRegion);

            rig.Pad.Press(SDL.GamepadButton.East);
            rig.Poll();
            Assert.True(rig.Diagram.IsPressed("A"));
            PadWindowRouter.Send(rig.Window, UiButton.Back);
            Assert.True(rig.Window.IsVisible);
            now += TimeSpan.FromSeconds(0.5);
            rig.Poll();
            Assert.True(rig.Window.IsTesting);
            now += InputSettingsWindow.TestExitHold;
            rig.Poll();
            Assert.False(rig.Window.IsTesting);
            rig.Pad.Release(SDL.GamepadButton.East);
            rig.Poll();

            // The keyboard: Enter is Start's key, so it lights Start and does not rebind; Escape stops.
            rig.Window.SetTesting(true);
            rig.Window.KeyPress(Key.Enter, RawInputModifiers.None, PhysicalKey.None, null);
            Assert.True(rig.Diagram.IsPressed("Start"));
            Assert.Equal(PadCapture.None, rig.Window.Capturing);
            rig.Window.KeyRelease(Key.Enter, RawInputModifiers.None, PhysicalKey.None, null);
            rig.Window.KeyPress(Key.Escape, RawInputModifiers.None, PhysicalKey.None, null);
            Assert.False(rig.Window.IsTesting);
            rig.Window.Close();
        });

        [Fact]
        public Task Nothing_lights_while_a_binding_is_captured() => UiTest.Run(() =>
        {
            Rig rig = Open("SNES");
            rig.Window.MouseDown(InWindow(rig, "L"), MouseButton.Left);
            rig.Window.MouseUp(InWindow(rig, "L"), MouseButton.Left);
            rig.Pad.Press(SDL.GamepadButton.East);
            rig.Poll();
            Assert.Empty(rig.Diagram.Pressed);
            rig.Window.Close();
        });
    }
}
