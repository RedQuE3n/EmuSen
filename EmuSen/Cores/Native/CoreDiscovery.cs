using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using System.Text.Json;
using System.Text.Json.Nodes;

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
        public CoreInfo Info => Sidecar.Info;
        public string EngineName => Info.DisplayName;

        // Why the last Open refused, or the library's own report.
        public string Report => _refusal ?? _library?.Report ?? "not yet loaded";

        public CoreLibrary? Open()
        {
            if (_library is not null || _refusal is not null) return _library is { Available: true } ? _library : null;
            if (!File.Exists(Sidecar.LibraryPath)) { _refusal = $"{System.IO.Path.GetFileName(Sidecar.LibraryPath)} is missing beside its sidecar"; return null; }
            if (!string.Equals(CoreSidecar.HashOf(Sidecar.LibraryPath), Sidecar.Sha256, StringComparison.OrdinalIgnoreCase))
            {
                _refusal = $"{System.IO.Path.GetFileName(Sidecar.LibraryPath)} is not the library its sidecar describes (its SHA-256 differs); it was not loaded";
                return null;
            }
            var library = CoreLibrary.Open(Sidecar.LibraryPath);
            _library = library;
            if (library.Available && !JsonNode.DeepEquals(JsonNode.Parse(library.Info.Text), Sidecar.InfoJson))
                _refusal = $"{System.IO.Path.GetFileName(Sidecar.LibraryPath)}'s info differs from its sidecar's; it is refused as altered";
            return _refusal is null && library.Available ? library : null;
        }
    }

    // The v1 engines in the cores directories, listed from their sidecars alone, so starting a frontend runs no core code - see EmuSen_CoreAPI.md §7.1, §19.
    public static class CoreDiscovery
    {
        public const string DevelopmentVariable = "EMUSEN_DEVELOPMENT_CORES";

        private static IReadOnlyList<DiscoveredCore>? _found;
        private static IReadOnlyList<string>? _directories;
        private static bool? _includeDevelopment;

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

        public static void Rescan() => _found = null;

        public static IReadOnlyList<DiscoveredCore> Found => _found ??= Scan();

        private static IReadOnlyList<DiscoveredCore> Scan()
        {
            var list = new List<DiscoveredCore>();
            foreach (string dir in Directories.Distinct(StringComparer.Ordinal))
            {
                if (!Directory.Exists(dir)) continue;
                foreach (string file in Directory.EnumerateFiles(dir, "*" + CoreSidecar.Suffix).Order(StringComparer.Ordinal))
                    if (CoreSidecar.Read(file) is { } sidecar && sidecar.Abi.StartsWith($"{CoreInterface.Major}.", StringComparison.Ordinal)
                        && (!sidecar.Development || IncludeDevelopment) && !list.Any(c => c.Info.Id == sidecar.Info.Id))
                        list.Add(new DiscoveredCore(sidecar));
            }
            return list;
        }

        // The engines whose systems claim this extension.
        public static IEnumerable<DiscoveredCore> ForExtension(string extension) => Found.Where(c => c.Info.Claims(extension));

        // The engines that run a system, by its id.
        public static IEnumerable<DiscoveredCore> ForSystem(string systemId) => Found.Where(c => c.Info.Systems.Any(s => s.Id == systemId));

        public static DiscoveredCore? ByEngineName(string? name) => name is null ? null : Found.FirstOrDefault(c => c.EngineName == name);
    }
}
