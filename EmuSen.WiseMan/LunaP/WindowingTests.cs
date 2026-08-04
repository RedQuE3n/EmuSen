using System;
using System.IO;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using EmuSen.Galaxia;
using EmuSen.LunaP.Windowing;

namespace EmuSen.WiseMan.LunaP
{
    // ToolWindow/PollingWindow/WindowSlot - see EmuSen_LunaP.md §8.
    public class WindowingTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(WindowingTests).GetTypeInfo().Assembly);

        private readonly string _configDir;

        // Keeps remembered geometry off the developer's real config tree - the same guard Galaxia's own tests use.
        public WindowingTests()
        {
            _configDir = Path.Combine(Path.GetTempPath(), "lunap-windowing-" + Guid.NewGuid().ToString("N"));
            ConfigStore.OverrideDirectory = _configDir;
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            if (Directory.Exists(_configDir)) Directory.Delete(_configDir, recursive: true);
        }

        private sealed class CountingWindow : PollingWindow
        {
            public int Refreshes;

            public CountingWindow()
            {
                Width = 200;
                Height = 150;
                StartPolling();
            }

            protected override TimeSpan RefreshInterval => TimeSpan.FromMilliseconds(20);

            protected override void Refresh() => Refreshes++;
        }

        [Fact]
        public Task Polling_primes_itself_once_before_any_tick() => Session.Dispatch(() =>
        {
            var window = new CountingWindow();

            // Constructed but never shown: the priming Refresh has run, the timer has not.
            Assert.Equal(1, window.Refreshes);
        }, default);

        // Asserted on the timer's own state rather than a tick count: a count-based test would have to race a real clock to mean anything.
        [Fact]
        public Task Polling_stops_while_the_window_is_hidden() => Session.Dispatch(() =>
        {
            var window = new CountingWindow();

            // Never shown: primed, but nothing scheduled.
            Assert.False(window.IsPolling);

            window.Show();
            Assert.True(window.IsPolling);

            window.Hide();
            Assert.False(window.IsPolling);

            int atHide = window.Refreshes;
            window.Show();

            Assert.True(window.IsPolling);

            // Coming back repaints immediately rather than showing however stale it got.
            Assert.True(window.Refreshes > atHide);
        }, default);

        [Fact]
        public Task Polling_stops_while_the_window_is_minimised() => Session.Dispatch(() =>
        {
            var window = new CountingWindow();
            window.Show();
            Assert.True(window.IsPolling);

            window.WindowState = WindowState.Minimized;
            Assert.False(window.IsPolling);

            int atMinimise = window.Refreshes;
            window.WindowState = WindowState.Normal;

            Assert.True(window.IsPolling);
            Assert.True(window.Refreshes > atMinimise);
        }, default);

        [Fact]
        public Task Closing_a_polling_window_stops_it() => Session.Dispatch(() =>
        {
            var window = new CountingWindow();
            window.Show();
            Assert.True(window.IsPolling);

            window.Close();

            // A timer left running holds the closed window alive and keeps calling into it.
            Assert.False(window.IsPolling);
        }, default);

        [Fact]
        public Task A_polling_window_that_never_called_start_still_polls_once_opened() => Session.Dispatch(() =>
        {
            var window = new ForgetfulWindow();
            Assert.Equal(0, window.Refreshes);

            window.Show();
            Assert.True(window.Refreshes > 0);
        }, default);

        private sealed class ForgetfulWindow : PollingWindow
        {
            public int Refreshes;

            public ForgetfulWindow()
            {
                Width = 200;
                Height = 150;
            }

            protected override TimeSpan RefreshInterval => TimeSpan.FromMilliseconds(20);

            protected override void Refresh() => Refreshes++;
        }

        [Fact]
        public Task Escape_closes_only_when_a_window_opts_in() => Session.Dispatch(() =>
        {
            var closing = new ToolWindow { Width = 200, Height = 150, ClosesOnEscape = true };
            closing.Show();
            closing.RaiseEvent(new KeyEventArgs { RoutedEvent = InputElement.KeyDownEvent, Key = Key.Escape });
            Assert.False(closing.IsVisible);

            var staying = new ToolWindow { Width = 200, Height = 150 };
            staying.Show();
            staying.RaiseEvent(new KeyEventArgs { RoutedEvent = InputElement.KeyDownEvent, Key = Key.Escape });
            Assert.True(staying.IsVisible);
        }, default);

        [Fact]
        public Task A_window_with_no_key_is_never_remembered() => Session.Dispatch(() =>
        {
            var window = new ToolWindow { Width = 321, Height = 234 };
            window.Show();
            window.Close();

            Assert.False(new ConfigFile<System.Collections.Generic.Dictionary<string, WindowPlacement>>("windows.json").Exists);
        }, default);

        [Fact]
        public Task A_keyed_window_remembers_its_size() => Session.Dispatch(() =>
        {
            var window = new ToolWindow { WindowKey = "test.sized", Width = 321, Height = 234 };
            window.Show();
            window.Close();

            WindowPlacement? saved = WindowPlacementStore.Load("test.sized");
            Assert.NotNull(saved);
            Assert.Equal(321, saved!.Width);
            Assert.Equal(234, saved.Height);

            var reopened = new ToolWindow { WindowKey = "test.sized", Width = 100, Height = 100 };
            reopened.Show();

            Assert.Equal(321, reopened.Width);
            Assert.Equal(234, reopened.Height);
            reopened.Close();
        }, default);

        // A monitor that is no longer attached must not strand a window where it cannot be dragged back.
        [Theory]
        // Fully inside the primary screen.
        [InlineData(100, 100, 400, 300, true)]
        // Straddling the edge - still reachable, so still allowed.
        [InlineData(1800, 500, 400, 300, true)]
        // On a second monitor that is still attached.
        [InlineData(2200, 100, 400, 300, true)]
        // Where a now-unplugged monitor used to be.
        [InlineData(4000, 100, 400, 300, false)]
        // Negative space above and left of everything.
        [InlineData(-900, -900, 400, 300, false)]
        public void An_off_screen_position_is_rejected_against_the_attached_screens(
            int x, int y, int width, int height, bool expected)
        {
            var screens = new[]
            {
                new PixelRect(0, 0, 1920, 1080),
                new PixelRect(1920, 0, 1920, 1080),
            };

            Assert.Equal(expected, WindowPlacementStore.IsOnAScreen(screens, new PixelRect(x, y, width, height)));
        }

        // Not the same as "off screen": with nothing to check against, refusing would strand the window at the default position.
        [Fact]
        public void No_known_screens_allows_the_remembered_position()
        {
            Assert.True(WindowPlacementStore.IsOnAScreen(Array.Empty<PixelRect>(), new PixelRect(4000, 4000, 100, 100)));
            Assert.True(WindowPlacementStore.IsOnAScreen((Screens?)null, new PixelRect(4000, 4000, 100, 100)));
        }

        [Fact]
        public Task A_slot_opens_one_window_and_then_reuses_it() => Session.Dispatch(() =>
        {
            var slot = new WindowSlot<ToolWindow>();
            int created = 0;
            int refreshed = 0;

            slot.Show(null, () => { created++; return new ToolWindow { Width = 200, Height = 150 }; });
            ToolWindow? first = slot.Current;

            slot.Show(null, () => { created++; return new ToolWindow { Width = 200, Height = 150 }; }, _ => refreshed++);

            Assert.Equal(1, created);
            Assert.Equal(1, refreshed);
            Assert.Same(first, slot.Current);

            slot.Close();
            Assert.False(slot.IsOpen);
            Assert.Null(slot.Current);
        }, default);

        [Fact]
        public Task A_slot_reopens_after_the_window_was_closed_by_the_user() => Session.Dispatch(() =>
        {
            var slot = new WindowSlot<ToolWindow>();

            slot.Show(null, () => new ToolWindow { Width = 200, Height = 150 });
            ToolWindow first = slot.Current!;
            first.Close();

            Assert.False(slot.IsOpen);

            slot.Show(null, () => new ToolWindow { Width = 200, Height = 150 });
            Assert.True(slot.IsOpen);
            Assert.NotSame(first, slot.Current);
            slot.Close();
        }, default);

        // The core-swap case: refresh what is open, but never conjure a window nobody asked for.
        [Fact]
        public Task Refresh_if_open_never_creates_a_window() => Session.Dispatch(() =>
        {
            var slot = new WindowSlot<ToolWindow>();
            int refreshed = 0;

            slot.RefreshIfOpen(_ => refreshed++);
            Assert.False(slot.IsOpen);
            Assert.Equal(0, refreshed);

            slot.Show(null, () => new ToolWindow { Width = 200, Height = 150 });
            slot.RefreshIfOpen(_ => refreshed++);
            Assert.Equal(1, refreshed);

            slot.Close();
        }, default);
    }
}
