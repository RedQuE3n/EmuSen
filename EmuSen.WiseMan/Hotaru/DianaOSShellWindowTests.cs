using System;
using System.Collections.Generic;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using EmuSen.Hotaru.Views;

namespace EmuSen.WiseMan.Hotaru
{
    // The windowed shell a desktop launch gets - see EmuSen_Frontend_Driver.md §3c.
    public class DianaOSShellWindowTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(DianaOSShellWindowTests).GetTypeInfo().Assembly);

        private static SelectableTextBlock Output(DianaOSShellWindow w) => w.GetControl<SelectableTextBlock>("OutputText");
        private static TextBox Input(DianaOSShellWindow w) => w.GetControl<TextBox>("InputBox");
        private static TextBlock Prompt(DianaOSShellWindow w) => w.GetControl<TextBlock>("PromptText");

        private static void Type(DianaOSShellWindow w, string line)
        {
            Input(w).Text = line;
            w.KeyPress(Key.Enter, RawInputModifiers.None, PhysicalKey.Enter, null);
        }

        [Fact]
        public Task It_opens_on_the_banner_and_a_prompt() => Session.Dispatch(() =>
        {
            var window = new DianaOSShellWindow(null, null, new[] { "SNES (Venus)" });
            window.Show();

            Assert.Contains("Welcome to DianaOS", Output(window).Text!);
            Assert.Contains("SNES (Venus)", Output(window).Text!);
            Assert.Equal("DianaOS #: ", Prompt(window).Text);
        }, default);

        [Fact]
        public Task It_runs_a_real_command_against_the_interpreter() => Session.Dispatch(() =>
        {
            var window = new DianaOSShellWindow(null, null, null);
            window.Show();

            Type(window, "echo hello-from-the-window");

            Assert.Contains("DianaOS #: echo hello-from-the-window", Output(window).Text!);
            Assert.Contains("hello-from-the-window", Output(window).Text!);
            Assert.Equal("", Input(window).Text ?? "");
        }, default);

        // Without this the window is a dead end: no core, and no way to reach one.
        [Fact]
        public Task A_core_command_launches_the_rom_the_resolver_returns() => Session.Dispatch(() =>
        {
            string? launched = null;
            var window = new DianaOSShellWindow(
                romPath => { launched = romPath; return new Window(); },
                line => "/roms/resolved.smc",
                null);
            window.Show();

            Type(window, "core venus /roms/whatever.smc");

            Assert.Equal("/roms/resolved.smc", launched);
        }, default);

        [Fact]
        public Task A_refused_core_command_reports_and_launches_nothing() => Session.Dispatch(() =>
        {
            bool launched = false;
            var window = new DianaOSShellWindow(
                _ => { launched = true; return new Window(); },
                line => null, // the resolver refused and reported for itself
                null);
            window.Show();

            Type(window, "core venus /roms/missing.smc");

            Assert.False(launched);
        }, default);

        // A ROM that throws on load must not take the shell down with it.
        [Fact]
        public Task A_rom_that_fails_to_load_reports_and_leaves_the_window_open() => Session.Dispatch(() =>
        {
            var window = new DianaOSShellWindow(
                _ => throw new InvalidOperationException("unsupported mapper"),
                _ => "/roms/broken.smc",
                null);
            bool closed = false;
            window.Closed += (_, _) => closed = true;
            window.Show();

            Type(window, "core venus /roms/broken.smc");

            Assert.Contains("unsupported mapper", Output(window).Text!);
            Assert.False(closed);
        }, default);

        [Fact]
        public Task Up_arrow_recalls_the_previous_line() => Session.Dispatch(() =>
        {
            var window = new DianaOSShellWindow(null, null, null);
            window.Show();
            Type(window, "echo first");

            Input(window).Focus();
            window.KeyPress(Key.Up, RawInputModifiers.None, PhysicalKey.ArrowUp, null);

            Assert.Equal("echo first", Input(window).Text);
        }, default);

        [Fact]
        public Task Clear_empties_this_window_rather_than_a_terminal() => Session.Dispatch(() =>
        {
            var window = new DianaOSShellWindow(null, null, null);
            window.Show();
            Type(window, "echo something");
            Assert.Contains("something", Output(window).Text!);

            Type(window, "clear");

            Assert.True(string.IsNullOrEmpty(Output(window).Text));
        }, default);

        [Fact]
        public Task Shutdown_closes_the_window() => Session.Dispatch(() =>
        {
            var window = new DianaOSShellWindow(null, null, null);
            bool closed = false;
            window.Closed += (_, _) => closed = true;
            window.Show();

            Type(window, "shutdown");

            Assert.True(closed);
        }, default);

        // Once a game starts, every line has to go to the running core, not the pre-game interpreter.
        [Fact]
        public Task An_attached_live_shell_takes_over_every_line() => Session.Dispatch(() =>
        {
            var live = new FakeLiveShell();
            var window = new DianaOSShellWindow(null, null, null);
            window.Show();
            window.AttachLiveShell(live);

            Type(window, "regs");

            Assert.Equal(new[] { "regs" }, live.Submitted);
            Assert.Contains("attached to the running core", Output(window).Text!);
        }, default);

        [Fact]
        public Task Output_raised_from_the_emulation_thread_reaches_the_window() => Session.Dispatch(() =>
        {
            var live = new FakeLiveShell();
            var window = new DianaOSShellWindow(null, null, null);
            window.Show();
            window.AttachLiveShell(live);

            live.Emit("A = 0x1234");
            Avalonia.Threading.Dispatcher.UIThread.RunJobs();

            Assert.Contains("A = 0x1234", Output(window).Text!);
        }, default);

        [Fact]
        public Task An_attached_shell_recalls_the_running_cores_history() => Session.Dispatch(() =>
        {
            var live = new FakeLiveShell();
            live.History = new[] { "mem WRAM 0 16" };
            var window = new DianaOSShellWindow(null, null, null);
            window.Show();
            window.AttachLiveShell(live);

            Input(window).Focus();
            window.KeyPress(Key.Up, RawInputModifiers.None, PhysicalKey.ArrowUp, null);

            Assert.Equal("mem WRAM 0 16", Input(window).Text);
        }, default);

        private sealed class FakeLiveShell : ILiveShell
        {
            public List<string> Submitted { get; } = new();
            public string[] History { get; set; } = Array.Empty<string>();

            public bool IsAwaitingMoreInput => false;
            public string[] SnapshotHistory() => History;
            public void Submit(string line) => Submitted.Add(line);
            public event Action<string>? Output;
            public void Emit(string text) => Output?.Invoke(text);
        }
    }
}
