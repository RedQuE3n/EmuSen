using System;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text.Json;
using EmuSen.Crystal.Client;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;

namespace EmuSen.WiseMan.Fixtures
{
    // An installed libretro entry laid out as Cores lays one out: the project's proxy and libretro host copied in, the test core beside them, approved through the runtime by its hash - see EmuSen_LibretroHost.md §10d.
    public sealed class LibretroEntryFixture : IDisposable
    {
        [DllImport("libc", EntryPoint = "setenv")] private static extern int SetEnv(string name, string value, int overwrite);
        [DllImport("libc", EntryPoint = "unsetenv")] private static extern int UnsetEnv(string name);

        public const string Folder = "libretro", ProxyLibrary = "emusen_libretro_proxy", HostLibrary = "emusen_libretro_host";

        private static string Extension => OperatingSystem.IsWindows() ? ".dll" : OperatingSystem.IsMacOS() ? ".dylib" : ".so";

        public static string ProxyPath(string folder) => Path.Combine(folder, "emusen-core-broker" + Extension);

        public static string AdapterPath(string folder) => Path.Combine(folder, "emusen-libretro-host" + Extension);

        // The runner the proxy starts, named in the process's own environment, which .NET's SetEnvironmentVariable does not reach.
        public static void Environment(string name, string? value) => _ = value is null ? UnsetEnv(name) : SetEnv(name, value, 1);

        public string Root { get; } = Path.Combine(Path.GetTempPath(), "EmuSenLibretro_" + Guid.NewGuid().ToString("N"));
        public string Cores => Path.Combine(Root, "Cores");
        public string Stem { get; }
        public string Entry => Path.Combine(Cores, Folder, Stem);
        public string CoreFile => Path.Combine(Entry, $"{Stem}_libretro.so");
        public string Sha256 { get; }

        // The engine the runtime lists the entry as, its sidecar's display name.
        public (string EngineName, string Library) Found { get; }

        public LibretroEntryFixture() : this("") { }

        // An entry whose information file names the libretro system, so that its machine reports the systems table's console; under another stem, that core's profile applies.
        internal LibretroEntryFixture(string systemId, string stem = "testcore", string extensions = "", string corename = "TestCore")
        {
            Stem = stem;
            DataStore.OverrideDirectory = Path.Combine(Root, "Home");
            ConfigStore.OverrideDirectory = Path.Combine(Root, "Config");
            Lay(Entry, systemId, stem, extensions, corename);
            Environment("EMUSEN_CORE_RUNNER", Path.Combine(AppContext.BaseDirectory, "emusen-core-runner"));
            Sha256 = CoreSidecars.HashOf(CoreFile);
            using var runtime = new CrystalRuntime(new CrystalRuntimeOptions { Cores = new[] { AppContext.BaseDirectory }, PlayerCores = new[] { Cores } });
            runtime.CoreApprove(Sha256);
            runtime.Rescan();
            using JsonDocument listing = JsonDocument.Parse(runtime.Cores());
            string proxy = ProxyPath(Entry);
            JsonElement row = listing.RootElement.EnumerateArray().Single(r => r.GetProperty("library").GetString() == proxy);
            Found = (row.GetProperty("engine").GetString() ?? throw new InvalidOperationException($"{proxy}: {row.GetProperty("report").GetString()}"), proxy);
        }

        // An entry's files: the two copies from the installation, the test core named on its stem, its information file, with the extensions a real core of that stem lists when given.
        public static void Lay(string folder, string systemId = "", string stem = "testcore", string extensions = "", string corename = "TestCore")
        {
            Directory.CreateDirectory(folder);
            File.Copy(CoreSidecars.LibraryPath(ProxyLibrary), ProxyPath(folder), overwrite: true);
            File.Copy(CoreSidecars.LibraryPath(HostLibrary), AdapterPath(folder), overwrite: true);
            File.Copy(CoreSidecars.LibraryPath("emusen_libretro_testcore"), Path.Combine(folder, $"{stem}_libretro.so"), overwrite: true);
            File.WriteAllText(Path.Combine(folder, $"{stem}_libretro.info"), $"display_name = \"EmuSen - Test Core\"\ncorename = \"{corename}\"\nlicense = \"GPLv3\"\nauthors = \"EmuSen\"\nsystemid = \"{systemId}\"\ndatabase = \"\"\n" + (extensions.Length > 0 ? $"supported_extensions = \"{extensions}\"\n" : ""));
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(Root, recursive: true); } catch (IOException) { } catch (UnauthorizedAccessException) { }
        }
    }
}
