using System;
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
    // A launch whose questions are still being asked off the interface thread when the window closes ends quietly, never with an exception on the interface thread - see EmuSen_Debugging_Tools_Reference_v5.md §3.69.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class LaunchAfterCloseTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session = HeadlessUnitTestSession.GetOrStartForAssembly(typeof(LaunchAfterCloseTests).GetTypeInfo().Assembly);
        private readonly string _root = Path.Combine(Environment.GetEnvironmentVariable("TMPDIR") is { Length: > 0 } t ? t : Path.GetTempPath(), "EmuSenLaunchClose_" + Guid.NewGuid().ToString("N"));

        private static FieldInfo Seam(string name) => typeof(MainWindow).GetField(name, BindingFlags.Static | BindingFlags.NonPublic)!;
        private readonly object _firmware = Seam("FirmwareQuery").GetValue(null)!;

        public LaunchAfterCloseTests()
        {
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            Directory.CreateDirectory(Path.Combine(_root, "Roms"));
            // A file no core opens, so the launch ends on its failure path, which shows the library again.
            File.WriteAllBytes(Path.Combine(_root, "Roms", "Broken.sfc"), [1, 2, 3]);
            new AppSettings { BigPictureInterface = { LaunchScreenDuration = EmuSen.Galaxia.Models.BigPictureInterface.LaunchDisabled }, RomDirectory = Path.Combine(_root, "Roms"), LibraryView = AppSettings.LibraryList, LogDirectory = Path.Combine(_root, "Logs"), ResumeOnLaunch = AppSettings.ResumeNever }.Save();
        }

        public void Dispose()
        {
            Seam("FirmwareQuery").SetValue(null, _firmware);
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        [Fact]
        public Task A_launch_still_asking_when_the_window_closes_throws_nothing_on_the_interface_thread() => Session.Dispatch(() =>
        {
            Exception? fault = null;
            void Caught(object? s, DispatcherUnhandledExceptionEventArgs e)
            {
                fault ??= e.Exception;
                e.Handled = true;
            }
            Dispatcher.UIThread.UnhandledException += Caught;
            try
            {
                var window = new MainWindow { Width = 1280, Height = 800 };
                window.Show();
                var clock = Stopwatch.StartNew();
                ListBox list = window.GetControl<ListBox>("LibraryList");
                while (list.ItemCount == 0 && clock.ElapsedMilliseconds < 10_000) { Dispatcher.UIThread.RunJobs(); Thread.Sleep(5); }
                Assert.True(list.ItemCount > 0, "the library lists the game");
                list.SelectedIndex = 0;
                // The firmware question is answered only once the window has closed, as a slow answer would be.
                using var release = new ManualResetEventSlim();
                Seam("FirmwareQuery").SetValue(null, (Func<string, string>)(_ => { release.Wait(TimeSpan.FromSeconds(10)); return "[]"; }));
                // The launch asks its questions off the interface thread; the window closes before they are answered.
                typeof(MainWindow).GetMethod("LaunchSelectedLibraryEntry", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);
                window.Close();
                release.Set();
                clock.Restart();
                while (clock.ElapsedMilliseconds < 3_000) { Dispatcher.UIThread.RunJobs(); Thread.Sleep(5); }
                Assert.Null(fault);
            }
            finally { Dispatcher.UIThread.UnhandledException -= Caught; }
        }, default);
    }
}
