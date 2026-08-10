using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Automation;
using Avalonia.Automation.Peers;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.VisualTree;
using EmuSen.Endymion.Input;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

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
        private static Window BuildInputSettings() => new InputSettingsWindow(
            new ControllerKeyBindings(new[] { "NES", "SNES" }),
            new GamepadBindings(new[] { "NES", "SNES" }), null!, new AppSettings(), new HotkeyBindingMap(), null);

        public static TheoryData<string> Windows => new()
        {
            "InputSettings", "ActiveCheats", "CheatDatabase", "Preferences",
            "DebugSettings", "Vstop", "Main", "RomBrowser", "DianaOSConsole",
        };

        private static Window Build(string name) => name switch
        {
            "InputSettings" => BuildInputSettings(),
            "ActiveCheats" => new ActiveCheatsWindow(),
            "CheatDatabase" => new CheatDatabaseWindow(),
            "Preferences" => new PreferencesWindow(new AppSettings()),
            "DebugSettings" => new DebugSettingsWindow(),
            "Vstop" => new VstopWindow(),
            "Main" => new MainWindow(),
            "RomBrowser" => new RomBrowserWindow(),
            "DianaOSConsole" => new DianaOSConsoleWindow(),
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

        // Seven binding rows, fourteen buttons, two captions between them. The caption stays - an
        // accessible name that drops the visible label breaks voice control - so what tells them
        // apart is help text, and this is what stops somebody removing it as redundant.
        //
        // ONE TAB PER RUN, AND THAT IS NOT A DETAIL. Avalonia realises only the selected tab, so a
        // window opened on General contains the hotkey rows and none of the console binding rows -
        // which is exactly what the first version of this test did, while its name claimed to cover
        // all fourteen buttons. It stayed green under a sabotage that stripped the help text off
        // every console rebind button, because those buttons were not in the tree to check. The
        // existing render tests already knew this and said so (EmuSen_Settings_Reference.md §4.6);
        // this one had to rediscover it by surviving a sabotage it should have caught.
        [Theory]
        [InlineData(null)]
        [InlineData("NES")]
        [InlineData("SNES")]
        public Task Every_rebind_button_says_which_binding_it_belongs_to(string? console) => UiTest.Run(() =>
        {
            using var window = new Showing(LaidOut(new InputSettingsWindow(
                new ControllerKeyBindings(new[] { "NES", "SNES" }),
                new GamepadBindings(new[] { "NES", "SNES" }), null!, new AppSettings(), new HotkeyBindingMap(), console)));

            Button[] repeated = window.Window.GetVisualDescendants()
                .OfType<Button>()
                .Where(b => string.Equals(b.Content as string, "Rebind Key", StringComparison.Ordinal)
                            || string.Equals(b.Content as string, "Clear", StringComparison.Ordinal)
                            || string.Equals(b.Content as string, "Rebind Pad", StringComparison.Ordinal)
                            || string.Equals(b.Content as string, "Clear Pad", StringComparison.Ordinal))
                .ToArray();

            Assert.True(repeated.Length >= 4, $"expected several repeated rebind buttons, found {repeated.Length}");

            foreach (Button b in repeated)
            {
                AutomationPeer peer = ControlAutomationPeer.CreatePeerForElement(b);

                Assert.Equal(b.Content as string, peer.GetName());
                Assert.False(string.IsNullOrWhiteSpace(peer.GetHelpText()),
                    $"a '{b.Content}' button has no help text, so it is indistinguishable from the others");
            }

            // And no two say the same thing, which is the actual defect being guarded.
            string[] helps = repeated.Select(b => ControlAutomationPeer.CreatePeerForElement(b).GetHelpText() ?? "").ToArray();
            Assert.Equal(helps.Length, helps.Distinct().Count());
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
            using var debug = new Showing(LaidOut(new DebugSettingsWindow()));

            LunaSwitch[] switches = debug.Window.GetVisualDescendants().OfType<LunaSwitch>().ToArray();
            Assert.True(switches.Length >= 10, $"expected the debug window's switches, found {switches.Length}");

            foreach (LunaSwitch s in switches)
            {
                Assert.Equal(s.Label, NameOf(s));
            }

            using var prefs = new Showing(LaidOut(new PreferencesWindow(new AppSettings())));

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

        private sealed class Showing : IDisposable
        {
            public Showing(Window window) => Window = window;

            public Window Window { get; }

            public void Dispose() => Window.Close();
        }
    }
}
