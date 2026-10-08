using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.LogicalTree;
using Avalonia.VisualTree;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Pass 4's switches, each set on its sheet by the pad and each changing the pixels inside its own element's box and nowhere else - see EmuSen_BigPicture.md §29.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemedSwitchesTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ThemedSwitchesTests).GetTypeInfo().Assembly);

        private readonly ITestOutputHelper _out;

        public ThemedSwitchesTests(ITestOutputHelper output) => _out = output;

        internal static readonly DeviceStatus Device = new(Bluetooth: true, Wifi: true, BatteryPercent: 64);

        internal const string Clock = "<clock name=\"clock\"><pos>0.75 0.02</pos><fontSize>0.05</fontSize><color>FFFFFF</color></clock>";
        internal const string Status = "<systemstatus name=\"status\"><pos>0.98 0.12</pos><origin>1 0</origin><height>0.05</height><entries>all</entries><color>FFFFFF</color></systemstatus>";

        private static SheetLayer Sheets(ThemedSession s) => s.Window.GetControl<SheetLayer>("Sheets");

        private static Control RootOf(Window window) => SheetLayer.PresenterOf(window)?.SheetOf(window) ?? window;

        internal static T Named<T>(Window window, string name) where T : Control =>
            RootOf(window).GetLogicalDescendants().OfType<T>().FirstOrDefault(c => c.Name == name) ?? throw new InvalidOperationException($"no {typeof(T).Name} {name}");

        // Theme Settings from the pad menu, on its Interface tab.
        internal static ThemeSettingsWindow OpenInterface(ThemedSession s)
        {
            ThemedLibraryFlowTests.Choose(s, "Theme Settings");
            s.Settle();
            var sheet = Assert.IsType<ThemeSettingsWindow>(Sheets(s).Current);
            TabControl tabs = Named<TabControl>(sheet, "ThemeSettingsTabs");
            while ((tabs.SelectedItem as TabItem)?.Header as string != "Interface") s.Pad.R1();
            s.Settle();
            return sheet;
        }

        // A switch on the Interface tab reached by the pad and flipped with A, then the sheet put away with B.
        internal static void Flip(ThemedSession s, string name)
        {
            ThemeSettingsWindow sheet = OpenInterface(s);
            PadAudit.Reach(Sheets(s).SheetOf(sheet)!, s.Pad, e => e is Control { Name: { } n } && n == name);
            s.Pad.A();
            PutAway(s);
            Assert.False(Sheets(s).IsPresenting);
            s.Settle();
        }

        // B back out of the Interface submenu and then out of the sheet, as ES-DE's menus go back a screen at a time (§4.72.8).
        internal static void PutAway(ThemedSession s)
        {
            for (int guard = 0; guard < 4 && Sheets(s).IsPresenting; guard++) s.Pad.B();
        }

        internal static void Choose(ThemedSession s, string dropdown, string text)
        {
            ThemeSettingsWindow sheet = OpenInterface(s);
            Named<Dropdown>(sheet, dropdown).SelectedItem = text;
            PutAway(s);
            s.Settle();
        }

        private static Rect BoxOf(ThemedSession s, Control c) => new(c.TranslatePoint(default, s.Window)!.Value, c.Bounds.Size);

        private static T Only<T>(ThemedSession s) where T : Control => s.Themed.Stage!.Current.Scene.Entries.Select(e => e.Control).OfType<T>().Single();

        private static bool Has<T>(ThemedSession s) where T : Control => s.Themed.Stage!.Current.Scene.Entries.Any(e => e.Control is T);

        private static ThemedSession Session4(string extra = Clock + Status) => new(extraGamelist: extra, status: Device);

        private static void Frozen(ThemedSession s)
        {
            s.Themed.LiveClock = false;
            s.Themed.Now = () => new DateTime(2026, 9, 27, 13, 45, 0);
        }

        private void Assert_only_inside(RenderedFrame before, RenderedFrame after, Rect box, string what, int least = 50)
        {
            (int inside, int outside) = BuiltInBadgesTests.Split(before, after, box.Inflate(1));
            _out.WriteLine($"{what}: {inside} pixels changed inside its box {box}, {outside} outside, within {BuiltInBadgesTests.Changed(before, after, box.Inflate(1))}");
            Assert.True(inside > least, $"{what}: only {inside} pixels changed inside");
            Assert.Equal(0, outside);
        }

        // P106: the clock, off by default as ES-DE's DisplayClock is, turned on from the sheet by the pad, changes only the pixels inside its box; turned off, the frame is the one before.
        [Fact]
        public Task The_clock_is_off_by_default_and_turning_it_on_changes_only_its_own_box() => Session.Dispatch(() =>
        {
            using ThemedSession s = Session4();
            Frozen(s);
            ThemedLibraryPadTests.Enter(s, "snes");
            Assert.False(s.Settings.BigPictureInterface.DisplayClock);
            Assert.False(Has<ClockLabel>(s));
            RenderedFrame off = s.Capture();

            Flip(s, "DisplayClock");
            Assert.True(s.Settings.BigPictureInterface.DisplayClock);
            Assert.True(AppSettings.Load().BigPictureInterface.DisplayClock);
            Assert.Equal(("snes", "gamelist"), (s.System, s.View));
            ClockLabel clock = Only<ClockLabel>(s);
            Assert.Equal("13:45", clock.Text);
            RenderedFrame on = s.Capture();
            Assert_only_inside(off, on, BoxOf(s, clock), "clock");

            Flip(s, "DisplayClock");
            Assert.Equal(0, SceneAssets.Differing(off, s.Capture()));
        }, default);

        // P106 on Art Book Next's system view, whose own clock sits at the top left (its list variants set the gamelist's to scope none): turned on, it changes only its own box.
        [ArtBookNextFact]
        public Task Art_Book_Next_s_clock_turned_on_changes_only_its_own_box() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(themeDirectory: ArtBookNextFactAttribute.Folder, status: Device);
            Frozen(s);
            Assert.Equal("system", s.View);
            RenderedFrame off = s.Capture();
            _out.WriteLine(string.Join("; ", s.Themed.Stage!.Current.Scene.Entries.Where(e => e.Element.Type == "clock").Select(e => $"{e.Element.Name}: {e.Skipped ?? "drawn"}")));
            Flip(s, "DisplayClock");
            _out.WriteLine(string.Join("; ", s.Themed.Stage!.Current.Scene.Entries.Where(e => e.Element.Type == "clock").Select(e => $"{e.Element.Name}: {e.Skipped ?? "drawn"}")));
            ClockLabel clock = Only<ClockLabel>(s);
            Assert.Equal("13:45", clock.Text);
            Assert_only_inside(off, s.Capture(), BoxOf(s, clock), "Art Book Next's clock");
        }, default);

        // A session's clock follows the wall clock by itself; a stage (b) scene draws the fixed time it is given.
        [Fact]
        public Task A_session_s_clock_is_live_and_a_scene_s_clock_is_the_time_it_is_given() => Session.Dispatch(() =>
        {
            using ThemedSession s = new(extraGamelist: Clock, status: Device, settings: a => a.BigPictureInterface.DisplayClock = true);
            ThemedLibraryPadTests.Enter(s, "snes");
            ClockLabel clock = Only<ClockLabel>(s);
            Assert.True(clock.Live);
            Assert.Null(clock.Time);
        }, default);

        // The on-screen help: turned off, the help bar is not drawn and nothing else changes.
        [Fact]
        public Task Turning_the_help_off_removes_the_help_bar_and_changes_nothing_else() => Session.Dispatch(() =>
        {
            using ThemedSession s = Session4();
            Frozen(s);
            ThemedLibraryPadTests.Enter(s, "snes");
            HintBar bar = Only<HintBar>(s);
            Rect box = BoxOf(s, bar);
            RenderedFrame on = s.Capture();
            Flip(s, "DisplayHelp");
            Assert.False(s.Settings.BigPictureInterface.DisplayHelp);
            Assert.False(Has<HintBar>(s));
            Assert.Contains(s.Themed.Stage!.Current.Scene.Entries, e => e.Element.Type == "helpsystem" && e.Skipped == "the help is turned off");
            Assert_only_inside(on, s.Capture(), box, "help bar", 500);

            s.Pad.B();
            Assert.False(Has<HintBar>(s));
        }, default);

        // ES-DE's four system status switches: each takes its indicator out of the status element, and changes nothing outside the element's box.
        [Theory]
        [InlineData("StatusBluetooth", "bluetooth")]
        [InlineData("StatusWifi", "wifi")]
        [InlineData("StatusBattery", "battery_high")]
        [InlineData("StatusBatteryPercentage", "64%")]
        public Task Each_status_switch_takes_its_indicator_out_of_the_status_bar_alone(string name, string shown) => Session.Dispatch(() =>
        {
            using ThemedSession s = Session4();
            Frozen(s);
            ThemedLibraryPadTests.Enter(s, "snes");
            DeviceStatusBar Bar() => Only<DeviceStatusBar>(s);
            IEnumerable<string> Shown() => Bar().Shown().Keys.Append(Bar().Shown().Percentage ?? "");
            Assert.Contains(shown, Shown());
            Rect box = BoxOf(s, Bar());
            RenderedFrame on = s.Capture();
            Flip(s, name);
            Assert.DoesNotContain(shown, Shown());
            Assert_only_inside(on, s.Capture(), box, name);
            // The percentage is drawn beside the battery, so it goes with it.
            if (name == "StatusBattery") Assert.Null(Bar().Shown().Percentage);
        }, default);

        // Quick system select, all six of ES-DE's choices, over a list: which pair changes the system, what the others do, and what the help bar calls them.
        [Theory]
        [InlineData(BigPictureInterface.QuickSelectLeftRightOrShoulders, "leftright")]
        [InlineData(BigPictureInterface.QuickSelectLeftRightOrTriggers, "leftright")]
        [InlineData(BigPictureInterface.QuickSelectShoulders, "shoulders")]
        [InlineData(BigPictureInterface.QuickSelectTriggers, "triggers")]
        [InlineData(BigPictureInterface.QuickSelectLeftRight, "leftright")]
        [InlineData(BigPictureInterface.QuickSelectDisabled, "none")]
        public Task Quick_system_select_takes_the_pair_ES_DE_documents_for_a_list(string setting, string pair) => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(status: Device);
            ThemedLibraryPadTests.Enter(s, "snes");
            Choose(s, "QuickSystemSelect", InterfaceSettingsPane.QuickSelectChoices.Single(c => c.Value == setting).Text);
            Assert.Equal(setting, s.Settings.BigPictureInterface.QuickSystemSelect);
            Assert.Equal(("snes", "gamelist"), (s.System, s.View));
            s.Sounds.Clear();

            s.Pad.Right();
            Assert.Equal(pair == "leftright" ? "nes" : "snes", s.System);
            if (pair == "leftright") s.Pad.Left();
            Assert.Equal("snes", s.System);

            s.Pad.R1();
            Assert.Equal(pair == "shoulders" ? "nes" : "snes", s.System);
            if (pair == "shoulders") s.Pad.L1();
            else Assert.Equal(ThemedSession.SnesGames[^1], s.Game);
            Assert.Equal("snes", s.System);
            s.Pad.L2();
            Assert.Equal(pair == "triggers" ? "gb" : "snes", s.System);
            if (pair == "triggers") s.Pad.R2();
            else Assert.Equal(ThemedSession.SnesGames[0], s.Game);
            Assert.Equal("snes", s.System);
            Assert.Equal(pair != "none", s.Sounds.Contains("quicksysselect"));

            string Label(string entry) => Only<HintBar>(s).Entries!.Select((e, i) => (e, i)).Where(p => p.e.Button == entry switch
            {
                "l" => PadGlyphButton.LeftShoulder, "lt" => PadGlyphButton.LeftTrigger, _ => PadGlyphButton.DPadLeftRight,
            }).Select(p => p.e.Label).FirstOrDefault() ?? "";
            Assert.Equal(pair == "leftright" ? "System" : "", Label("left/right"));
            Assert.Equal(pair == "shoulders" ? "System" : "Jump", Label("l"));
            Assert.Equal(pair == "triggers" ? "System" : "First", Label("lt"));
        }, default);

        // ES-DE's "System on startup" and "Startup view": the first showing opens there; a system the library lacks gives the first; a later showing keeps where the player was.
        [Theory]
        [InlineData("snes", BigPictureInterface.ViewGamelist, "snes", "gamelist")]
        [InlineData("gb", BigPictureInterface.ViewSystem, "gb", "system")]
        [InlineData("n64", BigPictureInterface.ViewGamelist, "nes", "gamelist")]
        [InlineData("", BigPictureInterface.ViewSystem, "nes", "system")]
        public Task The_first_showing_opens_at_the_startup_system_and_view(string system, string view, string openedAt, string openedIn) => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(status: Device, settings: a => { a.BigPictureInterface.StartupSystem = system; a.BigPictureInterface.StartupView = view; });
            Assert.Equal((openedAt, openedIn), (s.System, s.View));
            if (s.View == "gamelist") s.Pad.B();
            s.Pad.Right();
            string moved = s.System!;
            s.Refresh();
            Assert.Equal((moved, "system"), (s.System, s.View));
        }, default);

        // ES-DE's "Systems sorting", in the three orders Mistress can answer: its own release order, full names, and each system's first release year.
        [Theory]
        [InlineData(BigPictureInterface.SortRelease, "nes,gb,gbc,snes,n64")]
        [InlineData(BigPictureInterface.SortFullNames, "gb,gbc,n64,nes,snes")]
        [InlineData(BigPictureInterface.SortReleaseYear, "nes,gb,snes,n64,gbc")]
        public Task The_systems_follow_the_chosen_order_and_the_startup_list_with_them(string sorting, string order) => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(status: Device);
            File.WriteAllBytes(Path.Combine(s.RomDirectory, "Iron Kite (Synthetic).gbc"), new byte[0x200]);
            File.WriteAllBytes(Path.Combine(s.RomDirectory, "Jade Orbit (Synthetic).z64"), new byte[0x1000]);
            s.Refresh();
            Choose(s, "SystemsSorting", EmuSen.Mistress.BigPicture.SystemsOrder.Choices.Single(c => c.Value == sorting).Text);
            Assert.Equal(order, string.Join(",", s.Themed.Stage!.Current.Data.Systems.Select(x => x.System.Name)));
            ThemeSettingsWindow sheet = OpenInterface(s);
            string[] startup = ((System.Collections.IEnumerable)Named<Dropdown>(sheet, "StartupSystem").ItemsSource!).Cast<string>().ToArray();
            Assert.Equal(7, startup.Length);
            Assert.StartsWith("Default", startup[0]);
            PutAway(s);
        }, default);

        // ES-DE's quick scrolling overlay: off by default; on, a held list shows the passing game's first two letters over a shade once its repeats begin, a star over favourites kept on top, and nothing once let go.
        [Fact]
        public Task The_quick_scrolling_overlay_shows_while_a_list_is_held_only_when_turned_on() => Session.Dispatch(() =>
        {
            using ThemedSession s = Session4("");
            Frozen(s);
            ThemedLibraryPadTests.Enter(s, "snes");
            ScrollLetterOverlay Overlay() => s.Themed.Stage!.Current.ScrollLetters;

            s.Pad.Pad.Press(SDL.GamepadButton.DPadDown);
            s.Pad.Tick();
            s.Run(600);
            Assert.False(Overlay().Showing);
            s.Run(300);
            Assert.False(Overlay().Showing);
            s.Pad.Pad.Release(SDL.GamepadButton.DPadDown);
            s.Pad.Tick();
            s.Pad.L2();

            Flip(s, "ListScrollOverlay");
            Assert.True(s.Settings.BigPictureInterface.ListScrollOverlay);
            RenderedFrame still = s.Capture();
            s.Pad.Pad.Press(SDL.GamepadButton.DPadDown);
            s.Pad.Tick();
            s.Run(300);
            Assert.False(Overlay().Showing);
            s.Run(400);
            Assert.True(Overlay().Showing);
            Assert.Equal(s.Game![..2], Overlay().Letters);
            Assert.False(Overlay().Star);
            RenderedFrame held = s.Capture();
            Rect list = BoxOf(s, Only<TextRowList>(s));
            int darker = 0, lit = 0;
            for (int y = (int)list.Top; y < (int)list.Bottom; y++)
                for (int x = (int)list.Left; x < (int)list.Right; x++)
                {
                    int i = (y * still.Width + x) * 4;
                    if (held.Rgba[i] + held.Rgba[i + 1] + held.Rgba[i + 2] < still.Rgba[i] + still.Rgba[i + 1] + still.Rgba[i + 2]) darker++;
                }
            for (int y = 350; y < 450; y++)
                for (int x = 560; x < 720; x++)
                    if (held.Rgba[(y * held.Width + x) * 4] > 200 && still.Rgba[(y * still.Width + x) * 4] < 200) lit++;
            _out.WriteLine($"overlay: {darker} pixels of the list darker, {lit} lit in the middle");
            Assert.True(darker > 1000, $"the shade darkened {darker} pixels of the list");
            Assert.True(lit > 300, $"the letters lit {lit} pixels in the middle");
            s.Pad.Pad.Release(SDL.GamepadButton.DPadDown);
            s.Pad.Tick();
            Assert.False(Overlay().Showing);

            // Favourites on top: a star instead of the letters while the list passes them, the letters again after.
            s.Pad.L2();
            for (int i = 0; i < 4; i++)
            {
                s.Pad.Y();
                Assert.True(s.Themed.SelectedGame!.Favorite);
                s.Pad.Down();
            }
            s.Pad.L2();
            s.Pad.Pad.Press(SDL.GamepadButton.DPadDown);
            s.Pad.Tick();
            s.Run(550);
            Assert.True(s.Themed.SelectedGame!.Favorite);
            Assert.True(Overlay().Showing);
            Assert.True(Overlay().Star);
            s.Run(200);
            Assert.False(s.Themed.SelectedGame!.Favorite);
            Assert.False(Overlay().Star);
            Assert.Equal(s.Game![..2], Overlay().Letters);
            s.Pad.Pad.Release(SDL.GamepadButton.DPadDown);
            s.Pad.Tick();
        }, default);

        // Navigation volume: the stream's gain is the setting over a hundred, set by Preferences' slider, and at 0 nothing is played at all.
        [Fact]
        public Task The_navigation_volume_is_the_stream_s_gain_and_zero_plays_nothing() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(status: Device);
            float Gain() => (float)typeof(MainWindow).GetProperty("UiSoundGain", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(s.Window)!;
            Assert.Equal(0.7f, Gain(), 3);
            ThemedLibraryFlowTests.Choose(s, "Preferences");
            Window prefs = Sheets(s).Current!;
            Slider slider = Named<Slider>(prefs, "NavigationVolumeSlider");
            Assert.Equal(70, slider.Value);
            slider.Value = 35;
            Assert.Equal(35, s.Settings.BigPictureInterface.NavigationVolume);
            Assert.Equal(35, AppSettings.Load().BigPictureInterface.NavigationVolume);
            Assert.Equal(0.35f, Gain(), 3);
            slider.Value = 0;
            PutAway(s);
            s.Sounds.Clear();
            s.Pad.Right();
            s.Pad.A();
            Assert.Empty(s.Sounds);
        }, default);

        // A theme with no sounds plays Mistress's own for each action, and a theme with some plays its own and Mistress's for the rest, per sound as THEMES.md says.
        [Fact]
        public Task A_theme_without_a_sound_gets_Mistress_s_own_for_it_and_keeps_the_ones_it_has() => Session.Dispatch(() =>
        {
            using (var none = new ThemedSession(status: Device, sounds: false))
            {
                ThemedLibraryPadTests.Enter(none, "snes");
                none.Pad.Down();
                none.Pad.Y();
                none.Pad.B();
                Assert.Equal(new[] { "systembrowse", "systembrowse", "select", "scroll", "favorite", "back" }.Select(n => NavigationSounds.Key(n)), none.Sounds);
            }

            using var some = new ThemedSession(status: Device);
            File.Delete(some.Theme.PathOf("sounds/scroll.wav"));
            some.Themed.Forget();
            some.Refresh();
            ThemedLibraryPadTests.Enter(some, "snes");
            some.Pad.Down();
            Assert.Equal(new[] { "systembrowse", "systembrowse", "select", NavigationSounds.Key("scroll") }, some.Sounds);
        }, default);

        // Mistress's seven sounds are its own synthesis: 48 kHz stereo floats, short, no two alike, the same every time, and none read from a file.
        [Fact]
        public void The_fallback_sounds_are_synthesised_distinct_short_and_the_same_every_time()
        {
            var seen = new HashSet<string>();
            foreach (string name in ThemedSession.SoundNames)
            {
                byte[] a = NavigationSounds.Samples(name), b = NavigationSounds.Samples(NavigationSounds.Key(name));
                Assert.Equal(a, b);
                Assert.Equal(0, a.Length % 8);
                double seconds = a.Length / 8.0 / NavigationSounds.Rate;
                Assert.InRange(seconds, 0.02, 0.3);
                float peak = Enumerable.Range(0, a.Length / 4).Max(i => Math.Abs(BitConverter.ToSingle(a, i * 4)));
                Assert.InRange(peak, 0.05f, 1f);
                Assert.True(seen.Add(Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(a))), $"{name} repeats another sound");
                _out.WriteLine($"{name}: {seconds * 1000:F0} ms, peak {peak:F2}, sha256 {Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(a))[..16]}");
            }
            Assert.Empty(NavigationSounds.Samples("nonesuch"));
        }

        // The controller chosen in the metadata editor is drawn on the controller badge in the shape of that console's pad.
        [Fact]
        public Task The_controller_chosen_in_the_editor_is_drawn_on_the_controller_badge() => Session.Dispatch(() =>
        {
            const string badges = "<badges name=\"badges\"><pos>0.55 0.8</pos><size>0.3 0.1</size><slots>controller,favorite</slots><lines>1</lines><itemsPerLine>4</itemsPerLine></badges>";
            using ThemedSession s = Session4(badges);
            ThemedLibraryPadTests.Enter(s, "snes");
            Assert.Empty(Only<BadgeStrip>(s).Entries!);
            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
            ((Dropdown)editor.EditorOf(GameMetadata.Controller)).SelectedItem = "SNES";
            ThemedGameOptionsTests.Reach(s, "MetadataSave");
            s.Pad.A();
            s.Settle();
            Assert.Equal("gamepad_nintendo_snes", ThemedGameOptionsTests.StoredEdits(s, ThemedSession.SnesGames[0] + ".sfc")[GameMetadata.Controller]);
            BadgeEntry entry = Assert.Single(Only<BadgeStrip>(s).Entries!);
            Assert.Equal((BadgeKind.Controller, ControllerShape.Snes), (entry.Kind, entry.Controller));
        }, default);

        // Pass 6's folder entries carry Mistress's own folder badge, with the link drawn over it for a linked folder, and a game carries none (§29, §30).
        [Fact]
        public Task Pass_6_s_folders_show_the_built_in_folder_badge_and_a_linked_folder_its_link() => Session.Dispatch(() =>
        {
            const string badges = "<badges name=\"badges\"><pos>0.55 0.8</pos><size>0.3 0.1</size><slots>folder,favorite</slots><lines>1</lines><itemsPerLine>4</itemsPerLine></badges>";
            using var s = new ThemedSession(extraGamelist: badges, status: Device, roms: ThemedFoldersTests.Library);
            string usa = Path.Combine(s.RomDirectory, "NES", "USA");
            ThemedCollectionsTests.Records(s).SaveEdits(usa, new Dictionary<string, string?> { [GameMetadata.FolderLink] = ThemedFoldersTests.UsaGame + ".nes" }, DateTime.Now);
            ThemedCollectionsTests.Refresh(s);
            ThemedLibraryPadTests.Enter(s, "nes");
            BadgeEntry? Only() => s.Themed.Stage!.Current.Scene.Entries.Select(e => e.Control).OfType<BadgeStrip>().Single().Entries!.SingleOrDefault();

            Assert.Equal("Europe", s.Game);
            Assert.Equal((BadgeKind.Folder, false, (string?)null), (Only()!.Kind, Only()!.Linked, Only()!.IconPath));
            RenderedFrame plain = s.Capture();
            s.Pad.Down(2);
            Assert.Equal("USA", s.Game);
            Assert.NotNull(s.Themed.SelectedGame!.FolderLink);
            Assert.Equal((BadgeKind.Folder, true), (Only()!.Kind, Only()!.Linked));
            BadgeStrip strip = s.Themed.Stage!.Current.Scene.Entries.Select(e => e.Control).OfType<BadgeStrip>().Single();
            Assert.True(BuiltInBadgesTests.Split(plain, s.Capture(), BoxOf(s, strip)).Inside > 50, "the link drew nothing over the folder");
            s.Pad.Down();
            Assert.False(s.Themed.SelectedGame!.Folder);
            Assert.Null(Only());
        }, default);

        // Every control of the Interface tab is reached by the pad, and the tab is on the sheet whatever the theme.
        [Fact]
        public Task Every_control_of_the_interface_tab_is_reached_by_the_pad() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(status: Device);
            ThemeSettingsWindow sheet = OpenInterface(s);
            Control root = Sheets(s).SheetOf(sheet)!;
            HashSet<Avalonia.Input.InputElement> reached = PadAudit.Reachable(root, s.Pad);
            var missing = PadAudit.Operable(root).Where(c => !reached.Contains(c)).Select(PadAudit.Describe).ToList();
            foreach (string m in missing) _out.WriteLine("unreachable: " + m);
            Assert.Empty(missing);
            foreach (string name in new[] { "QuickSystemSelect", "StartupSystem", "StartupView", "SystemsSorting" }) Named<Dropdown>(sheet, name);
            foreach (string name in new[] { "ListScrollOverlay", "DisplayClock", "DisplayHelp", "StatusBluetooth", "StatusWifi", "StatusBattery", "StatusBatteryPercentage" }) Named<LunaSwitch>(sheet, name);
        }, default);
    }
}
