using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;

namespace EmuSen.Cores.Native
{
    // An installed libretro core's folder as the host checks it before any runner starts: the project's two copies the installation's, the libretro core's hash approved and the file still that build - see EmuSen_LibretroHost.md §1.1, §1.3, §10d.
    public static class LibretroEntries
    {
        // emusen-core-entry's LIBRETRO, BROKER and ADAPTER: the folder under /Cores and the names of the two copies in an entry.
        public const string Folder = "libretro", Broker = "emusen-core-broker", Adapter = "emusen-libretro-host";

        // The libraries those copies are made from, as RustCores.props builds them into the installation and its cores.manifest.
        public const string ProxyLibrary = "emusen_libretro_proxy", HostLibrary = "emusen_libretro_host";

        private static string Extension => OperatingSystem.IsWindows() ? ".dll" : OperatingSystem.IsMacOS() ? ".dylib" : ".so";

        private static string Built(string library) => (OperatingSystem.IsWindows() ? "" : "lib") + library + Extension;

        public static string ProxyPath(string folder) => Path.Combine(folder, Broker + Extension);
        public static string AdapterPath(string folder) => Path.Combine(folder, Adapter + Extension);

        // The entries under a player's cores directory, by name; the dot folders (data, staging, the catalogue) are not entries.
        public static IEnumerable<string> Folders(string playerDirectory)
        {
            string root = Path.Combine(playerDirectory, Folder);
            if (!Directory.Exists(root)) return Array.Empty<string>();
            return Directory.EnumerateDirectories(root).Where(d => !Path.GetFileName(d).StartsWith('.') && new DirectoryInfo(d).LinkTarget is null).Order(StringComparer.Ordinal).ToArray();
        }

        // The one <stem>_libretro library in the folder, as the proxy finds it; null where there is none or more than one.
        public static string? CorePath(string folder)
        {
            var found = Directory.EnumerateFiles(folder, "*_libretro" + Extension).ToArray();
            return found.Length == 1 ? found[0] : null;
        }

        // Rule 1 for an entry: each copy is a library this installation's manifest lists under the name it is built as.
        private static string? Stale(string folder)
        {
            foreach (var (copy, library) in new[] { (ProxyPath(folder), ProxyLibrary), (AdapterPath(folder), HostLibrary) })
                if (!File.Exists(copy) || CoreDiscovery.ScanHash(copy) is not { } hash || !CoreDiscovery.InstallationLists(Built(library), hash))
                    return $"{Path.GetFileName(copy)} in {folder} is not this installation's {Built(library)}; the entry needs refresh (cores verify), and nothing of it was started";
            return null;
        }

        // Why nothing of the entry may start, or null: its copies stale, its libretro core unnamed, unapproved, or not the build approved; awaits where an approval would settle it.
        public static string? Refusal(string folder, CoreInfo info, out bool awaits)
        {
            awaits = false;
            if (Stale(folder) is { } stale) return stale;
            if (info.Libretro is not { } core) return $"the sidecar in {folder} names no libretro core; nothing of it was started";
            if (CorePath(folder) is not { } path || Path.GetFileName(path) != core.File) return $"{folder} does not hold {core.File} alone; nothing of it was started";
            awaits = true;
            if (!CoreDiscovery.Approved(core.Sha256)) return $"the libretro core {core.File} (SHA-256 {core.Sha256}) has not been approved; nothing of it was started";
            if (!string.Equals(CoreDiscovery.ScanHash(path), core.Sha256, StringComparison.Ordinal)) return $"{core.File} is not the build that was approved (its SHA-256 differs); nothing of it was started";
            awaits = false;
            return null;
        }

        // The entry's standing for a listing: needs_refresh, approved or needs_approval.
        public static string Standing(string folder, CoreInfo info) =>
            Stale(folder) is not null ? "needs_refresh" : Refusal(folder, info, out _) is null ? "approved" : "needs_approval";
    }
}
