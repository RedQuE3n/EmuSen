using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
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

        public void Dispose()
        {
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
    }
}
