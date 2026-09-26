using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.VisualTree;
using EmuSen.Endymion;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The render loop's idleness, the help bar's pad family, the sounds and their switch, Preferences, and every sheet reached by the pad in the themed session - see EmuSen_BigPicture.md §15.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemedLibraryHostTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ThemedLibraryHostTests).GetTypeInfo().Assembly);

        private readonly ITestOutputHelper _out;

        public ThemedLibraryHostTests(ITestOutputHelper output) => _out = output;

        private static void Refresh(ThemedSession s)
        {
            typeof(MainWindow).GetMethod("RefreshLibrary", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(s.Window, null);
            s.Settle();
        }

        // P39: a still view asks for no frame, a moving one for every frame, and a text in its pause for none until the pause ends.
        [Fact]
        public Task A_still_view_draws_nothing_a_moving_one_draws_each_frame_and_a_pause_sleeps_until_it_ends() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            Assert.Null(s.WakeAt);
            Assert.Equal(0, s.Loop(5000));

            s.Pad.Right();
            Assert.Equal(s.Now, s.WakeAt);
            int moving = s.Loop(600);
            _out.WriteLine($"carousel step: {moving} frames in 600 ms");
            Assert.InRange(moving, 20, 30);
            Assert.Null(s.WakeAt);
            Assert.Equal(0, s.Loop(3000));

            File.WriteAllBytes(Path.Combine(s.RomDirectory, "A Name Far Too Long To Fit In The List Of This Theme At All (Synthetic).sfc"), SyntheticRom.BuildBlank());
            Refresh(s);
            ThemedLibraryPadTests.Enter(s, "snes");
            Assert.StartsWith("A Name Far Too Long", s.Game);
            s.Settle();
            TimeSpan entered = s.Now;
            Assert.Equal(entered + TimeSpan.FromSeconds(3), s.WakeAt);
            Assert.Equal(0, s.Loop(2900));
            int scrolling = s.Loop(400);
            Assert.InRange(scrolling, 5, 25);

            // A short name that fits never scrolls, so the view sleeps for good.
            s.Pad.Down();
            s.Settle();
            Assert.Null(s.WakeAt);
            Assert.Equal(0, s.Loop(8000));
        }, default);

        // A text container waits its start delay before it moves, and the loop sleeps through the wait; ES-DE's clock is off by default, whatever the theme says (§13.8).
        [Fact]
        public Task A_text_container_wakes_the_loop_at_its_delay_and_the_clock_stays_off() => Session.Dispatch(() =>
        {
            const string extra = "<text name=\"ticker\"><pos>0.55 0.85</pos><size>0.2 0.05</size><fontSize>0.04</fontSize>" +
                                 "<text>A literal line much too long for the small box it has been given in this theme</text>" +
                                 "<container>true</container><containerType>horizontal</containerType><containerStartDelay>2</containerStartDelay></text>" +
                                 "<clock name=\"clock\"><pos>0.9 0.02</pos><fontSize>0.04</fontSize></clock>";
            using var s = new ThemedSession(extraGamelist: extra);
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Settle();
            Assert.DoesNotContain(s.Themed.Stage!.Current.Scene.Entries, e => e.Control is ClockLabel);
            Assert.Contains(s.Themed.Stage.Current.Scene.Entries, e => e.Element.Type == "clock" && e.Skipped is not null);
            Assert.Equal(s.Now + TimeSpan.FromSeconds(2), s.WakeAt);
            Assert.Equal(0, s.Loop(1950));
            Assert.InRange(s.Loop(200), 5, 15);
        }, default);

        // A library of thousands, with a media folder that has none of them: the media scan is paid once, not on every return to the library.
        [Fact]
        public Task A_large_library_is_scanned_for_media_once_and_not_on_every_showing() => Session.Dispatch(() =>
        {
            string media = Path.Combine(Path.GetTempPath(), "EmuSenEmptyMedia", Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(media);
            using var s = new ThemedSession(settings: a => a.EsdeMediaDirectory = media);
            for (int i = 0; i < 3500; i++) File.WriteAllBytes(Path.Combine(s.RomDirectory, $"Filler {i:D4}.nes"), []);
            var clock = System.Diagnostics.Stopwatch.StartNew();
            Refresh(s);
            double first = clock.Elapsed.TotalMilliseconds, firstScan = s.Themed.MediaTime.TotalMilliseconds, firstTheme = s.Themed.LoadTime.TotalMilliseconds;
            clock.Restart();
            Refresh(s);
            double second = clock.Elapsed.TotalMilliseconds;
            _out.WriteLine($"3,508 games: first refresh {first:F0} ms (media scan {firstScan:F0} ms, themes {firstTheme:F1} ms); second {second:F0} ms (media scan {s.Themed.MediaTime.TotalMilliseconds:F2} ms, themes {s.Themed.LoadTime.TotalMilliseconds:F2} ms)");
            Assert.True(s.Shown);
            Assert.True(s.Themed.MediaTime.TotalMilliseconds < firstScan / 10, $"{s.Themed.MediaTime.TotalMilliseconds} against {firstScan}");
            Directory.Delete(media, true);
        }, default);

        [Fact]
        public void SDL_s_pad_types_and_names_give_the_families()
        {
            var expected = new Dictionary<string, PadFamily>
            {
                ["Unknown"] = PadFamily.Generic, ["Standard"] = PadFamily.Generic, ["Xbox360"] = PadFamily.Xbox, ["XboxOne"] = PadFamily.Xbox,
                ["PS3"] = PadFamily.PlayStation, ["PS4"] = PadFamily.PlayStation, ["PS5"] = PadFamily.PlayStation,
                ["NintendoSwitchPro"] = PadFamily.Nintendo, ["NintendoSwitchJoyconLeft"] = PadFamily.Nintendo, ["NintendoSwitchJoyconRight"] = PadFamily.Nintendo,
                ["NintendoSwitchJoyconPair"] = PadFamily.Nintendo, ["GameCube"] = PadFamily.Nintendo,
            };
            foreach (SDL.GamepadType type in Enum.GetValues<SDL.GamepadType>().Where(t => t.ToString() != "Count"))
            {
                Assert.True(expected.ContainsKey(type.ToString()), $"SDL has a pad type {type} this test does not classify");
                Assert.Equal(expected[type.ToString()], PadFamilies.Of(type, "Some pad"));
            }

            Assert.Equal(PadFamily.Xbox, PadFamilies.Of(SDL.GamepadType.Standard, "Lenovo Legion Go S"));
            Assert.Equal(PadFamily.Xbox, PadFamilies.Of(SDL.GamepadType.Unknown, "Steam Deck"));
            Assert.Equal(PadFamily.PlayStation, PadFamilies.Of(SDL.GamepadType.Unknown, "Sony DualSense Wireless Controller"));
            Assert.Equal(PadFamily.Nintendo, PadFamilies.Of(SDL.GamepadType.Standard, "Nintendo Switch Pro Controller"));
            Assert.Equal(PadFamily.Generic, PadFamilies.Of(SDL.GamepadType.Standard, "8BitDo Zero 2"));
            Assert.Equal(PadFamily.Generic, PadFamilies.Of(SDL.GamepadType.Unknown, null));
            // SDL's type wins over a name that says otherwise.
            Assert.Equal(PadFamily.PlayStation, PadFamilies.Of(Enum.Parse<SDL.GamepadType>("PS4"), "Xbox-style adapter"));
        }

        // P43: another family redraws the help bar and nothing else.
        [Fact]
        public Task The_help_bar_follows_the_connected_pad_s_family_and_nothing_else_changes() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            ThemedLibraryPadTests.Enter(s, "snes");
            HintBar Bar() => s.Themed.Stage!.Current.Scene.Entries.Select(e => e.Control).OfType<HintBar>().Single();
            Assert.Equal(PadFamily.Generic, Bar().PadFamily);
            RenderedFrame generic = s.Capture();

            var seen = new List<(PadFamily Family, RenderedFrame Frame)> { (PadFamily.Generic, generic) };
            foreach ((string type, string name, PadFamily family) in new[] { ("PS5", "DualSense", PadFamily.PlayStation), ("NintendoSwitchPro", "Pro Controller", PadFamily.Nintendo), ("Standard", "Lenovo Legion Go S", PadFamily.Xbox) })
            {
                s.Pad.Pad.Type = Enum.Parse<SDL.GamepadType>(type);
                s.Pad.Pad.Name = name;
                s.Pad.Tick();
                Assert.Equal(family, Bar().PadFamily);
                RenderedFrame frame = s.Capture();
                var box = new Rect(Bar().TranslatePoint(default, s.Window)!.Value, Bar().Bounds.Size);
                int inside = 0, outside = 0;
                for (int y = 0; y < frame.Height; y++)
                    for (int x = 0; x < frame.Width; x++)
                    {
                        int i = (y * frame.Width + x) * 4;
                        if (frame.Rgba[i] == generic.Rgba[i] && frame.Rgba[i + 1] == generic.Rgba[i + 1] && frame.Rgba[i + 2] == generic.Rgba[i + 2]) continue;
                        if (box.Inflate(1).Contains(new Point(x + 0.5, y + 0.5))) inside++; else outside++;
                    }
                _out.WriteLine($"{family}: {inside} pixels changed in the help bar, {outside} outside it");
                Assert.True(inside > 50);
                Assert.Equal(0, outside);
                seen.Add((family, frame));
            }

            // The view's own state did not move.
            Assert.Equal("snes", s.System);
            Assert.Equal(ThemedSession.SnesGames[0], s.Game);
        }, default);

        [Fact]
        public Task With_the_switch_off_no_sound_reaches_the_stream() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(settings: a => a.NavigationSounds = false);
            s.Pad.Right();
            s.Pad.A();
            s.Pad.Down();
            s.Pad.Select();
            s.Pad.B();
            Assert.Equal("system", s.View);
            Assert.Empty(s.Sounds);
        }, default);

        // P44's second half: a theme's WAV decoded once to the stream's format.
        [Fact]
        public void A_navigation_sound_is_decoded_once_to_the_stream_s_format()
        {
            string wav = Path.Combine(Path.GetTempPath(), "EmuSenUiSound", Guid.NewGuid().ToString("N") + ".wav");
            ThemedSession.Wav(wav, 440, rate: 22050, seconds: 0.1);
            byte[]? decoded = UiSoundPlayer.Decode(wav);
            Assert.NotNull(decoded);
            // 0.1 s at 48 kHz, two channels of 32-bit floats; SDL's resampler may round by a frame or so.
            Assert.InRange(decoded!.Length, 4800 * 8 - 64, 4800 * 8 + 64);
            Assert.Null(UiSoundPlayer.Decode(wav + ".missing"));
            File.Delete(wav);
        }

        // The second stream on whatever device SDL opens: a new sound replaces the one still queued rather than waiting behind it.
        [Fact]
        public void A_new_sound_replaces_the_one_still_playing_on_the_interface_s_stream()
        {
            string folder = Path.Combine(Path.GetTempPath(), "EmuSenUiStream", Guid.NewGuid().ToString("N"));
            string longer = Path.Combine(folder, "long.wav"), shorter = Path.Combine(folder, "short.wav");
            ThemedSession.Wav(longer, 330, seconds: 2);
            ThemedSession.Wav(shorter, 660, seconds: 0.5);
            // Silent, so a test run plays nothing on the machine's speakers; the gain is applied after the queue.
            using var player = new UiSoundPlayer { Volume = 0 };
            player.Play(longer);
            if (!player.IsOpen)
            {
                _out.WriteLine("SDL opened no playback device here; nothing to measure.");
                return;
            }

            int second = UiSoundPlayer.Decode(shorter)!.Length, whole = UiSoundPlayer.Decode(longer)!.Length;
            Assert.InRange(player.Queued, whole / 2, whole);
            player.Play(shorter);
            _out.WriteLine($"queued after the second sound: {player.Queued} bytes; the second alone is {second}, the first {whole}");
            Assert.InRange(player.Queued, second / 2, second);
            Directory.Delete(folder, true);
        }

        // The machine's own battery, not a mouse's; Wi-Fi by a wireless interface's state; Bluetooth by its radio switch.
        [Fact]
        public void The_device_status_is_read_from_sysfs_and_nothing_is_invented()
        {
            string root = Path.Combine(Path.GetTempPath(), "EmuSenSysfs", Guid.NewGuid().ToString("N"));
            void Put(string relative, string text)
            {
                string path = Path.Combine(root, relative);
                Directory.CreateDirectory(Path.GetDirectoryName(path)!);
                File.WriteAllText(path, text + "\n");
            }

            Assert.Equal(new DeviceStatus(), EmuSen.Mistress.BigPicture.DeviceStatusReader.Read(root));
            Put("power_supply/hidpp_battery_0/type", "Battery");
            Put("power_supply/hidpp_battery_0/scope", "Device");
            Put("power_supply/hidpp_battery_0/capacity", "15");
            Put("power_supply/BAT0/type", "Battery");
            Put("power_supply/BAT0/capacity", "64");
            Put("power_supply/BAT0/status", "Charging");
            Put("net/wlan0/operstate", "up");
            Directory.CreateDirectory(Path.Combine(root, "net/wlan0/wireless"));
            Put("net/eth0/operstate", "up");
            Put("rfkill/rfkill0/type", "bluetooth");
            Put("rfkill/rfkill0/soft", "1");
            Put("rfkill/rfkill0/hard", "0");
            Assert.Equal(new DeviceStatus(Bluetooth: false, Wifi: true, BatteryPercent: 64, Charging: true), EmuSen.Mistress.BigPicture.DeviceStatusReader.Read(root));
            Put("net/wlan0/operstate", "down");
            Put("rfkill/rfkill0/soft", "0");
            Assert.Equal(new DeviceStatus(Bluetooth: true, Wifi: false, BatteryPercent: 64, Charging: true), EmuSen.Mistress.BigPicture.DeviceStatusReader.Read(root));
            Directory.Delete(root, true);
        }

        [Fact]
        public Task Preferences_chooses_the_library_style_and_the_session_follows_it_when_the_sheet_closes() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            ThemedLibraryFlowTests.Choose(s, "Preferences");
            var sheets = s.Window.GetControl<SheetLayer>("Sheets");
            Assert.IsType<PreferencesWindow>(sheets.Current);
            Assert.Empty(s.Window.OwnedWindows);
            Control sheet = sheets.SheetOf(sheets.Current!)!;
            var tabs = sheet.GetVisualDescendants().OfType<TabControl>().First();
            for (int i = 0; i < 6 && (tabs.SelectedItem as TabItem)?.Header as string != "Appearance"; i++) s.Pad.R1();

            PadAudit.Reach(sheet, s.Pad, e => e is LunaSwitch { Name: "NavigationSoundsSwitch" });
            s.Pad.A();
            Assert.False(AppSettings.Load().NavigationSounds);
            PadAudit.Reach(sheet, s.Pad, e => e is Dropdown { Name: "LibraryStyleDropdown" });
            s.Pad.Left();
            Assert.Equal(AppSettings.LibraryStyleMistress, AppSettings.Load().LibraryStyle);
            Assert.True(s.Shown);

            s.Pad.B();
            Assert.False(sheets.IsPresenting);
            s.Settle();
            Assert.False(s.Shown);
            Assert.True(s.Window.GetControl<Control>("LibraryContent").IsVisible);
        }, default);

        // P45: the themed session's pad menu reaches every sheet, and every control on each is reached by the pad at 1280 by 800.
        [Theory]
        [InlineData("Cheats")]
        [InlineData("Graphics Settings")]
        [InlineData("Shaders")]
        [InlineData("Controller Bindings")]
        [InlineData("Preferences")]
        public Task Every_control_of_each_sheet_opened_from_the_themed_view_is_reached_by_the_pad(string entry) => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            ThemedLibraryFlowTests.Choose(s, entry);
            var sheets = s.Window.GetControl<SheetLayer>("Sheets");
            Assert.True(sheets.IsPresenting);
            Assert.Empty(s.Window.OwnedWindows);
            Assert.True(s.Shown);
            Control sheet = sheets.SheetOf(sheets.Current!)!;
            TabControl? tabs = sheet.GetVisualDescendants().OfType<TabControl>().FirstOrDefault();
            var missing = new List<string>();
            for (int page = 0; page < (tabs?.ItemCount ?? 1); page++)
            {
                if (page > 0) s.Pad.R1();
                s.Window.UpdateLayout();
                HashSet<InputElement> reached = PadAudit.Reachable(sheet, s.Pad);
                missing.AddRange(PadAudit.Operable(sheet).Where(c => !reached.Contains(c)).Select(c => $"[{(tabs?.SelectedItem as TabItem)?.Header}] {PadAudit.Describe(c)}"));
            }
            foreach (string m in missing) _out.WriteLine("unreachable: " + m);
            Assert.Empty(missing);

            s.Pad.B();
            Assert.False(sheets.IsPresenting);
            s.Pad.Right();
            Assert.Equal("gb", s.System);
        }, default);
    }
}
