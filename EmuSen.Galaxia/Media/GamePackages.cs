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

        private static bool Native(params string?[] texts) => GalaxiaNative.Active && texts.All(t => GalaxiaNative.Crosses(t));

        public static PackageIndex Index(PackagePlaces places) =>
            Native(places.Packages, places.Library, places.Compressed) ? IndexFromJson(GalaxiaNative.PackageIndex(places.Packages, places.Library, places.Compressed)) : Managed.Index(places);

        // IOException with words for a package that refuses the path: a different dump, or a game gone and not found.
        public static PackageResolution Resolve(string path, PackagePlaces places) =>
            Native(path, places.Packages, places.Library, places.Compressed) ? ResolutionFromJson(GalaxiaNative.PackageResolve(path, places.Packages, places.Library, places.Compressed)) : Managed.Resolve(places, path);

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
                return (tops[0], GamePackage.Managed.Parse(reader.ReadToEnd()));
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

        // The C# rule, held to the library's by the parity class - see EmuSen_RustPlatform.md §3.9.
        internal static class Managed
        {
            private static readonly EnumerationOptions Recursive = new() { RecurseSubdirectories = true, IgnoreInaccessible = true, AttributesToSkip = FileAttributes.Hidden };

            public static string Under(string root, string rel) =>
                rel.Split(new[] { '/', '\\' }, StringSplitOptions.RemoveEmptyEntries).Aggregate(root, Path.Combine);

            private static string? Text(JsonNode? n) => n is JsonValue v && v.TryGetValue(out string? s) ? s : null;

            private static long? Number(JsonNode? n) => n is JsonValue v && v.GetValueKind() == JsonValueKind.Number && ulong.TryParse(v.ToJsonString(), NumberStyles.None, CultureInfo.InvariantCulture, out ulong u) ? (long)u : null;

            public static string? Place(PackagePlaces roots, string folder, JsonNode? entry)
            {
                if (entry is not JsonObject e || Text(e["path"]) is not { } rel || Text(e["where"]) is not { } where) return null;
                string? at = where switch
                {
                    "library" => roots.Library is { } l ? Under(l, rel) : null,
                    "compressed" => Under(roots.Compressed, rel),
                    "package" => Under(folder, rel),
                    "absolute" => rel,
                    _ => null,
                };
                return at is null ? null : Path.GetFullPath(at);
            }

            public static List<string> Folders(string packages)
            {
                var found = new List<string>();
                Walk(packages, 0, found);
                return found;
            }

            private static void Walk(string dir, int depth, List<string> found)
            {
                string[] names;
                try { names = Directory.EnumerateDirectories(dir).Select(d => Path.GetFileName(d)).Where(n => !n.StartsWith('.')).ToArray(); }
                catch (Exception e) when (e is IOException or UnauthorizedAccessException) { return; }
                Array.Sort(names, StringComparer.Ordinal);
                foreach (string name in names)
                {
                    string sub = Path.Join(dir, name);
                    if (File.Exists(Path.Join(sub, GamePackage.FileName))) found.Add(sub);
                    else if (depth + 1 < MaxDepth) Walk(sub, depth + 1, found);
                }
            }

            private static IEnumerable<JsonNode?> Entries(JsonNode? o, string name) => o is JsonObject j && j[name] is JsonArray a ? a : Enumerable.Empty<JsonNode?>();

            // Whether a manifest's game is a CHD the library tool wrote beside its original, in the library - see EmuSen_GamePackages.md §20.
            public static bool BesideChd(JsonObject doc) => Text(doc["container"]?["type"]) == "chd" && Text(doc["game"]?["where"]) == "library";

            private static bool SameSeen(JsonObject entry, string at)
            {
                JsonObject seen = entry["seen"] as JsonObject ?? entry;
                return SeenOf(at) is { } now && Number(seen["size"]) == now.Size && Text(seen["mtime"]) == now.Mtime;
            }

            // The tool's record verifies a CHD beside its original: three checks passed, its SHA-1, the CHD and every original at the size and time recorded - see EmuSen_GamePackages.md §20.3.
            public static bool BesideVerified(PackagePlaces roots, string folder, JsonObject doc, string game)
            {
                if (doc["container"] is not JsonObject container) return false;
                bool passed = new[] { "chdman", "tracks", "toc" }.All(k => Text(container["checks"]?[k]) == "passed");
                bool sha1 = !string.IsNullOrEmpty(Text(container["sha1"]));
                bool chd = doc["game"] is JsonObject g && g["seen"] is JsonObject && SameSeen(g, game);
                JsonNode?[] sources = Entries(container, "sources").ToArray();
                bool originals = sources.Length > 0 && sources.All(s => s is JsonObject o && Place(roots, folder, o) is { } at && SameSeen(o, at));
                return passed && sha1 && chd && originals;
            }

            public static ClaimingPackage? PackageOf(PackagePlaces roots, string folder, JsonObject doc)
            {
                if (Place(roots, folder, doc["game"]) is not { } game || Text(doc["system"]) is not { } system) return null;
                var claims = new List<string> { game };
                // A CHD beside its original claims the original only while the tool's record verifies it - see EmuSen_GamePackages.md §20.3.
                bool verified = !BesideChd(doc) || BesideVerified(roots, folder, doc, game);
                foreach (JsonNode? entry in Entries(doc, "companions").Concat(verified ? Entries(doc["container"], "sources") : Enumerable.Empty<JsonNode?>()).Concat(Entries(doc, "sources")))
                    if (Place(roots, folder, entry) is { } at && !claims.Contains(at)) claims.Add(at);
                return new ClaimingPackage(folder, system, game, claims);
            }

            private static JsonObject? Read(string manifest)
            {
                try { return GamePackage.Managed.Parse(File.ReadAllText(manifest)); }
                catch (Exception e) when (e is IOException or UnauthorizedAccessException or FormatException) { return null; }
            }

            public static PackageIndex Index(PackagePlaces roots)
            {
                List<string> folders = Folders(roots.Packages);
                var packages = folders.Select(f => Read(Path.Join(f, GamePackage.FileName)) is { } doc ? PackageOf(roots, f, doc) : null).OfType<ClaimingPackage>().ToArray();
                return new PackageIndex(folders, packages);
            }

            public static (long Size, string Mtime)? SeenOf(string at)
            {
                var info = new FileInfo(at);
                return info.Exists ? (info.Length, MtimeText(info.LastWriteTimeUtc)) : null;
            }

            public static string MtimeText(DateTime utc) =>
                new DateTime(utc.Ticks - utc.Ticks % TimeSpan.TicksPerSecond, DateTimeKind.Utc).ToString("yyyy-MM-dd'T'HH:mm:ss'Z'", CultureInfo.InvariantCulture);

            public static JsonObject? Recorded(JsonObject doc)
            {
                if (doc["container"] is JsonObject container)
                    return Text(container["type"]) == "zip" && new[] { "sha1", "md5", "crc32" }.Any(k => Text(container[k]) is not null) ? container : null;
                if (doc["dump"] is not JsonObject dump) return null;
                foreach (JsonNode? h in new[] { dump["file"], dump["sheet"] })
                    if (h is JsonObject o && new[] { "sha1", "md5", "crc32" }.Any(k => Text(o[k]) is not null)) return o;
                return null;
            }

            public static bool SameDump(JsonObject recorded, RomHashes taken)
            {
                string? Want(string k) => Text(recorded[k])?.ToLowerInvariant();
                if (Want("sha1") is { } s) return s == taken.Sha1;
                if (Want("md5") is { } m) return m == taken.Md5;
                return Want("crc32") is { } c && c == taken.Crc32;
            }

            public static string Named(JsonObject recorded)
            {
                foreach ((string k, string word) in new[] { ("sha1", "SHA-1"), ("md5", "MD5"), ("crc32", "CRC-32") })
                    if (Text(recorded[k]) is { } h) return $"{word} {h.ToLowerInvariant()}";
                return "";
            }

            private static string TakenNamed(JsonObject recorded, RomHashes taken) =>
                Text(recorded["sha1"]) is not null ? $"SHA-1 {taken.Sha1}" : Text(recorded["md5"]) is not null ? $"MD5 {taken.Md5}" : $"CRC-32 {taken.Crc32}";

            public static long? RecordedSize(JsonObject doc) =>
                doc["game"] is JsonObject g ? Number(g["size"]) ?? (g["seen"] is JsonObject s ? Number(s["size"]) : null) : null;

            private static RomHashes? Hash(string at)
            {
                try { return RomHash.Managed.Of(at, 0, -1, default); }
                catch (Exception e) when (e is IOException or UnauthorizedAccessException) { return null; }
            }

            private static JsonObject SeenValue(long size, string mtime) => new() { ["size"] = size, ["mtime"] = mtime };

            public static (string Where, string Path) EntryPlace(PackagePlaces roots, string full)
            {
                if (roots.Library is { } library)
                {
                    string root = Path.TrimEndingDirectorySeparator(Path.GetFullPath(library));
                    if (Within(full, root) && full.Length > root.Length) return ("library", full[root.Length..].TrimStart('/', '\\').Replace('\\', '/'));
                }
                return ("absolute", full);
            }

            public static void Moved(PackagePlaces roots, JsonObject doc, string found)
            {
                (string place, string rel) = EntryPlace(roots, found);
                var game = doc["game"] as JsonObject;
                string old = Text(game?["path"]) ?? "", oldWhere = Text(game?["where"]) ?? "";
                if (game is not null)
                {
                    game["where"] = place;
                    game["path"] = rel;
                    if (SeenOf(found) is { } seen) game["seen"] = SeenValue(seen.Size, seen.Mtime);
                }
                static string? FolderOf(string p) => p.LastIndexOfAny(new[] { '/', '\\' }) is int i and >= 0 ? p[..i] : null;
                if (FolderOf(old) is not { } oldDir) return;
                string? newDir = FolderOf(rel);
                if (doc["companions"] is not JsonArray items) return;
                foreach (JsonObject c in items.OfType<JsonObject>())
                {
                    if (Text(c["where"]) != oldWhere || Text(c["path"]) is not { } p || !p.StartsWith(oldDir + "/", StringComparison.Ordinal)) continue;
                    string tail = p[(oldDir.Length + 1)..];
                    c["where"] = place;
                    c["path"] = place == "absolute" ? Under(Path.GetDirectoryName(found) ?? "", tail) : newDir is null ? tail : $"{newDir}/{tail}";
                }
            }

            public static string? FindInLibrary(PackagePlaces roots, long size, JsonObject recorded)
            {
                if (roots.Library is not { } library) return null;
                List<string> same;
                try { same = Directory.EnumerateFiles(library, "*", Recursive).Where(f => new FileInfo(f).Length == size).Select(Path.GetFullPath).ToList(); }
                catch (Exception e) when (e is IOException or UnauthorizedAccessException) { return null; }
                same.Sort(StringComparer.Ordinal);
                return same.FirstOrDefault(at => Hash(at) is { } h && SameDump(recorded, h));
            }

            private static void Write(string manifest, JsonObject doc)
            {
                try { AtomicFile.Managed.Write(manifest, new UTF8Encoding(false).GetBytes(GamePackage.Managed.Format(doc))); }
                catch (FormatException) { }
            }

            // A package whose game is now another dump, refused when the package is opened itself and not by a claimed file's path - see EmuSen_GamePackages.md §14.8 Q34.
            private sealed class ChangedDumpException(string message) : IOException(message);

            public static PackageResolution OpenPackage(PackagePlaces roots, string folder, string? opened)
            {
                string manifest = Path.Join(folder, GamePackage.FileName);
                JsonObject doc;
                try { doc = GamePackage.Managed.Parse(File.ReadAllText(manifest)); }
                catch (FormatException refused) { throw new IOException($"{manifest}: {refused.Message}"); }
                catch (Exception e) when (e is IOException or UnauthorizedAccessException) { throw new IOException($"{manifest}: {e.Message}"); }
                string game = Place(roots, folder, doc["game"]) ?? throw new IOException($"{manifest}: its game's library is not set (AppSettings.RomDirectory)");
                if (SeenOf(game) is { } now)
                {
                    var seen = (doc["game"] as JsonObject)?["seen"] as JsonObject;
                    bool unchanged = seen is null || (Number(seen["size"]) == now.Size && Text(seen["mtime"]) == now.Mtime);
                    if (unchanged || Recorded(doc) is not { } recorded) return new PackageResolution(game, folder, false);
                    RomHashes taken = Hash(game) ?? throw new IOException($"{game} could not be read to compare it with the package {folder}");
                    if (!SameDump(recorded, taken))
                        throw new ChangedDumpException($"{game} has {TakenNamed(recorded, taken)}, but the package {folder} was made for {Named(recorded)}: it is not the dump the package's saves belong to. pkg check says more; pkg make makes a package for this file once the old one is moved aside.");
                    ((JsonObject)doc["game"]!)["seen"] = SeenValue(now.Size, now.Mtime);
                    Write(manifest, doc);
                    return new PackageResolution(game, folder, false);
                }
                string where = Text((doc["game"] as JsonObject)?["where"]) ?? "";
                if ((where is "library" or "absolute") && RecordedSize(doc) is long size && Recorded(doc) is { } wanted && FindInLibrary(roots, size, wanted) is { } found)
                {
                    Moved(roots, doc, found);
                    Write(manifest, doc);
                    return new PackageResolution(found, folder, true);
                }
                if (opened is not null && File.Exists(opened)) return new PackageResolution(opened, folder, false);
                throw new IOException($"The package {folder} names {game}, which is gone, and no file of its size and hashes was found in the library.");
            }

            // Whether a claimed file still opens as its package: unless it is one a self-contained package's game was copied from and its size, time and SHA-1 are no longer those recorded - see EmuSen_GamePackages.md §19.
            public static bool SourceHolds(PackagePlaces roots, string folder, string full)
            {
                if (Read(Path.Join(folder, GamePackage.FileName)) is not { } doc) return true;
                if (Entries(doc, "sources").OfType<JsonObject>().FirstOrDefault(e => Place(roots, folder, e) == full) is not { } entry) return true;
                if (entry["seen"] is not JsonObject seen || (SeenOf(full) is { } now && Number(seen["size"]) == now.Size && Text(seen["mtime"]) == now.Mtime)) return true;
                if (Text(entry["sha1"]) is not { } want) return true;
                try { return string.Equals(RomHash.Managed.Of(full, 0, -1, default).Sha1, want, StringComparison.OrdinalIgnoreCase); }
                catch (Exception e) when (e is IOException or UnauthorizedAccessException) { return false; }
            }

            public static PackageResolution Resolve(PackagePlaces roots, string at)
            {
                string full = Path.GetFullPath(at);
                if (Directory.Exists(full) && File.Exists(Path.Join(full, GamePackage.FileName))) return OpenPackage(roots, full, null);
                // A package's zip is unpacked by the platform library alone, which keeps each file's time to the tick - see EmuSen_GamePackages.md §19.
                if (ArchiveImages.IsArchive(full) && File.Exists(full) && ZipManifest(full) is not null)
                    throw new IOException($"{full} holds a game package, which is unpacked into Packages by EmuSen's platform library, and it is not in use here.");
                if (Path.GetFileName(full) == GamePackage.FileName && File.Exists(full)) return OpenPackage(roots, Path.GetDirectoryName(full) ?? "", null);
                var plain = new PackageResolution(at, null, false);
                if (!File.Exists(full)) return plain;
                PackageIndex index = Index(roots);
                if (index.Claimant(full) is { } claimant)
                {
                    if (!SourceHolds(roots, claimant.Folder, full)) return plain;
                    try { return OpenPackage(roots, claimant.Folder, full); }
                    catch (ChangedDumpException) { return plain; }
                }
                long size = new FileInfo(full).Length;
                RomHashes? taken = null;
                foreach (ClaimingPackage p in index.Packages.Where(p => !File.Exists(p.Game)))
                {
                    string manifest = Path.Join(p.Folder, GamePackage.FileName);
                    if (Read(manifest) is not { } doc) continue;
                    string where = Text((doc["game"] as JsonObject)?["where"]) ?? "";
                    if (!(where is "library" or "absolute") || RecordedSize(doc) != size || Recorded(doc) is not { } recorded) continue;
                    taken ??= Hash(full);
                    if (taken is not null && SameDump(recorded, taken))
                    {
                        Moved(roots, doc, full);
                        Write(manifest, doc);
                        return new PackageResolution(full, p.Folder, true);
                    }
                }
                return plain;
            }
        }
    }
}
