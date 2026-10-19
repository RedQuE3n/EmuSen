using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using System.Text.Json;
using System.Text.Json.Nodes;
using EmuSen.Galaxia.Library;

namespace EmuSen.Cores.Native
{
    // A library's sidecar, <file>.core.json: its info and settings schema and its SHA-256, so a core can be listed without loading it - see EmuSen_CoreAPI.md §7.1, §19.
    public sealed record CoreSidecar(string SidecarPath, string LibraryPath, string Sha256, string Abi, ulong Capabilities, CoreInfo Info, JsonNode InfoJson, string SettingsText)
    {
        // A core not yet offered to players, which discovery skips unless asked - see EmuSen_CoreAPI.md §27.
        public bool Development { get; init; }

        public const string Suffix = ".core.json";

        public static string PathFor(string libraryPath) => libraryPath + Suffix;

        public static string HashOf(string path)
        {
            using var file = File.OpenRead(path);
            return Convert.ToHexStringLower(SHA256.HashData(file));
        }

        // Loads the library once, as the build step does, and writes its sidecar beside it.
        public static CoreSidecar Write(string libraryPath, bool development = false)
        {
            var library = CoreLibrary.Open(libraryPath);
            if (!library.Available) throw new InvalidOperationException(library.Report);
            var doc = new JsonObject
            {
                ["sidecar"] = 1,
                ["library"] = System.IO.Path.GetFileName(libraryPath),
                ["sha256"] = HashOf(libraryPath),
                ["abi"] = $"{library.AbiVersion >> 16}.{library.AbiVersion & 0xFFFF}",
                ["capabilities"] = library.Capabilities,
                ["info"] = JsonNode.Parse(library.Info.Text),
                ["settings"] = JsonNode.Parse(library.SettingsText),
            };
            if (development) doc["development"] = true;
            string path = PathFor(libraryPath);
            File.WriteAllText(path, doc.ToJsonString(new JsonSerializerOptions { WriteIndented = true }) + "\n");
            return Read(path) ?? throw new InvalidDataException($"{path} could not be read back");
        }

        // Null for a file that is not a sidecar of this version, or whose library is not beside it.
        public static CoreSidecar? Read(string sidecarPath)
        {
            try
            {
                if (JsonNode.Parse(File.ReadAllText(sidecarPath)) is not JsonObject doc || doc["sidecar"]?.GetValue<int>() != 1) return null;
                string? name = doc["library"]?.GetValue<string>();
                if (name is null || name != System.IO.Path.GetFileName(name) || doc["info"] is not JsonObject info) return null;
                string library = System.IO.Path.Combine(System.IO.Path.GetDirectoryName(sidecarPath)!, name);
                return new CoreSidecar(sidecarPath, library, doc["sha256"]?.GetValue<string>() ?? "", doc["abi"]?.GetValue<string>() ?? "",
                    doc["capabilities"]?.GetValue<ulong>() ?? 0, CoreDescriptorReader.Info(info.ToJsonString()), info.DeepClone(), doc["settings"]?.ToJsonString() ?? "[]")
                { Development = doc["development"]?.GetValue<bool>() == true };
            }
            catch (Exception e) when (e is JsonException or IOException or InvalidOperationException or FormatException or UnauthorizedAccessException)
            {
                return null;
            }
        }
    }

    // A v1 engine found by its sidecar; Open loads it only when a game is opened on it, refusing a library that is not the one the sidecar describes.
    public sealed class DiscoveredCore(CoreSidecar sidecar)
    {
        private CoreLibrary? _library;
        private string? _refusal;

        public CoreSidecar Sidecar { get; } = sidecar;

        // Found in one of the player's directories - see EmuSen_Backend.md §29.40.4.
        public bool Player { get; init; }

        // The last Open refused for want of the player's approval alone.
        public bool AwaitsApproval { get; private set; }
        public CoreInfo Info => Sidecar.Info;
        public string EngineName => Info.DisplayName;

        // Why the last Open refused, or the library's own report.
        public string Report => _refusal ?? _library?.Report ?? "not yet loaded";

        public CoreLibrary? Open()
        {
            if (_library is not null || _refusal is not null) return _library is { Available: true } ? _library : null;
            if (!File.Exists(Sidecar.LibraryPath)) { _refusal = $"{System.IO.Path.GetFileName(Sidecar.LibraryPath)} is missing beside its sidecar"; return null; }
            string hash = CoreSidecar.HashOf(Sidecar.LibraryPath);
            if (!string.Equals(hash, Sidecar.Sha256, StringComparison.OrdinalIgnoreCase))
            {
                _refusal = $"{System.IO.Path.GetFileName(Sidecar.LibraryPath)} is not the library its sidecar describes (its SHA-256 differs); it was not loaded";
                return null;
            }
            if (!CoreDiscovery.Vouches(Sidecar.LibraryPath, hash))
            {
                _refusal = $"{System.IO.Path.GetFileName(Sidecar.LibraryPath)} is in no cores manifest and has not been approved; it was not loaded";
                AwaitsApproval = true;
                return null;
            }
            var library = CoreLibrary.Open(Sidecar.LibraryPath);
            _library = library;
            if (library.Available && !JsonNode.DeepEquals(JsonNode.Parse(library.Info.Text), Sidecar.InfoJson))
                _refusal = $"{System.IO.Path.GetFileName(Sidecar.LibraryPath)}'s info differs from its sidecar's; it is refused as altered";
            return _refusal is null && library.Available ? library : null;
        }

        // How far the library is trusted, by its sidecar's hash: the manifest's, the player's approval, or neither.
        public string Standing => !Player && CoreDiscovery.InManifest(Sidecar.LibraryPath, Sidecar.Sha256) ? "manifest"
            : CoreDiscovery.Approved(Sidecar.Sha256) ? "approved" : "needs_approval";
    }

    // A library in a player's directory that no sidecar describes, listed by its hash and read only once approved - see EmuSen_Backend.md §29.40.4.
    public sealed record UnreadCore(string LibraryPath, string Sha256)
    {
        public string Report => CoreLibrary.IsOpen(LibraryPath) ? CoreLibrary.Open(LibraryPath).Report : "not yet read";
    }

    // The v1 engines in the cores directories, listed from their sidecars alone, so starting a frontend runs no core code - see EmuSen_CoreAPI.md §7.1, §19.
    public static class CoreDiscovery
    {
        public const string DevelopmentVariable = "EMUSEN_DEVELOPMENT_CORES";

        private static IReadOnlyList<DiscoveredCore>? _found;
        private static IReadOnlyList<UnreadCore> _unread = Array.Empty<UnreadCore>();
        private static IReadOnlyList<string>? _directories;
        private static IReadOnlyList<string>? _playerDirectories;
        private static bool? _includeDevelopment;
        private static Dictionary<string, IReadOnlyList<(string Sha256, string Name)>> _manifests = new();
        private static HashSet<string> _approvals = new(StringComparer.Ordinal);
        private static readonly Dictionary<string, (long Length, DateTime Time, string Sha256)> ScanHashes = new(StringComparer.Ordinal);

        // Whether cores still in development are listed: off for players, on with EMUSEN_DEVELOPMENT_CORES=1 or when a test asks.
        public static bool IncludeDevelopment => _includeDevelopment ?? Environment.GetEnvironmentVariable(DevelopmentVariable) == "1";

        // Set in place of the variable, null for the variable's answer; the next question scans again.
        public static void UseDevelopment(bool? include)
        {
            _includeDevelopment = include;
            _found = null;
        }

        // Beside the assemblies, and its cores folder.
        public static IReadOnlyList<string> Directories => _directories ?? new[] { AppContext.BaseDirectory, System.IO.Path.Combine(AppContext.BaseDirectory, "cores") };

        // Other directories in their place, null for the defaults; the next question scans again.
        public static void UseDirectories(IReadOnlyList<string>? directories)
        {
            _directories = directories;
            _found = null;
        }

        // The player's own: /Cores where the installation's are the defaults, none where a caller names them - see EmuSen_Backend.md §29.40.4.
        public static IReadOnlyList<string> PlayerDirectories => _playerDirectories ?? (_directories is null ? new[] { DataStore.Cores } : Array.Empty<string>());

        // Other player's directories in their place, null for the default; the next question scans again.
        public static void UsePlayerDirectories(IReadOnlyList<string>? directories)
        {
            _playerDirectories = directories;
            _found = null;
        }

        public static void Rescan() => _found = null;

        public static IReadOnlyList<DiscoveredCore> Found => _found ??= Scan();

        // The player's libraries that no sidecar describes, as the last scan found them.
        public static IReadOnlyList<UnreadCore> Unread { get { _ = Found; return _unread; } }

        // Said when the approvals' file is refused; the runtime points it at its log.
        public static Action<string>? Say { get; set; }

        // Rule 2: the player approved the hash.
        public static bool Approved(string sha256) { _ = Found; return _approvals.Contains(sha256.ToLowerInvariant()); }

        // Rule 1: the manifest of the library's own installation directory lists it under its name with this hash.
        public static bool InManifest(string library, string sha256)
        {
            _ = Found;
            string dir = System.IO.Path.TrimEndingDirectorySeparator(System.IO.Path.GetFullPath(System.IO.Path.GetDirectoryName(library)!));
            string name = System.IO.Path.GetFileName(library), hash = sha256.ToLowerInvariant();
            return _manifests.TryGetValue(dir, out var entries) && entries.Any(e => e.Name == name && e.Sha256 == hash);
        }

        public static bool Vouches(string library, string sha256) => Approved(sha256) || InManifest(library, sha256);

        // Whether a library the listing shows has this hash.
        public static bool ListsHash(string sha256) =>
            Found.Any(c => string.Equals(c.Sidecar.Sha256, sha256, StringComparison.OrdinalIgnoreCase)) || _unread.Any(u => string.Equals(u.Sha256, sha256, StringComparison.OrdinalIgnoreCase));

        private static string LibrarySuffix => OperatingSystem.IsWindows() ? ".dll" : OperatingSystem.IsMacOS() ? ".dylib" : ".so";

        // A player's library's hash, read once while its length and time stay those it was read at.
        private static string? ScanHash(string path)
        {
            try
            {
                var info = new FileInfo(path);
                string full = System.IO.Path.GetFullPath(path);
                lock (ScanHashes)
                    if (ScanHashes.TryGetValue(full, out var known) && known.Length == info.Length && known.Time == info.LastWriteTimeUtc) return known.Sha256;
                string hash = CoreSidecar.HashOf(path);
                lock (ScanHashes) ScanHashes[full] = (info.Length, info.LastWriteTimeUtc, hash);
                return hash;
            }
            catch (Exception e) when (e is IOException or UnauthorizedAccessException) { return null; }
        }

        private static IReadOnlyList<DiscoveredCore> Scan()
        {
            var list = new List<DiscoveredCore>();
            var unread = new List<UnreadCore>();
            var manifests = new Dictionary<string, IReadOnlyList<(string, string)>>(StringComparer.Ordinal);
            var seen = new HashSet<string>(StringComparer.Ordinal);
            foreach (var (dir, player) in Directories.Select(d => (d, false)).Concat(PlayerDirectories.Select(d => (d, true))))
            {
                if (!seen.Add(dir) || !Directory.Exists(dir)) continue;
                if (!player) manifests.TryAdd(System.IO.Path.TrimEndingDirectorySeparator(System.IO.Path.GetFullPath(dir)), CoreTrust.ManifestOf(dir));
                var files = Directory.EnumerateFiles(dir).Select(System.IO.Path.GetFileName).Order(StringComparer.Ordinal).ToArray();
                var described = new HashSet<string>(StringComparer.Ordinal);
                foreach (string name in files.Where(f => f!.EndsWith(CoreSidecar.Suffix, StringComparison.Ordinal)))
                {
                    if (CoreSidecar.Read(System.IO.Path.Combine(dir, name!)) is not { } sidecar) continue;
                    if (player && File.Exists(sidecar.LibraryPath) && !string.Equals(ScanHash(sidecar.LibraryPath), sidecar.Sha256, StringComparison.OrdinalIgnoreCase)) continue;
                    described.Add(sidecar.LibraryPath);
                    if (sidecar.Abi.StartsWith($"{CoreInterface.Major}.", StringComparison.Ordinal) && (!sidecar.Development || IncludeDevelopment) && !list.Any(c => c.Info.Id == sidecar.Info.Id))
                        list.Add(new DiscoveredCore(sidecar) { Player = player });
                }
                if (!player) continue;
                foreach (string name in files.Where(f => f!.EndsWith(LibrarySuffix, StringComparison.Ordinal) && f.Length > LibrarySuffix.Length))
                {
                    string path = System.IO.Path.Combine(dir, name!);
                    if (!described.Contains(path) && ScanHash(path) is { } hash) unread.Add(new UnreadCore(path, hash));
                }
            }
            _manifests = manifests;
            _approvals = CoreTrust.ReadApprovals(CoreTrust.ApprovalsPath, Say).ToHashSet(StringComparer.Ordinal);
            _unread = unread;
            return list;
        }

        // A system its core marks in development is offered only where development cores are asked for - see EmuSen_CoreAPI.md §27.4.
        public static bool Offered(CoreSystem system) => !system.Development || IncludeDevelopment;

        // The engines whose offered systems claim this extension.
        public static IEnumerable<DiscoveredCore> ForExtension(string extension) =>
            Found.Where(c => c.Info.Systems.Any(s => Offered(s) && s.Extensions.Contains(extension, StringComparer.OrdinalIgnoreCase)));

        // The engines that offer a system, by its id.
        public static IEnumerable<DiscoveredCore> ForSystem(string systemId) => Found.Where(c => c.Info.Systems.Any(s => s.Id == systemId && Offered(s)));

        public static DiscoveredCore? ByEngineName(string? name) => name is null ? null : Found.FirstOrDefault(c => c.EngineName == name);
    }
}
