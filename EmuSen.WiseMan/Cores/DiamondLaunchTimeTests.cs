using System.Diagnostics;
using System.Runtime.InteropServices;
using EmuSen.Common.Firmware;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Media;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // A PlayStation launch's steps timed one by one on the tester's discs, first, cold and warm; opt-in by EMUSEN_DIAMOND_LAUNCHTIME_LIBRARY - see Diamond_Players.md §12.8.
    [Collection(TestCollections.ProcessGlobals)]
    public class DiamondLaunchTimeTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenDiamondLaunchTime_" + Guid.NewGuid().ToString("N"));

        public DiamondLaunchTimeTests(ITestOutputHelper output)
        {
            _output = output;
            DataStore.OverrideDirectory = Environment.GetEnvironmentVariable("EMUSEN_DIAMOND_LAUNCHTIME_HOME") is { Length: > 0 } home ? home : Path.Combine(_root, "Home");
            FirmwareLibrary.Directory = Path.Combine(_root, "NoFirmware");
            CoreDiscovery.UseDevelopment(false);
        }

        public void Dispose()
        {
            CoreDiscovery.UseDevelopment(null);
            FirmwareLibrary.ResetDirectory();
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        [DllImport("libc", SetLastError = true)]
        private static extern int posix_fadvise(int fd, long offset, long len, int advice);

        // Every page of a file dropped from the page cache, so that its next read is the device's.
        private static void Evict(string path)
        {
            using var handle = File.OpenHandle(path);
            posix_fadvise((int)handle.DangerousGetHandle(), 0, 0, 4);
        }

        private static IEnumerable<string> FilesOf(string game)
        {
            var files = ImageSets.AllFiles(game).ToList();
            if (ArchiveImages.IsArchive(game) && Directory.Exists(ArchiveImages.FolderFor(game))) files.AddRange(Directory.EnumerateFiles(ArchiveImages.FolderFor(game), "*", SearchOption.AllDirectories));
            return files;
        }

        private sealed class Steps(ITestOutputHelper output, string game, string condition)
        {
            private readonly Stopwatch _all = Stopwatch.StartNew();
            private readonly List<string> _parts = new();
            public T Time<T>(string step, Func<T> f)
            {
                long start = Stopwatch.GetTimestamp();
                T result = f();
                _parts.Add($"{step} {Stopwatch.GetElapsedTime(start).TotalMilliseconds:F0}");
                return result;
            }

            public void Done() => output.WriteLine($"{game} | {condition} | {string.Join(" | ", _parts)} | total {_all.Elapsed.TotalMilliseconds:F0} ms");
        }

        // The steps MissingFirmwareFor and CoreEngine.LoadRom take, one by one through the same public calls, then the first frame; the real Load after them as a check.
        private void Launch(string game, string condition, bool secondCreate)
        {
            var t = new Steps(_output, Path.GetFileName(game), condition);
            string console = t.Time("route", () => CoreCatalog.ConsoleForRom(game) ?? "Unknown");
            string? engine = CoreCatalog.EngineChosen(console, null);
            var probe = t.Time("probe-create", () => (CoreEngine)CoreFactory.ForFirmwareProbe(game, engine));
            t.Time("firmware-question", () => probe.RequiredFirmwareFor(game));
            var engineCore = t.Time("load-create-engine", () => (CoreEngine)CoreFactory.Create(game, true, engine));
            var discs = t.Time("load-discs", () => CoreEngine.DiscsOf(game));
            byte[] image = t.Time("load-read", () => File.ReadAllBytes(discs[0]));
            var media = t.Time("load-companions", () => ImageSets.FilesFor(discs[0]).Select(f => (f.Which, File.ReadAllBytes(f.Path))).ToList());
            var firmware = t.Time("load-firmware_for", () => engineCore.Library.FirmwareFor(image));
            string settings = string.Join("\n", engineCore.Library.Settings.Select(s => $"{s.Key}={s.Default}"));
            var files = new List<(uint, byte[])>(media);
            var machine = t.Time("create", () => new CoreMachine(engineCore.Library, image, settings, files));
            if (secondCreate)
            {
                machine.Dispose();
                files.Add((0, new byte[128 * 1024]));
                machine = t.Time("create-again", () => new CoreMachine(engineCore.Library, image, settings, files));
            }
            t.Time("first-frame", () => { machine.Advance(); return 0; });
            machine.Dispose();
            probe.Dispose();
            engineCore.Dispose();
            image = Array.Empty<byte>();
            GC.Collect();
            var whole = Stopwatch.StartNew();
            using var loaded = (CoreEngine)CoreFactory.Load(game, engine: engine).Core;
            double load = whole.Elapsed.TotalMilliseconds;
            loaded.RunFrame();
            t.Time("check-Load+frame", () => (int)load);
            _output.WriteLine($"{Path.GetFileName(game)} | {condition} | wants {string.Join(",", firmware.Select(w => w.Name))} | CoreFactory.Load {load:F0} ms, with its first frame {whole.Elapsed.TotalMilliseconds:F0} ms");
            t.Done();
            GC.Collect();
        }

        // Each game of the library first (an archive not yet unpacked), cold (every file out of the page cache) and warm.
        [Fact]
        public void The_testers_discs_launch_step_by_step()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_DIAMOND_LAUNCHTIME_LIBRARY") is not { } library || !Directory.Exists(library))
            {
                _output.WriteLine("skipped: EMUSEN_DIAMOND_LAUNCHTIME_LIBRARY names a folder of the tester's discs, copied");
                return;
            }
            foreach (var entry in ImageSets.Shelved(Directory.EnumerateFiles(library, "*", SearchOption.AllDirectories).Where(f => CoreFactory.IsSupported(f)).Order().ToList()))
            {
                if (ArchiveImages.IsArchive(entry) && Directory.Exists(ArchiveImages.FolderFor(entry))) Directory.Delete(ArchiveImages.FolderFor(entry), recursive: true);
                foreach (string f in FilesOf(entry)) Evict(f);
                Launch(entry, "first", secondCreate: false);
                foreach (string f in FilesOf(entry)) Evict(f);
                Launch(entry, "cold", secondCreate: true);
                Launch(entry, "warm", secondCreate: true);
            }
        }
    }
}
