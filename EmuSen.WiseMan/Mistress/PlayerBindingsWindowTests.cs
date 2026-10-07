using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.LogicalTree;
using Avalonia.Threading;
using EmuSen.Cores;
using EmuSen.Endymion.Input;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;

namespace EmuSen.WiseMan.Mistress
{
    // The Controller Bindings window's player selector: a console's ports, a player's own pad map from a rebind, and the way back to player 1's - see EmuSen_Input.md §8.4.
    [Collection(TestCollections.ProcessGlobals)]
    public class PlayerBindingsWindowTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenPlayerBindingsWindow", Guid.NewGuid().ToString("N"));
        private static readonly string[] Consoles = CoreCatalog.ConsolesInReleaseOrder.Select(c => c.Console).ToArray();

        public PlayerBindingsWindowTests()
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

        private static T? Named<T>(Control root, string name) where T : Control =>
            root.GetLogicalDescendants().OfType<T>().FirstOrDefault(c => c.Name == name);

        private static (InputSettingsWindow Window, SimulatedPad One, SimulatedPad Two, GamepadManager Gamepad, GamepadBindings Pads) Open(string console)
        {
            SimulatedPad one = new() { Name = "One" }, two = new() { Name = "Two" };
            var gamepad = new GamepadManager(new GamepadBindingMap(), start: true, SimulatedPads.With(one, two));
            var pads = new GamepadBindings(Consoles);
            var window = new InputSettingsWindow(new ControllerKeyBindings(Consoles), pads, gamepad, new AppSettings(), new HotkeyBindingMap(), console) { Width = 1280, Height = 800 };
            window.Show();
            window.CaptureRenderedFrame();
            Dispatcher.UIThread.RunJobs();
            return (window, one, two, gamepad, pads);
        }

        private static void Capture(InputSettingsWindow window) =>
            typeof(InputSettingsWindow).GetMethod("PollForPadButton", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);

        [Fact]
        public Task Each_console_offers_as_many_players_as_it_has_ports() => UiTest.Run(() =>
        {
            (InputSettingsWindow window, _, _, _, _) = Open("SNES");
            var tabs = Named<TabControl>(window, "Tabs")!;
            Control bar = Named<StackPanel>(window, "PlayerBar")!;
            Dropdown choice = Named<Dropdown>(window, "PlayerSelector")!;
            int[] counts = new int[Consoles.Length];
            for (int i = 0; i < Consoles.Length; i++)
            {
                tabs.SelectedIndex = i + 1;
                counts[i] = bar.IsVisible ? choice.ItemCount : 1;
            }
            // The Genesis's two pads as its port rows stand by default; an adapter set there adds its players.
            Assert.Equal(Consoles.Select(c => ControllerPorts.ForConsole(c, _ => null)), counts);
            Assert.Equal(new[] { 2, 1, 2, 4, 2 }, counts);
            tabs.SelectedIndex = 0;
            Assert.False(bar.IsVisible);
            tabs.SelectedIndex = Array.IndexOf(Consoles, "SNES") + 1;
            Assert.Equal(new[] { "Player 1", "Player 2" }, choice.Items.Cast<string>());
            Assert.False(Named<Button>(window, "UsePlayer1Button")!.IsEnabled);
            window.Close();
        });

        // A rebind for player 2 gives it its own map and leaves player 1's alone; Use Player 1's takes it back.
        [Fact]
        public Task Rebinding_for_player_2_gives_it_its_own_buttons() => UiTest.Run(() =>
        {
            (InputSettingsWindow window, SimulatedPad one, SimulatedPad two, GamepadManager gamepad, GamepadBindings pads) = Open("SNES");
            Named<Dropdown>(window, "PlayerSelector")!.SelectedItem = "Player 2";
            Assert.Equal(2, window.PlayerOf("SNES"));
            Assert.Equal(1, window.PlayerOf("NES"));
            Assert.Contains("player 1's gamepad buttons until", Named<TextBlock>(window, "PlayerWordsSNES")!.Text);

            ControllerDiagram diagram = window.DiagramFor("SNES")!;
            diagram.Select("B", NavigationMethod.Directional);
            PadWindowRouter.Send(window, UiButton.Accept);
            Capture(window);
            two.Press(SDL.GamepadButton.North);
            Capture(window);
            two.Release(SDL.GamepadButton.North);
            Capture(window);

            Assert.True(pads.HasOwn("SNES", 2));
            Assert.Equal(SDL.GamepadButton.North, pads.For("SNES", 2).ButtonToPad[PadButton.B]);
            Assert.Equal(SDL.GamepadButton.South, pads.For("SNES").ButtonToPad[PadButton.B]);
            Assert.Equal(SDL.GamepadButton.North, GamepadBindings.Load(Consoles).For("SNES", 2).ButtonToPad[PadButton.B]);
            Assert.Equal("North", diagram.BindingOf("B").Pad);

            // The drawing lights player 2's pad through player 2's map.
            two.Press(SDL.GamepadButton.North);
            gamepad.Poll();
            Dispatcher.UIThread.RunJobs();
            Assert.True(diagram.IsPressed("B"));
            two.Release(SDL.GamepadButton.North);

            Button back = Named<Button>(window, "UsePlayer1Button")!;
            Assert.True(back.IsEnabled);
            back.RaiseEvent(new Avalonia.Interactivity.RoutedEventArgs(Button.ClickEvent));
            Assert.False(pads.HasOwn("SNES", 2));
            Assert.Equal("South", diagram.BindingOf("B").Pad);
            window.Close();
        });

        // The selector stands at the tab strip's end and the pad reaches it.
        [Fact]
        public Task The_pad_alone_reaches_the_player_selector() => UiTest.Run(() =>
        {
            (InputSettingsWindow window, _, _, _, _) = Open("N64");
            window.DiagramFor("N64")!.Select(window.DiagramFor("N64")!.Regions[0].Id, NavigationMethod.Directional);
            HashSet<InputElement> reached = PadAudit.Reachable(PadWindowRouter.RootOf(window), b => PadWindowRouter.Send(window, b));
            Assert.Contains(Named<Dropdown>(window, "PlayerSelector")!, reached);
            window.Close();
        });
    }
}
