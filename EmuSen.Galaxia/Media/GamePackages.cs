using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;
using System.Text.Json.Nodes;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Galaxia.Native;

namespace EmuSen.Galaxia.Media
{
    // The three folders a package's paths are relative to: Packages, AppSettings.RomDirectory (null for none) and Compressed.
    public sealed record PackagePlaces(string Packages, string? Library, string Compressed)
    {
        // The tree's folders and the library the settings name, checked as a folder setting is.
        public static PackagePlaces Current() => new(DataStore.Packages, SettingFolders.Checked(AppSettings.Load().RomDirectory).Path, DataStore.Compressed);
    }

    // A package the index holds: its folder, system, game and every file it claims, each a full path.
    public sealed record ClaimingPackage(string Folder, string System, string Game, IReadOnlyList<string> Claims);

    // Every package folder, refused ones included, and the packages whose manifests read.
    public sealed record PackageIndex(IReadOnlyList<string> Folders, IReadOnlyList<ClaimingPackage> Packages)
    {
        public ClaimingPackage? Claimant(string full) => Packages.FirstOrDefault(p => p.Claims.Contains(full, StringComparer.Ordinal));

        public bool InsidePackage(string full) => Folders.Any(f => GamePackages.Within(full, f));
    }

    // What a path opens as: the game file, the package whose stores it uses (null for none), and whether the manifest was rewritten for a moved game.
    public sealed record PackageResolution(string Game, string? Package, bool Moved);

    // The claim index and "one game, one save": the twin of the library's claims.rs - see EmuSen_GamePackages.md §3.4–§3.6 and §14, EmuSen_Galaxia.md §3.11.
    public static class GamePackages
    {
        public const int MaxDepth = 32;

        public static PackageIndex Index(PackagePlaces places) =>
            IndexFromJson(GalaxiaNative.PackageIndex(places.Packages, places.Library, places.Compressed));

        // IOException with words for a package that refuses the path: a different dump, or a game gone and not found.
        public static PackageResolution Resolve(string path, PackagePlaces places) =>
            ResolutionFromJson(GalaxiaNative.PackageResolve(path, places.Packages, places.Library, places.Compressed));

        // The package a game file opens with, or null; a refusal is no package here, and is said where the game is opened.
        public static string? RootFor(string? path, PackagePlaces? places = null) => string.IsNullOrEmpty(path) ? null : OpenedAs(path, places).Package;

        // The game file a path opens and the package whose stores it uses; the path itself and none where it is refused, the twin of claims.rs' opened_as.
        public static (string Game, string? Package) OpenedAs(string path, PackagePlaces? places = null)
        {
            if (string.IsNullOrEmpty(path)) return (path, null);
            try { return Resolve(path, places ?? PackagePlaces.Current()) is var r ? (r.Game, r.Package) : default; }
            catch (IOException) { return (path, null); }
        }

        // A package's zip beside its folder, the twin of package_zip.rs' zip_beside - see EmuSen_GamePackages.md §19.
        public static string ZipBeside(string folder) => Path.TrimEndingDirectorySeparator(folder) + ".zip";

        // The package a zip holds, its top folder and manifest, read through System.IO.Compression; null for a zip that holds none or does not read.
        public static (string Top, JsonObject Doc)? ZipManifest(string zip)
        {
            try
            {
                using var archive = System.IO.Compression.ZipFile.OpenRead(zip);
                var tops = archive.Entries.Select(e => e.FullName).Where(n => n.EndsWith("/" + GamePackage.FileName, StringComparison.Ordinal))
                    .Select(n => n[..^(GamePackage.FileName.Length + 1)]).Where(t => t.Length > 0 && !t.Contains('/')).ToArray();
                if (tops.Length != 1) return null;
                using var reader = new StreamReader(archive.GetEntry($"{tops[0]}/{GamePackage.FileName}")!.Open());
                return (tops[0], GamePackage.Parse(reader.ReadToEnd()).Document);
            }
            catch (Exception e) when (e is IOException or InvalidDataException or UnauthorizedAccessException or FormatException) { return null; }
        }

        // Whether full is folder or lies under it.
        public static bool Within(string full, string folder)
        {
            string f = Path.TrimEndingDirectorySeparator(folder);
            return full == f || (full.StartsWith(f, StringComparison.Ordinal) && full.Length > f.Length && full[f.Length] == Path.DirectorySeparatorChar);
        }

        private static PackageIndex IndexFromJson(string json)
        {
            using JsonDocument doc = JsonDocument.Parse(json);
            JsonElement root = doc.RootElement;
            var folders = root.GetProperty("folders").EnumerateArray().Select(f => f.GetString()!).ToArray();
            var packages = root.GetProperty("packages").EnumerateArray().Select(p => new ClaimingPackage(p.GetProperty("folder").GetString()!, p.GetProperty("system").GetString()!,
                p.GetProperty("game").GetString()!, p.GetProperty("claims").EnumerateArray().Select(c => c.GetString()!).ToArray())).ToArray();
            return new PackageIndex(folders, packages);
        }

        private static PackageResolution ResolutionFromJson(string json)
        {
            using JsonDocument doc = JsonDocument.Parse(json);
            JsonElement r = doc.RootElement;
            return new PackageResolution(r.GetProperty("game").GetString()!, r.GetProperty("package").GetString(), r.GetProperty("moved").GetBoolean());
        }
    }
}
