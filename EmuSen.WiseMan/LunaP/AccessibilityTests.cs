using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Automation;
using Avalonia.Automation.Peers;
using Avalonia.Automation.Provider;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.VisualTree;
using EmuSen.Endymion.Input;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Pages;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Mistress.BigPicture;

namespace EmuSen.WiseMan.LunaP
{
    // What a screen reader finds in EmuSen's windows - see EmuSen_LunaP.md §7.
    //
    // The instrument is the one LunaP.md §24.1 and Pegasus_Design.md §13.1 used:
    // ControlAutomationPeer.CreatePeerForElement is the route a platform bridge takes, so a probe
    // over a real window sees what assistive technology would.
    //
    // MOST OF WHAT THESE ASSERT WAS BOUGHT BY A VERSION BUMP RATHER THAN BY CODE HERE. On LunaP
    // 0.3.0 these same eleven windows named 69 of 97 tab stops; on 0.5.0, with no application
    // change at all, 95 of 97. DebugSettingsWindow went from 1 of 16 to 16 of 16 because its
    // fifteen LunaSwitches carry their labels in OnContent, which the toolkit's peer now reads.
    // That is worth a guard precisely because nothing in this repository would notice it breaking.
    public class AccessibilityTests
    {
        public static TheoryData<string> Windows => new()
        {
            "ActiveCheats", "CheatDatabase", "ThemeSettings",
            "Vstop", "Main", "RomBrowser", "DianaShConsole",
        };

        private static Window Build(string name) => name switch
        {
            "ActiveCheats" => new ActiveCheatsWindow(),
            "CheatDatabase" => new CheatDatabaseWindow(),
            "ThemeSettings" => new ThemeSettingsWindow(new AppSettings(), () => new System.Net.Http.HttpClient(), _ => { }),
            "Vstop" => new VstopWindow(),
            "Main" => new MainWindow(),
            "RomBrowser" => new RomBrowserWindow(),
            "DianaShConsole" => new HotaruWindow(),
            _ => throw new ArgumentOutOfRangeException(nameof(name), name, "No builder for this window."),
        };

        private static string NameOf(Visual v) =>
            ControlAutomationPeer.CreatePeerForElement((Control)v).GetName() ?? "";

        // Shown, then measured and arranged. All three matter: IsEffectivelyVisible is false
        // throughout a window that was never shown, which would make every guard below pass by
        // having nothing to check. Pegasus_Design.md §13.5 records that mistake being made.
        private static Window LaidOut(Window window)
        {
            window.Show();
            Avalonia.Threading.Dispatcher.UIThread.RunJobs();
            window.Measure(new Size(1200, 800));
            window.Arrange(new Rect(0, 0, 1200, 800));
            Avalonia.Threading.Dispatcher.UIThread.RunJobs();
            return window;
        }

        // IsEffectivelyVisible is load-bearing: a ComboBox template carries a hidden TextBox for
        // its editable mode, and counting it reports a tab stop no keyboard reaches (LunaP.md §24.5).
        private static List<Visual> TabStops(Window window) =>
            window.GetVisualDescendants()
                .Where(v => v is InputElement { Focusable: true, IsTabStop: true, IsEffectivelyVisible: true })
                .ToList();

        // THE GUARD THAT COVERS WINDOWS NOBODY REMEMBERS TO ADD HERE. It does not know what the
        // controls are: it walks whatever the keyboard can land on and fails with the type names of
        // any that announce as nothing. An unnamed tab stop is a dead end for somebody who cannot
        // see where focus went - the control says its kind, "edit" or "button", and stops.
        [Theory]
        [MemberData(nameof(Windows))]
        public Task Nothing_the_keyboard_can_reach_is_unnamed(string name) => UiTest.Run(() =>
        {
            using var window = new Showing(LaidOut(Build(name)));

            List<Visual> stops = TabStops(window.Window);

            // Asserted first, so this cannot pass by finding nothing to check.
            Assert.True(stops.Count > 0, $"{name} reported no tab stops at all, so the walk proved nothing.");

            string[] unnamed = stops
                .Where(v => string.IsNullOrWhiteSpace(NameOf(v)))
                .Select(v => v.GetType().Name)
                .ToArray();

            Assert.True(unnamed.Length == 0, $"{name} has unnamed tab stops: {string.Join(", ", unnamed)}");
        });

        // A list whose rows announce as themselves is still a list of nothing in particular.
        [Theory]
        [InlineData("Main")]
        [InlineData("CheatDatabase")]
        [InlineData("ActiveCheats")]
        [InlineData("RomBrowser")]
        public Task Every_list_says_what_it_is_a_list_of(string name) => UiTest.Run(() =>
        {
            using var window = new Showing(LaidOut(Build(name)));

            ListBox[] lists = window.Window.GetVisualDescendants().OfType<ListBox>().ToArray();
            Assert.NotEmpty(lists);

            foreach (ListBox list in lists)
            {
                Assert.False(string.IsNullOrWhiteSpace(NameOf(list)), $"a list in {name} has no name");
            }
        });

        // LunaP puts these in the automation tree and deliberately does not name them, because only
        // the application knows whether a run of meters is audio or core load, and a guessed
        // description of a live pixel buffer is a wrong alt text - believed, rather than asked
        // about. LunaP.md §24.2 draws that line; this is the consumer's half.
        [Fact]
        public Task The_dashboards_name_the_toolkit_controls_the_toolkit_cannot() => UiTest.Run(() =>
        {
            using var coretop = new Showing(LaidOut(new EmuSen.Serenity.Dashboards.CoretopWindow()));
            using var feed = new Showing(LaidOut(new EmuSen.Hotaru.Views.FeedWindow()));

            var anonymous = new List<string>();

            foreach (Window w in new[] { coretop.Window, feed.Window })
            {
                foreach (Visual v in w.GetVisualDescendants())
                {
                    if (v is not (MeterList or RgbaImageView)) continue;
                    if (string.IsNullOrWhiteSpace(NameOf(v))) anonymous.Add($"{w.GetType().Name}.{v.GetType().Name}");
                }
            }

            Assert.True(anonymous.Count == 0, "unnamed toolkit controls: " + string.Join(", ", anonymous));
        });

        // Status lines are read rather than sought: seventeen assignments in MainWindow alone,
        // carrying results, state and failures on one line. Polite is what makes them arrive.
        [Theory]
        [InlineData("Main")]
        [InlineData("CheatDatabase")]
        [InlineData("ActiveCheats")]
        public Task The_status_line_announces_itself(string name) => UiTest.Run(() =>
        {
            using var window = new Showing(LaidOut(Build(name)));

            bool anyLive = window.Window.GetVisualDescendants()
                .OfType<Control>()
                .Any(c => AutomationProperties.GetLiveSetting(c) == AutomationLiveSetting.Polite);

            Assert.True(anyLive, $"{name} has no live region, so a changed status is never announced");
        });

        // THE BUMP ITSELF. Every one of these labels comes from LunaP's own peers and templates
        // rather than from anything in this repository, so this is what would go red if a future
        // LunaP dropped them - which is the failure this repository could least otherwise see.
        [Fact]
        public Task The_toolkit_still_names_its_own_controls() => UiTest.Run(() =>
        {
            using var switchesShown = new Showing(LaidOut(new Window { Width = 480, Height = 200, Content = new StackPanel { Children = { new LunaSwitch { Label = "Show the status bar" }, new LunaSwitch { Label = "Pause the game when another window is in front" } } } }));

            LunaSwitch[] switches = switchesShown.Window.GetVisualDescendants().OfType<LunaSwitch>().ToArray();
            Assert.True(switches.Length >= 1, $"expected the switches, found {switches.Length}");

            foreach (LunaSwitch s in switches)
            {
                Assert.Equal(s.Label, NameOf(s));
            }

            using var prefs = new Showing(LaidOut(new Window { Width = 640, Height = 240, Content = new StackPanel { Children = { new PathPickerRow { BrowseTitle = "Choose the ROM folder" }, new PathPickerRow { BrowseTitle = "Choose the log folder" } } } }));

            // PathPickerRow names its own parts from BrowseTitle, and the Browse buttons keep their
            // caption while help text tells them apart.
            Button[] browse = prefs.Window.GetVisualDescendants()
                .OfType<Button>()
                .Where(b => string.Equals(b.Content as string, "Browse...", StringComparison.Ordinal))
                .ToArray();

            Assert.True(browse.Length >= 2, $"expected several Browse buttons, found {browse.Length}");

            string[] helps = browse.Select(b => ControlAutomationPeer.CreatePeerForElement(b).GetHelpText() ?? "").ToArray();
            Assert.DoesNotContain("", helps);
            Assert.Equal(helps.Length, helps.Distinct().Count());
        });

        // The words a big-screen settings row is drawn with, as a reader should hear them: its label and its value.
        private static string Spoken(PageRow row) =>
            MenuRow.Describe(row.Label, row.Status ?? (row.Kind == PageRowKind.Asks ? "…" : null), MenuRowKind.Action, false);

        private static AutomationPeer PeerNamed(Control root, string name) =>
            ControlAutomationPeer.CreatePeerForElement(root.GetVisualDescendants().OfType<Control>().First(c => c.Name == name));

        private static string Selection(Control root, string name) =>
            string.Join("|", PeerNamed(root, name).GetProvider<ISelectionProvider>()!.GetSelection().Select(p => p.GetName()));

        // What a reader is given on a big-screen page: the element holding the focus, which must be the selected row and the rows' one selection.
        private static string Heard(Control layout, string rows = "SettingsRows")
        {
            var focused = TopLevel.GetTopLevel(layout)!.FocusManager!.GetFocusedElement() as Control;
            Assert.NotNull(focused);
            AutomationPeer peer = ControlAutomationPeer.CreatePeerForElement(focused!);
            Assert.True(peer.HasKeyboardFocus(), $"{focused!.Name} does not report the focus");
            Assert.True(peer.GetProvider<ISelectionItemProvider>()?.IsSelected, $"{focused!.Name} is not reported selected");
            Assert.Equal(peer.GetName(), Selection(layout, rows));
            return peer.GetName();
        }

        // Down the page from the focused row to its last, the reader told each row's label and value in turn, the focus passing from row to row and never resting on the screen between.
        private static int Walk(SettingsLayout layout, Action down) =>
            Walk(layout, () => layout.Rows, () => layout.Focused, "SettingsRows", down);

        private static int Walk(Control layout, Func<IReadOnlyList<PageRow>> rows, Func<string?> focus, string rowsName, Action down)
        {
            var told = new List<object?>();
            TopLevel.GetTopLevel(layout)!.AddHandler(InputElement.GotFocusEvent, (_, e) => told.Add(e.Source));
            for (int steps = 1; ; steps++)
            {
                Avalonia.Threading.Dispatcher.UIThread.RunJobs();
                PageRow row = rows().Single(r => r.Key == focus());
                Assert.Equal(Spoken(row), Heard(layout, rowsName));
                if (row.Key == rows()[^1].Key) return steps;
                told.Clear();
                down();
                Assert.True(told.Count > 0, $"the move from {row.Key} was not told");
                Assert.DoesNotContain(layout, told);
            }
        }

        // Settings ▸ Consoles ▸ a console's page by pad: the chosen section and the row the pad is on are what a reader hears selected, at every step.
        [Fact]
        public Task Settings_tells_a_reader_the_row_the_pad_is_on_through_a_console_s_page() => ThemedLibraryPadTests.Run(s =>
        {
            SettingsLayout layout = SettingsPad.Open(s.Window, s.Pad);
            SettingsPad.Section(s.Window, s.Pad, SettingsScreen.Consoles);
            Assert.Equal("Consoles", Selection(layout, "SettingsSections"));
            Assert.Equal(Spoken(layout.Rows[0]), Heard(layout));
            SettingsPad.To(s.Window, s.Pad, "console:PlayStation");
            Assert.Equal(Spoken(layout.Rows.Single(r => r.Key == "console:PlayStation")), Heard(layout));
            s.Pad.A();
            Assert.Equal(2, layout.Trail.Count);
            Assert.Equal(layout.Trail[^1], Selection(layout, "SettingsSections"));
            Assert.True(Walk(layout, () => s.Pad.Down()) >= 4, "the console's page had too few rows to walk");
        });

        // A game's page by pad: each action told selected as the pad walks across them, then its chosen section selected and each row of Options told the same.
        [Fact]
        public Task A_game_s_page_tells_a_reader_the_action_and_the_row_the_pad_is_on() => ThemedLibraryPadTests.Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.X();
            PageLayout layout = s.Window.GamePageLayout!;
            var told = new List<object?>();
            s.Window.AddHandler(InputElement.GotFocusEvent, (_, e) => told.Add(e.Source));
            var heard = new List<string>();
            for (int guard = 0; guard < 20; guard++)
            {
                Avalonia.Threading.Dispatcher.UIThread.RunJobs();
                PageAction action = layout.Actions.Single(a => a.Key == layout.Focused.Key);
                string said = Heard(layout, "PageActions");
                Assert.Equal(action.Value is null ? action.Text : $"{action.Text}, {action.Value}", said);
                if (heard.Count == 0 || heard[^1] != said) heard.Add(said);
                if (action.Key == layout.Actions[^1].Key) break;
                told.Clear();
                s.Pad.Right();
                Assert.NotEmpty(told);
                Assert.DoesNotContain(layout, told);
            }
            Assert.Equal(layout.Actions[^1].Key, layout.Focused.Key);
            Assert.True(heard.Count >= 2, "the page had too few actions to walk");
            for (int i = 0; i < 6 && s.Window.GamePageShown!.Section != GamePage.Options; i++) s.Pad.R1();
            Assert.Equal(GamePage.Options, s.Window.GamePageShown!.Section);
            Avalonia.Threading.Dispatcher.UIThread.RunJobs();
            Assert.Equal("Options", Selection(layout, "PageSections"));
            Assert.True(layout.Focused.InActions);
            Assert.Equal("", Selection(layout, "PageRows"));
            s.Pad.Down();
            Assert.False(layout.Focused.InActions);
            IReadOnlyList<PageRow> Rows() => ((PageContent.Rows)layout.Content!).Items;
            Assert.True(Walk(layout, Rows, () => layout.Focused.Key, "PageRows", () => s.Pad.Down()) >= 3, "the page had too few rows to walk");
        });

        // A running engine's own rows from its options, through the fake host: the same, each heading and held row included.
        [Fact]
        public Task A_running_engine_s_rows_are_told_as_the_pad_walks_them() => UiTest.Run(() =>
        {
            IReadOnlyList<SettingRow> rows = WindowFitAuditTests.HostedRows();
            var host = new SettingsHostFake { Rows = c => c == "PlayStation 2" ? rows : RuntimeCatalog.SettingsFor(c), RunningConsole = "PlayStation 2" };
            var screen = new SettingsScreen(host, SettingsScreen.Consoles);
            screen.OpenPath(SettingsScreen.Consoles, "console:PlayStation 2");
            var layout = new SettingsLayout { Form = AdaptiveForm.BigScreen };
            _ = new EmuSen.Mistress.Views.SettingsView(screen, layout, () => null);
            MenuPanel.SetScale(layout, 1);
            using var shown = new Showing(new Window { Width = 1280, Height = 800, Content = layout });
            shown.Window.Show();
            layout.Focus();
            Assert.Contains(layout.Rows, r => r.Key == "tc_deinterlace");
            Assert.True(Walk(layout, () => layout.Move(0, 1)) == layout.Rows.Count);
        });

        private sealed class Showing : IDisposable
        {
            public Showing(Window window) => Window = window;

            public Window Window { get; }

            public void Dispose() => Window.Close();
        }
    }
}
