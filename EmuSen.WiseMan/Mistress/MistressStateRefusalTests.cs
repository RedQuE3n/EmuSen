using System;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Threading;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // Mistress's word for a state of the other console is the core's: it once compared the record's version with the running core's and said "a newer build" (Moon_PAL.md §7); on the frontend API it shows the API's refusal - see EmuSen_Backend.md §14.12.
    [Collection(TestCollections.ProcessGlobals)]
    public class MistressStateRefusalTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(MistressStateRefusalTests).GetTypeInfo().Assembly);

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenMistressStateRefusal", Guid.NewGuid().ToString("N"));
        private readonly string _pal, _ntsc;

        public MistressStateRefusalTests()
        {
            string roms = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(roms);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            byte[] ntsc = SyntheticNesRom.Build(), pal = (byte[])ntsc.Clone();
            (pal[7], pal[12]) = ((byte)((pal[7] & 0xF3) | 0x08), 1);
            _pal = Path.Combine(roms, "Europe", "Game.nes");
            _ntsc = Path.Combine(roms, "America", "Game.nes");
            Directory.CreateDirectory(Path.GetDirectoryName(_pal)!);
            Directory.CreateDirectory(Path.GetDirectoryName(_ntsc)!);
            File.WriteAllBytes(_pal, pal);
            File.WriteAllBytes(_ntsc, ntsc);
            new AppSettings { RomDirectory = roms, ResumeOnLaunch = AppSettings.ResumeNever, StateDirectory = Path.Combine(_root, "States") }.Save();
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private static void WaitFor(Func<bool> condition, Func<string?>? said = null, int ms = 20_000)
        {
            var clock = Stopwatch.StartNew();
            while (!condition())
            {
                Dispatcher.UIThread.RunJobs();
                if (clock.ElapsedMilliseconds > ms) Assert.Fail($"timed out waiting for the window: {said?.Invoke()}");
                Thread.Sleep(2);
            }
        }

        private static void Pump(int ms)
        {
            var clock = Stopwatch.StartNew();
            while (clock.ElapsedMilliseconds < ms)
            {
                Dispatcher.UIThread.RunJobs();
                Thread.Sleep(2);
            }
        }

        private MainWindow Started(string rom)
        {
            var window = new MainWindow();
            window.Show();
            typeof(MainWindow).GetMethod("LoadRom", BindingFlags.Instance | BindingFlags.NonPublic, null, new[] { typeof(string), typeof(string) }, null)!.Invoke(window, new object[] { rom, "Game.nes" });
            TextBlock status = window.FindControl<TextBlock>("StatusText")!;
            WaitFor(() => status.Text?.StartsWith("Running", StringComparison.Ordinal) == true, () => status.Text);
            Pump(300);
            return window;
        }

        private static void Invoke(MainWindow window, string method) =>
            typeof(MainWindow).GetMethod(method, BindingFlags.Instance | BindingFlags.NonPublic, null, Type.EmptyTypes, null)!.Invoke(window, null);

        // A PAL copy's state loaded into the NTSC copy: the core refuses, and the status line says what it said - see EmuSen_Backend.md §14.13.
        [Fact]
        public Task A_pal_state_in_an_ntsc_game_is_refused_in_the_core_s_words() => Session.Dispatch(() =>
        {
            Assert.True(MoonRtCore.Available, MoonNative.Report);
            MainWindow saving = Started(_pal);
            TextBlock status = saving.FindControl<TextBlock>("StatusText")!;
            Invoke(saving, "SaveState");
            WaitFor(() => status.Text?.StartsWith("State saved", StringComparison.Ordinal) == true, () => status.Text);
            saving.Close();
            Pump(100);

            MainWindow loading = Started(_ntsc);
            status = loading.FindControl<TextBlock>("StatusText")!;
            Invoke(loading, "LoadState");
            // The two dumps differ, so the window first asks about another copy of the game; going on is what reaches the core.
            WaitFor(() => loading.OwnedWindows.Count > 0 || status.Text?.Contains("Load State", StringComparison.Ordinal) == true, () => status.Text);
            if (loading.OwnedWindows.FirstOrDefault() is { } ask)
                Avalonia.VisualTree.VisualExtensions.GetVisualDescendants(ask).OfType<Button>().First(b => b.Content as string == "Load Anyway").RaiseEvent(new Avalonia.Interactivity.RoutedEventArgs(Button.ClickEvent));
            WaitFor(() => status.Text?.Contains("Load State", StringComparison.Ordinal) == true, () => status.Text);
            string said = status.Text!;
            loading.Close();
            Assert.DoesNotContain("newer build", said);
            Assert.Contains("a state saved on another console", said);
        }, default);
    }
}
