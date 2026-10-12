using System;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Media;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Big picture's launch of the tester's PlayStation discs timed from the pad's A to the first frame, the interface thread's holds measured; opt-in by EMUSEN_DIAMOND_LAUNCHTIME_LIBRARY - see Diamond_Players.md §12.8.
    [Collection(TestCollections.ProcessGlobals)]
    public class PlayStationLaunchTimeTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(PlayStationLaunchTimeTests).GetTypeInfo().Assembly);

        private static readonly BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;

        private readonly ITestOutputHelper _out;

        public PlayStationLaunchTimeTests(ITestOutputHelper output) => _out = output;

        [DllImport("libc", SetLastError = true)]
        private static extern int link(string from, string to);

        [DllImport("libc", SetLastError = true)]
        private static extern int posix_fadvise(int fd, long offset, long len, int advice);

        private static void Evict(string path)
        {
            using var handle = File.OpenHandle(path);
            posix_fadvise((int)handle.DangerousGetHandle(), 0, 0, 4);
        }

        // The game's files linked, not copied, into the session's psx folder, so that a cold read is the copy's own device.
        private static string LinkInto(string game, string psx)
        {
            Directory.CreateDirectory(psx);
            string source = Path.GetDirectoryName(game)!;
            foreach (string f in ImageSets.AllFiles(game))
            {
                string to = Path.Combine(psx, Path.GetRelativePath(source, f));
                Directory.CreateDirectory(Path.GetDirectoryName(to)!);
                if (link(f, to) != 0) throw new IOException($"link {f}: errno {Marshal.GetLastWin32Error()}; TMPDIR must be on the library's filesystem");
            }
            return Path.Combine(psx, Path.GetRelativePath(source, game));
        }

        private static CoreEngine? Engine(MainWindow w) =>
            (typeof(MainWindow).GetField("_session", Hidden)!.GetValue(w) as EmuSen.Common.EmulatorSession)?.Core as CoreEngine;

        private string Launch(string game, string duration, bool cold)
        {
            string? linked = null;
            using var s = new ThemedSession(settings: a => a.BigPictureInterface.LaunchScreenDuration = duration, roms: r => linked = LinkInto(game, Path.Combine(r, "psx")));
            // The window's own load under the screen, which ThemedSession takes away for its stepped clock; this probe keeps the wall's time.
            typeof(MainWindow).GetProperty("EarlyLoader", Hidden)!.SetValue(s.Window, (Func<string, string?, EmuSen.Cores.ICore>)((p, engine) => EmuSen.Cores.CoreFactory.Loaded(p, headless: true, engine)));
            if (cold) foreach (string f in ImageSets.AllFiles(linked!)) Evict(f);
            for (int guard = 0; guard < 6 && s.System != "psx"; guard++) s.Pad.Right();
            if (s.System != "psx") return $"no psx system: {s.System}";
            s.Pad.A();
            var clock = Stopwatch.StartNew();
            s.Pad.A();
            double toScreen = clock.Elapsed.TotalMilliseconds;
            bool screen = typeof(MainWindow).GetProperty("LaunchScreenShown", Hidden)!.GetValue(s.Window) is not null;
            double longest = 0, loadStep = 0;
            while (LaunchScreenTests.Running(s.Window) is null && clock.Elapsed < TimeSpan.FromSeconds(120))
            {
                long step = Stopwatch.GetTimestamp();
                s.Run(16);
                double ms = Stopwatch.GetElapsedTime(step).TotalMilliseconds;
                longest = Math.Max(longest, ms);
                if (LaunchScreenTests.Running(s.Window) is not null) loadStep = ms;
                Thread.Sleep(Math.Max(0, 16 - (int)ms));
            }
            double started = clock.Elapsed.TotalMilliseconds;
            var core = Engine(s.Window);
            bool framed = core is not null && SpinWait.SpinUntil(() => core.TotalFrames >= 1, TimeSpan.FromSeconds(60));
            double firstFrame = clock.Elapsed.TotalMilliseconds;
            LaunchScreenTests.Stop(s.Window);
            return $"{(cold ? "cold" : "warm")} {duration} | A to screen {toScreen:F0} ms{(screen ? "" : " (no screen)")} | the step that loaded {loadStep:F0} ms | game started {started:F0} ms | first frame {(framed ? $"{firstFrame:F0} ms" : "none")}";
        }

        [Fact]
        public Task The_testers_discs_launch_from_big_picture_timed() => Session.Dispatch(() =>
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_DIAMOND_LAUNCHTIME_LIBRARY") is not { } library || !Directory.Exists(library))
            {
                _out.WriteLine("skipped: EMUSEN_DIAMOND_LAUNCHTIME_LIBRARY names a folder of the tester's discs, copied");
                return;
            }
            string only = Environment.GetEnvironmentVariable("EMUSEN_DIAMOND_LAUNCHTIME_ONLY") ?? "";
            foreach (string game in ImageSets.Shelved(Directory.EnumerateFiles(library, "*", SearchOption.AllDirectories).Where(f => EmuSen.Cores.CoreFactory.IsSupported(f)).Order().ToList()).Where(g => g.Contains(only, StringComparison.Ordinal)))
                foreach (bool cold in new[] { true, false })
                    foreach (string duration in new[] { BigPictureInterface.LaunchNormal, BigPictureInterface.LaunchDisabled })
                        _out.WriteLine($"{Path.GetFileName(game)} | {Launch(game, duration, cold)}");
        }, default);
    }
}
