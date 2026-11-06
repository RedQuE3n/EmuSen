using System.IO;
using System.Linq;

namespace EmuSen.WiseMan.Fixtures
{
    // A test tree's cores.manifest, as an installation carries one, so its cores load without an approval - see EmuSen_Backend.md §29.40.6.
    public static class CoreManifest
    {
        // Every library a sidecar in dir names and that is there, in sha256sum's format.
        public static void Write(string dir) =>
            File.WriteAllText(Path.Combine(dir, CoreSidecars.ManifestName), string.Concat(Directory.EnumerateFiles(dir, "*" + CoreSidecars.Suffix).Order()
                .Select(CoreSidecars.LibraryOf).Where(l => l is not null && File.Exists(l))
                .Select(library => $"{CoreSidecars.HashOf(library!)}  {Path.GetFileName(library)}\n")));
    }
}
