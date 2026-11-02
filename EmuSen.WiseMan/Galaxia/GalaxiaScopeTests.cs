using System;
using System.IO;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Threading;
using System.Threading.Tasks;
using EmuSen.Crystal.Client;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Native;
using Xunit;

namespace EmuSen.WiseMan.Galaxia
{
    // A test's scope reaches the platform library on every thread its flow takes, and no other flow - see EmuSen_Galaxia.md §3.14.
    public class GalaxiaScopeTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenScope_" + Guid.NewGuid().ToString("N"));

        public void Dispose()
        {
            if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true);
        }

        private (string Home, string Etc) Dirs(string name) => (Path.Combine(_root, name, "home"), Path.Combine(_root, name, "etc"));

        [Fact]
        public void Inside_a_scope_the_library_resolves_its_directories_and_writes_its_config_there_and_leaving_restores_the_process_tree()
        {
            string config = ConfigStore.Directory, saves = DataStore.Saves;
            var (home, etc) = Dirs("one");
            using (GalaxiaScope.Enter(data: home, config: etc))
            {
                Assert.Equal(etc, ConfigStore.Directory);
                Assert.StartsWith(home, DataStore.Saves);
                Assert.True(GalaxiaNative.ConfigWrite(null, "scope-probe.json", "{}"));
                Assert.True(File.Exists(Path.Combine(etc, "scope-probe.json")));
                Assert.Null(ConfigStore.LegacyPathFor("scope-probe.json"));
            }
            Assert.Equal(config, ConfigStore.Directory);
            Assert.Equal(saves, DataStore.Saves);
        }

        [Fact]
        public async Task A_continuation_that_lands_on_another_thread_keeps_the_scope_in_the_library()
        {
            var (home, etc) = Dirs("hop");
            using var scope = GalaxiaScope.Enter(data: home, config: etc);
            int first = Environment.CurrentManagedThreadId, hops = 0;
            for (int i = 0; i < 200 && hops == 0; i++)
            {
                await Task.Delay(1).ConfigureAwait(false);
                if (Environment.CurrentManagedThreadId != first) hops++;
            }
            Assert.True(hops > 0, "no continuation moved to another thread");
            Assert.Equal(etc, ConfigStore.Directory);
            Assert.True(GalaxiaNative.ConfigWrite(null, "hop.json", "{}"));
            Assert.True(File.Exists(Path.Combine(etc, "hop.json")));
        }

        [Fact]
        public async Task Two_scopes_on_interleaved_threads_each_see_only_their_own_tree()
        {
            string process = ConfigStore.Directory;
            using var both = new Barrier(2);
            async Task<int> Run(string name)
            {
                await Task.Yield();
                var (home, etc) = Dirs(name);
                using var scope = GalaxiaScope.Enter(data: home, config: etc);
                int threads = 0, last = -1;
                for (int i = 0; i < 100; i++)
                {
                    both.SignalAndWait(TimeSpan.FromSeconds(10));
                    Assert.Equal(etc, ConfigStore.Directory);
                    Assert.StartsWith(home, DataStore.Saves);
                    await Task.Delay(1).ConfigureAwait(false);
                    if (Environment.CurrentManagedThreadId != last) { threads++; last = Environment.CurrentManagedThreadId; }
                }
                return threads;
            }
            int[] seen = await Task.WhenAll(Task.Run(() => Run("left")), Task.Run(() => Run("right")));
            Assert.All(seen, n => Assert.True(n > 1, "the flow stayed on one thread"));
            Assert.Equal(process, ConfigStore.Directory);
        }

        [Fact]
        public void The_library_s_own_threads_refuse_to_start_inside_a_scope()
        {
            var (home, etc) = Dirs("refused");
            using var scope = GalaxiaScope.Enter(data: home, config: etc);
            Assert.Contains("serial collection", Assert.Throws<InvalidOperationException>(() => new CrystalRuntime()).Message);
            Assert.Throws<InvalidOperationException>(() => LibraryToolJob.Start("{}"));
            Assert.Throws<InvalidOperationException>(() => ScrapeCommand.Ask("{}"));
        }

        [Fact]
        public void Nothing_outside_the_tests_enters_a_scope_or_calls_the_export()
        {
            string repo = Path.GetFullPath(Path.Combine(Here(), "..", ".."));
            string[] allowed = ["EmuSen.WiseMan/", "EmuSen.Galaxia/GalaxiaScope.cs", "EmuSen.Galaxia/Native/GalaxiaNative.cs", "Platform/galaxia/src/tree.rs", "Platform/platform/src/galaxia.rs"];
            var projects = Directory.EnumerateDirectories(repo).Where(d => Path.GetFileName(d) is var n && (n.StartsWith("EmuSen") || n == "Platform"));
            var callers = projects.SelectMany(d => new[] { "*.cs", "*.rs" }.SelectMany(p => Directory.EnumerateFiles(d, p, SearchOption.AllDirectories)))
                .Select(f => Path.GetRelativePath(repo, f).Replace('\\', '/'))
                .Where(f => !f.Contains("/obj/") && !f.Contains("/bin/") && !f.Contains("/target/") && !allowed.Any(f.StartsWith))
                .Where(f => File.ReadAllText(Path.Combine(repo, f)) is var text && (text.Contains("GalaxiaScope.Enter") || text.Contains("emusen_galaxia_thread_scope") || text.Contains("set_thread_scope")))
                .ToList();
            Assert.Empty(callers);
        }

        private static string Here([CallerFilePath] string path = "") => Path.GetDirectoryName(path)!;
    }
}
