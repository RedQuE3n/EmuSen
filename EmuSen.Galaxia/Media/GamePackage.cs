using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;
using System.Text.Json.Nodes;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Native;

namespace EmuSen.Galaxia.Media
{
    // A manifest this build does not take: not JSON, refused by the schema, or of a newer major (Newer) - see EmuSen_GamePackages.md §3.3.
    public sealed class PackageRefusedException(string message, bool newer) : FormatException(message)
    {
        public bool Newer { get; } = newer;
    }

    // A file of a package's set, game's shape and each companion's: its role, where its path is relative to, and its size.
    public sealed record PackageFile(string Role, string Where, string Path, long? Size = null);

    // A game package's manifest, version 1: the whole document, unknown fields kept, read and written by Galaxia's rule - see EmuSen_Galaxia.md §3.10.
    public sealed class GamePackage
    {
        public const string FileName = "package.json";
        public const string FormatName = "emusen-package";
        public const int Major = 1;
        public const int Minor = 0;

        // A package's own places for a game's battery saves, its states and its cheat list - see EmuSen_GamePackages.md §3.2.
        public const string SavesFolder = "saves", StatesFolder = "states", CheatsFile = "cheats.json";

        // A packaged game's own settings, the crop among them, and its art under ES-DE's type folders - see EmuSen_GamePackages.md §18.2 and §18.3.
        public const string SettingsFile = "settings.json", MediaFolder = "media";

        // The name a package's files go by: its folder's, less the extension - see EmuSen_GamePackages.md §14.2.
        public static string NameOf(string root) => Path.GetFileNameWithoutExtension(Path.TrimEndingDirectorySeparator(root));

        // A packaged game's art of one ES-DE type folder: <package>/media/<type folder>/<package's name><extension> - see EmuSen_GamePackages.md §18.3.
        public static string MediaPathFor(string root, string typeFolder, string extension) => Path.Combine(root, MediaFolder, typeFolder, NameOf(root) + extension);

        private GamePackage(JsonObject document) => Document = document;

        // Every member as read or set, in its order; a caller edits it and Write checks it.
        public JsonObject Document { get; }

        public string Version => (string)Document["version"]!;

        public string? Title => (string?)Document["title"];

        public string System => (string)Document["system"]!;

        public string Mode => (string)Document["mode"]!;

        public PackageFile Game
        {
            get
            {
                var g = (JsonObject)Document["game"]!;
                return new PackageFile((string)g["role"]!, (string)g["where"]!, (string)g["path"]!, (long?)g["size"]);
            }
        }

        // How many soft patches the list holds - see EmuSen_GamePackages.md §5.4.
        public int Patches => Document["patches"] is JsonArray list ? list.Count : 0;

        // The patched image's hashes as "patched" records them; null with no patches or none taken - see EmuSen_GamePackages.md §5.5.
        public Library.RomHashes? Patched =>
            Patches > 0 && Document["patched"] is JsonObject p && Text(p, "md5") is { } md5 && Text(p, "crc32") is { } crc && Text(p, "sha1") is { } sha
            && p["size"] is JsonValue size && size.TryGetValue(out long bytes)
                ? new Library.RomHashes(md5, crc, sha, bytes) : null;

        // What a state saved on the patched image records as its ROM.
        public (string Md5, long Size)? PatchedIdentity => Patched is { } h ? (h.Md5, h.Size) : null;

        // Whether the scraper sends the patched image's hashes first: patches, and "identity" set to "patched".
        public bool ScrapesAsPatched => (string?)Document["identity"] == "patched" && Patched is not null;

        // The key a patched game's saves and states are named apart by: eight hex digits of the list's patches_sha1 rule, an entry with no SHA-1 keyed by its file's name; null with no patches - see EmuSen_GamePackages.md §5.5.
        public string? SaveKey
        {
            get
            {
                if (Document["patches"] is not JsonArray { Count: > 0 } list) return null;
                var lines = new StringBuilder();
                foreach (JsonNode? entry in list)
                {
                    string format = entry is JsonObject e && e["format"] is JsonValue f && f.TryGetValue(out string? fs) ? fs : "";
                    string sha = entry is JsonObject o && o["sha1"] is JsonValue h && h.TryGetValue(out string? hs) ? hs.ToLowerInvariant()
                        : entry is JsonObject n && n["file"] is JsonValue file && file.TryGetValue(out string? name) ? name : "";
                    lines.Append(format).Append(' ').Append(sha).Append('\n');
                }
                return Convert.ToHexString(global::System.Security.Cryptography.SHA1.HashData(Encoding.UTF8.GetBytes(lines.ToString())))[..SaveKeyDigits].ToLowerInvariant();
            }
        }

        public const int SaveKeyDigits = 8;

        // The save key of the package at root as its manifest reads now; null where it lists no patches or does not read.
        public static string? SaveKeyAt(string? root)
        {
            if (string.IsNullOrEmpty(root)) return null;
            try { return Read(global::System.IO.Path.Combine(root, FileName)).SaveKey; }
            catch (Exception e) when (e is IOException or FormatException or UnauthorizedAccessException) { return null; }
        }

        private static string? Text(JsonObject o, string name) => o[name] is JsonValue v && v.TryGetValue(out string? s) ? s.ToLowerInvariant() : null;

        // A new manifest in the schema's order, with the fields later steps fill left empty.
        public static GamePackage Create(string? title, string system, string mode, PackageFile game)
        {
            var file = new JsonObject { ["role"] = game.Role, ["where"] = game.Where, ["path"] = game.Path };
            if (game.Size is long size) file["size"] = size;
            return new GamePackage(new JsonObject
            {
                ["format"] = FormatName,
                ["version"] = $"{Major}.{Minor}",
                ["title"] = title,
                ["system"] = system,
                ["mode"] = mode,
                ["game"] = file,
                ["dump"] = null,
                ["companions"] = new JsonArray(),
                ["patches"] = new JsonArray(),
                ["patched"] = null,
                ["identity"] = "original",
                ["container"] = null,
            });
        }

        // A manifest's text, checked; PackageRefusedException with words naming the field.
        public static GamePackage Parse(string text) => new(Node(GalaxiaNative.PackageFormat(text)));

        // The manifest at path; the refusal's words begin with the path.
        public static GamePackage Read(string path)
        {
            try
            {
                return new GamePackage(Node(GalaxiaNative.PackageRead(path)));
            }
            catch (PackageRefusedException refused) { throw new PackageRefusedException($"{path}: {refused.Message}", refused.Newer); }
        }

        // The bytes Write puts on disk; PackageRefusedException for a document the schema refuses.
        public string Format() => GalaxiaNative.PackageFormat(Document.ToJsonString());

        // Atomic, as AtomicFile writes; false and a report when the write failed; a refused document throws and is not written.
        public bool Write(string path)
        {
            return GalaxiaNative.PackageWrite(path, Document.ToJsonString());
        }

        private static JsonObject Node(string compact) => (JsonObject)JsonNode.Parse(compact)!;
    }
}
