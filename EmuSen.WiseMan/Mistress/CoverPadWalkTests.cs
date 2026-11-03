using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.VisualTree;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;
using static EmuSen.WiseMan.Mistress.BigPicture.WindowFitAuditTests;

namespace EmuSen.WiseMan.Mistress
{
    // Every cover the big screen draws over a screen, walked by the pad: each direction, X, Y, A and B, and neither the focus nor a press ever reaches the layer beneath - see EmuSen_Settings_Reference.md §4.45.10.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class CoverPadWalkTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session = HeadlessUnitTestSession.GetOrStartForAssembly(typeof(CoverPadWalkTests).GetTypeInfo().Assembly);
        private readonly string _root = Path.Combine(Environment.GetEnvironmentVariable("TMPDIR") is { Length: > 0 } t ? t : Path.GetTempPath(), "EmuSenCoverWalk_" + Guid.NewGuid().ToString("N"));

        public CoverPadWalkTests()
        {
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private MainWindow Main()
        {
            string roms = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(roms);
            new EmuSen.Galaxia.Models.AppSettings { RomDirectory = roms, BigScreen = true, LogDirectory = Path.Combine(_root, "Logs"), ResumeOnLaunch = EmuSen.Galaxia.Models.AppSettings.ResumeNever }.Save();
            var main = new MainWindow { Width = 1280, Height = 800 };
            main.Show();
            Settle(main);
            return main;
        }

        private static IInputElement? Focused(Window w) => TopLevel.GetTopLevel(w)?.FocusManager?.GetFocusedElement();

        private static bool Covered(IInputElement? e) => e is Control c && SheetLayer.GetIsCovered(c);

        // What lies beneath a cover that a stray press would change: every covered list's selection and every covered toggle's state.
        private static string Beneath(Control root) => string.Join(";", root.GetVisualDescendants().OfType<Control>().Where(SheetLayer.GetIsCovered).Select(c => c switch
        {
            Avalonia.Controls.Primitives.SelectingItemsControl s => $"{s.GetType().Name}:{s.SelectedIndex}",
            Avalonia.Controls.Primitives.ToggleButton b => $"toggle:{b.IsChecked}",
            _ => null,
        }).Where(s => s is not null));

        private static readonly (string Name, SDL.GamepadButton Button)[] Walk =
            [("Up", SDL.GamepadButton.DPadUp), ("Down", SDL.GamepadButton.DPadDown), ("Left", SDL.GamepadButton.DPadLeft), ("Right", SDL.GamepadButton.DPadRight), ("X", SDL.GamepadButton.West), ("Y", SDL.GamepadButton.North)];

        // Each direction, X and Y three times over the open cover: the focus stays on it and nothing beneath changes.
        private static void WalkOver(MainWindow main, PadDriver pad, Control sheet, Func<bool> open, string what)
        {
            string before = Beneath(sheet);
            for (int round = 0; round < 3; round++)
                foreach ((string name, SDL.GamepadButton button) in Walk)
                {
                    pad.Tap(button);
                    Settle(main);
                    Assert.True(open(), $"{what}: {name} closed the cover");
                    Assert.False(Covered(Focused(main)), $"{what}: {name} put the focus beneath the cover, on {Focused(main)}");
                    Assert.Equal(before, Beneath(sheet));
                }
        }

        private static (ToolPaneWindow Window, AdaptiveLayout Pane, Control Sheet) Open(MainWindow main, string show)
        {
            typeof(MainWindow).GetMethod(show, BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(main, null);
            Settle(main);
            var window = (ToolPaneWindow)Sheets(main).Current!;
            var pane = (AdaptiveLayout)window.GetType().GetProperty("Pane")!.GetValue(window)!;
            Assert.Equal(AdaptiveForm.BigScreen, pane.Form);
            for (int i = 0; i < 200 && (bool)(pane.GetType().GetProperty("Busy")?.GetValue(pane) ?? false); i++) { Thread.Sleep(25); pane.GetType().GetMethod("Poll")?.Invoke(pane, null); Settle(main); }
            pane.FocusRole(AdaptiveRole.Content);
            Settle(main);
            return (window, pane, Sheets(main).SheetOf(window)!);
        }

        [Theory]
        [InlineData("ShowCores")]
        [InlineData("ShowPackages")]
        [InlineData("ShowLibraryManager")]
        [InlineData("ShowScraper")]
        public Task A_tool_screens_list_card_and_question_keep_every_press(string show) => Session.Dispatch(() =>
        {
            MainWindow main = Main();
            var pad = new PadDriver(main);
            (ToolPaneWindow window, AdaptiveLayout pane, Control sheet) = Open(main, show);

            // The List card: walked, then A on whatever it holds the focus on acts there, then B closes it, the screen kept.
            if (pane.HasListTools)
            {
                pad.Select();
                Settle(main);
                Assert.True(pane.IsListToolsOpen, show);
                WalkOver(main, pad, sheet, () => pane.IsListToolsOpen, show + "'s List card");
                string before = Beneath(sheet);
                Assert.False(Covered(Focused(main)));
                pad.A();
                Settle(main);
                for (int i = 0; i < 3 && (pane.IsListToolsOpen || OnScreenKeyboard.OpenOver(main) is not null || Sheets(main).Current != window); i++) { pad.B(); Settle(main); }
                Assert.Same(window, Sheets(main).Current);
            }

            // The question card, the frame's own: walked, A on its first answer (No) and B both close it with the screen kept.
            MethodInfo ask = pane.GetType().GetMethod("Ask")!;
            foreach (SDL.GamepadButton closing in new[] { SDL.GamepadButton.South, SDL.GamepadButton.East })
            {
                var answered = (Task<bool>)ask.Invoke(pane, ["A question?", "Words.", "Yes", "No", null])!;
                Settle(main);
                Assert.True(pane.IsCoverOpen, show);
                WalkOver(main, pad, sheet, () => pane.IsCoverOpen, show + "'s question");
                string before = Beneath(sheet);
                Assert.False(Covered(Focused(main)));
                pad.Tap(closing);
                Settle(main);
                Assert.False(pane.IsCoverOpen, show);
                Assert.True(answered.IsCompleted, show);
                Assert.Same(window, Sheets(main).Current);
            }
            main.Close();
        }, default);

        // The on-screen keyboard over Cores' search: every press is the keyboard's, the list beneath unchanged.
        [Fact]
        public Task The_on_screen_keyboard_keeps_every_press() => Session.Dispatch(() =>
        {
            MainWindow main = Main();
            var pad = new PadDriver(main);
            (ToolPaneWindow window, AdaptiveLayout pane, Control sheet) = Open(main, "ShowCores");
            TextBox search = pane.GetVisualDescendants().OfType<TextBox>().First(t => Avalonia.Automation.AutomationProperties.GetName(t) == "Search");
            search.Focus();
            pad.A();
            Settle(main);
            Assert.NotNull(OnScreenKeyboard.OpenOver(main));
            string before = string.Join(";", sheet.GetVisualDescendants().OfType<ListBox>().Select(l => l.SelectedIndex));
            foreach ((string name, SDL.GamepadButton button) in Walk)
            {
                pad.Tap(button);
                Settle(main);
                Assert.NotNull(OnScreenKeyboard.OpenOver(main));
                Assert.Equal(before, string.Join(";", sheet.GetVisualDescendants().OfType<ListBox>().Select(l => l.SelectedIndex)));
            }
            for (int i = 0; i < 4 && OnScreenKeyboard.OpenOver(main) is not null; i++) { pad.B(); Settle(main); }
            Assert.Null(OnScreenKeyboard.OpenOver(main));
            Assert.Same(window, Sheets(main).Current);
            main.Close();
        }, default);

        // The Pause Card over a synthetic game, and each page it opens in place: every press is the card's, the game kept behind it, B back to the first page.
        [Fact]
        public Task The_pause_cards_pages_keep_every_press() => Session.Dispatch(() =>
        {
            string roms = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(roms);
            File.WriteAllBytes(Path.Combine(roms, "Cobalt Harbor (Synthetic).sfc"), PadRewindReelTests.ChangingBackdrop());
            var app = new EmuSen.Galaxia.Models.AppSettings { RomDirectory = roms, LibraryView = EmuSen.Galaxia.Models.AppSettings.LibraryList, ResumeOnLaunch = EmuSen.Galaxia.Models.AppSettings.ResumeNever, BigScreen = true, StateDirectory = Path.Combine(_root, "States"), LogDirectory = Path.Combine(_root, "Logs") };
            app.BigPictureInterface.MenuOpeningEffect = EmuSen.Galaxia.Models.BigPictureInterface.OpeningNone;
            app.Save();
            var main = new MainWindow { Width = 1280, Height = 800 };
            main.Show();
            var pad = new PadDriver(main);
            main.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
            pad.A();
            Settle(main);
            Control game = main.GetControl<Control>("GameFrame");
            Assert.True(game.IsVisible);
            pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
            Settle(main);
            Assert.True(PadMenu.IsOpen(main));
            string[] first = PadMenu.Lines(main);
            int pages = 0;
            for (int i = 0; i < PadMenu.Entries(main).Count; i++)
            {
                if (!PadMenu.Entries(main)[i].Opens) continue;
                PadMenu.List(main).SelectedIndex = i;
                pad.A();
                Settle(main);
                if (Sheets(main).Current is not null)
                {
                    // A tool's screen, walked by the test above; B closes it back to the card.
                    for (int k = 0; k < 4 && Sheets(main).Current is not null; k++) { pad.B(); Settle(main); }
                    Assert.Null(Sheets(main).Current);
                    continue;
                }
                if (!PadMenu.IsOpen(main) || PadMenu.Lines(main).SequenceEqual(first)) continue;
                pages++;
                string page = string.Join("|", PadMenu.Lines(main));
                foreach ((string name, SDL.GamepadButton button) in Walk.Where(w => w.Name is not ("X" or "Y")))
                {
                    pad.Tap(button);
                    Settle(main);
                    Assert.True(PadMenu.IsOpen(main), $"{first[i]}: {name} closed the card");
                    Assert.Null(Sheets(main).Current);
                    Assert.True(game.IsVisible, $"{first[i]}: {name} left the game");
                }
                for (int k = 0; k < 3 && !PadMenu.Lines(main).SequenceEqual(first); k++) { pad.B(); Settle(main); }
                Assert.True(PadMenu.IsOpen(main), first[i]);
                Assert.Equal(first, PadMenu.Lines(main));
            }
            Assert.True(pages > 0, "the Pause Card opened no page in place: " + string.Join(", ", first));
            typeof(MainWindow).GetMethod("StopEmulationThread", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(main, null);
            main.Close();
        }, default);

        // The Home card over the library: every press is the card's, and B closes it with no sheet opened beneath.
        [Fact]
        public Task The_home_card_keeps_every_press() => Session.Dispatch(() =>
        {
            MainWindow main = Main();
            var pad = new PadDriver(main);
            pad.Start();
            Settle(main);
            Func<bool> shown = () => PadMenu.IsOpen(main);
            Assert.True(shown());
            foreach ((string name, SDL.GamepadButton button) in Walk.Where(w => w.Name is not ("X" or "Y")))
            {
                pad.Tap(button);
                Settle(main);
                Assert.True(shown(), name);
                Assert.Null(Sheets(main).Current);
            }
            main.Close();
        }, default);
    }
}
