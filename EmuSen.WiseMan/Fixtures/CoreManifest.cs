using System.IO;
using System.Linq;
using EmuSen.Cores.Native;

namespace EmuSen.WiseMan.Fixtures
{
    // A test tree's cores.manifest, as an installation carries one, so its cores load without an approval - see EmuSen_Backend.md §29.40.6.
    public static class CoreManifest
    {
        // Every library a sidecar in dir names and that is there, in sha256sum's format.
        public static void Write(string dir) =>
            File.WriteAllText(Path.Combine(dir, CoreTrust.ManifestName), string.Concat(Directory.EnumerateFiles(dir, "*" + CoreSidecar.Suffix).Order()
                .Select(CoreSidecar.Read).Where(s => s is not null && File.Exists(s.LibraryPath))
                .Select(s => $"{CoreSidecar.HashOf(s!.LibraryPath)}  {Path.GetFileName(s.LibraryPath)}\n")));
    }
}
