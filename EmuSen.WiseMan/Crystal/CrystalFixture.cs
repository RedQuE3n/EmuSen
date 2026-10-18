using System;
using System.IO;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Client;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Cores;

namespace EmuSen.WiseMan.Crystal
{
    // A runtime over the v1 test core alone, installed with its sidecar in a folder of its own, its saves and config under a temporary tree.
    public sealed class CrystalFixture : IDisposable
    {
        public readonly string Root = Path.Combine(Path.GetTempPath(), "EmuSenCrystal_" + Guid.NewGuid().ToString("N"));

        public CrystalFixture()
        {
            Directory.CreateDirectory(Root);
            DataStore.OverrideDirectory = Path.Combine(Root, "Home");
            ConfigStore.OverrideDirectory = Path.Combine(Root, "Config");
            string cores = Path.Combine(Root, "cores");
            Directory.CreateDirectory(cores);
            string source = CoreAdapterTests.LibraryPath("v1_test_core");
            string copy = Path.Combine(cores, Path.GetFileName(source));
            File.Copy(source, copy);
            CoreSidecar.Write(copy);
            CoreDiscovery.UseDirectories(new[] { cores });
            Runtime = new CrystalRuntime();
        }

        public CrystalRuntime Runtime { get; }

        // A test core image of the payload, written to the tree under the name given.
        public string Game(string name = "game.tst", params byte[] payload)
        {
            string path = Path.Combine(Root, name);
            File.WriteAllBytes(path, CoreAdapterTests.Image(payload.Length == 0 ? new byte[] { 1, 2, 3, 4 } : payload));
            return path;
        }

        public CrystalSession Open(string? path = null, bool singleRows = false) =>
            Runtime.Open(new CrystalOpen { Path = path ?? Game(), NoBattery = true, SingleRows = singleRows });

        public void Dispose()
        {
            Runtime.Dispose();
            CoreDiscovery.UseDirectories(null);
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(Root, recursive: true); } catch (Exception e) when (e is IOException or UnauthorizedAccessException) { }
        }
    }
}
