using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Threading;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // The runtime's questions before a launch are asked off the interface thread, so a slow core's answer leaves the screen answering - see EmuSen_LibretroHost.md §10r.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class PreLaunchQueryTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session = HeadlessUnitTestSession.GetOrStartForAssembly(typeof(PreLaunchQueryTests).GetTypeInfo().Assembly);
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenPreLaunch_" + Guid.NewGuid().ToString("N"));
        private static FieldInfo Seam(string name) => typeof(MainWindow).GetField(name, BindingFlags.Static | BindingFlags.NonPublic)!;
        private readonly object _awaiting = Seam("AwaitingQuery").GetValue(null)!, _firmware = Seam("FirmwareQuery").GetValue(null)!, _cards = Seam("CardsQuery").GetValue(null)!;

        public PreLaunchQueryTests()
        {
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            Directory.CreateDirectory(Path.Combine(_root, "Roms"));
            new AppSettings { RomDirectory = Path.Combine(_root, "Roms"), LogDirectory = Path.Combine(_root, "Logs"), ResumeOnLaunch = AppSettings.ResumeNever }.Save();
        }

        private readonly object _confirm = Seam("ConfirmCoreApproval").GetValue(null)!;

        public void Dispose()
        {
            Seam("ConfirmCoreApproval").SetValue(null, _confirm);
            Seam("AwaitingQuery").SetValue(null, _awaiting);
            Seam("FirmwareQuery").SetValue(null, _firmware);
            Seam("CardsQuery").SetValue(null, _cards);
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        [Fact]
        public Task A_slow_firmware_answer_leaves_the_interface_thread_free() => Session.Dispatch(() =>
        {
            string rom = Path.Combine(_root, "Roms", "Loop.sfc");
            File.WriteAllBytes(rom, SyntheticRom.Build((5, new byte[] { 0x4C, 0x05, 0x80 })));
            var onInterface = new List<(string, bool)>();
            using var release = new ManualResetEventSlim();
            void Asked(string what) { lock (onInterface) onInterface.Add((what, Dispatcher.UIThread.CheckAccess())); }
            // The approval's answer is a record of Mistress's own, so its "none" is built as that type.
            Type awaiting = Seam("AwaitingQuery").FieldType;
            var rom0 = System.Linq.Expressions.Expression.Parameter(typeof(string));
            Seam("AwaitingQuery").SetValue(null, System.Linq.Expressions.Expression.Lambda(awaiting, System.Linq.Expressions.Expression.Block(
                System.Linq.Expressions.Expression.Invoke(System.Linq.Expressions.Expression.Constant((Action)(() => Asked("approval")))),
                System.Linq.Expressions.Expression.Constant(null, awaiting.GetGenericArguments()[1])), rom0).Compile());
            Seam("FirmwareQuery").SetValue(null, (Func<string, string>)(_ => { Asked("firmware"); release.Wait(TimeSpan.FromSeconds(10)); return "[]"; }));
            Seam("CardsQuery").SetValue(null, (Func<string, IReadOnlyList<CardMedium>>)(_ => { Asked("cards"); return Array.Empty<CardMedium>(); }));
            var window = new MainWindow();
            window.Show();
            var prepared = (Task)typeof(MainWindow).GetMethod("PrepareGameAsync", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, new object[] { rom })!;
            bool answered = false;
            Dispatcher.UIThread.Post(() => answered = true);
            var clock = Stopwatch.StartNew();
            while (!answered && clock.ElapsedMilliseconds < 5000) { Dispatcher.UIThread.RunJobs(); Thread.Sleep(5); }
            Assert.True(answered, "the interface thread runs its jobs while the firmware question is unanswered");
            Assert.False(prepared.IsCompleted);
            release.Set();
            while (!prepared.IsCompleted && clock.ElapsedMilliseconds < 10_000) { Dispatcher.UIThread.RunJobs(); Thread.Sleep(5); }
            Assert.True(prepared.IsCompletedSuccessfully);
            Assert.Equal(new[] { ("approval", false), ("firmware", false), ("cards", false) }, onInterface);
            window.Close();
        }, default);

        // Not Now on a console's waiting default opens the game on the route's engine and says why in one line, on the status and once the game is on screen (EmuSen_LibretroHost.md §10t).
        [Fact]
        public Task Not_now_on_a_waiting_default_says_the_game_opens_on_the_fallback() => Session.Dispatch(() =>
        {
            Type awaiting = Seam("AwaitingQuery").FieldType, record = awaiting.GetGenericArguments()[1];
            object core = Activator.CreateInstance(record, "pcee2_libretro.so", "Sony - PlayStation 2 (PCEE2)", new[] { "Someone" }, "GPLv3", new[] { "ps2" }, new string('a', 64))!;
            record.GetProperty("Instead")!.SetValue(core, "Sapphire");
            var rom0 = System.Linq.Expressions.Expression.Parameter(typeof(string));
            Seam("AwaitingQuery").SetValue(null, System.Linq.Expressions.Expression.Lambda(awaiting, System.Linq.Expressions.Expression.Constant(core, record), rom0).Compile());
            string? asked = null;
            Seam("ConfirmCoreApproval").SetValue(null, (Func<MainWindow, string, string, Task<bool>>)((_, _, message) => { asked = message; return Task.FromResult(false); }));
            var window = new MainWindow();
            window.Show();
            var ask = (Task)typeof(MainWindow).GetMethod("AskCoreApprovalAsync", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, new object[] { Path.Combine(_root, "Roms", "Jak.iso") })!;
            var clock = Stopwatch.StartNew();
            while (!ask.IsCompleted && clock.ElapsedMilliseconds < 5000) { Dispatcher.UIThread.RunJobs(); Thread.Sleep(5); }
            Assert.True(ask.IsCompletedSuccessfully);
            Assert.Contains(new string('a', 64), CoresWindowTests.Ungrouped(asked!));
            const string said = "Sony - PlayStation 2 (PCEE2) waits for your approval, so this game opens on Sapphire.";
            Assert.Equal(said, window.GetControl<TextBlock>("StatusText").Text);
            Assert.Equal(said, typeof(MainWindow).GetField("_engineFallbackNotice", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window));
            window.Close();
        }, default);

        // A console's approved default that could not be read: nothing is asked, and the game opens on the fallback with the reason in one line (EmuSen_LibretroHost.md §10v).
        [Fact]
        public Task A_held_default_is_said_with_its_reason_and_not_asked_about() => Session.Dispatch(() =>
        {
            Type awaiting = Seam("AwaitingQuery").FieldType, record = awaiting.GetGenericArguments()[1];
            object core = Activator.CreateInstance(record, "pcee2_libretro.so", "Sony - PlayStation 2 (PCEE2)", new[] { "Someone" }, "GPLv3", new[] { "ps2" }, new string('a', 64))!;
            record.GetProperty("Instead")!.SetValue(core, "Sapphire");
            record.GetProperty("Held")!.SetValue(core, "the entry needs refresh");
            var rom0 = System.Linq.Expressions.Expression.Parameter(typeof(string));
            Seam("AwaitingQuery").SetValue(null, System.Linq.Expressions.Expression.Lambda(awaiting, System.Linq.Expressions.Expression.Constant(core, record), rom0).Compile());
            bool asked = false;
            Seam("ConfirmCoreApproval").SetValue(null, (Func<MainWindow, string, string, Task<bool>>)((_, _, _) => { asked = true; return Task.FromResult(true); }));
            var window = new MainWindow();
            window.Show();
            var ask = (Task)typeof(MainWindow).GetMethod("AskCoreApprovalAsync", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, new object[] { Path.Combine(_root, "Roms", "Jak.iso") })!;
            var clock = Stopwatch.StartNew();
            while (!ask.IsCompleted && clock.ElapsedMilliseconds < 5000) { Dispatcher.UIThread.RunJobs(); Thread.Sleep(5); }
            Assert.True(ask.IsCompletedSuccessfully);
            Assert.False(asked, "an approved default is not asked about again");
            const string said = "Sony - PlayStation 2 (PCEE2) could not be used, so this game opens on Sapphire: the entry needs refresh";
            Assert.Equal(said, window.GetControl<TextBlock>("StatusText").Text);
            Assert.Equal(said, typeof(MainWindow).GetField("_engineFallbackNotice", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window));
            window.Close();
        }, default);
    }
}
