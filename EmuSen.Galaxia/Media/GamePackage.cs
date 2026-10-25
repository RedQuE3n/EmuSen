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
        public static string NameOf(string root) => Library.SaveLibrary.Managed.PackageStem(root);

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
        public static GamePackage Parse(string text) => new(GalaxiaNative.Takes(text) ? Node(GalaxiaNative.PackageFormat(text)) : Managed.Parse(text));

        // The manifest at path; the refusal's words begin with the path.
        public static GamePackage Read(string path)
        {
            try
            {
                return new GamePackage(GalaxiaNative.Takes(path) ? Node(GalaxiaNative.PackageRead(path)) : Managed.Parse(File.ReadAllText(path)));
            }
            catch (PackageRefusedException refused) { throw new PackageRefusedException($"{path}: {refused.Message}", refused.Newer); }
        }

        // The bytes Write puts on disk; PackageRefusedException for a document the schema refuses.
        public string Format() => GalaxiaNative.Active ? GalaxiaNative.PackageFormat(Document.ToJsonString()) : Managed.Format(Document);

        // Atomic, as AtomicFile writes; false and a report when the write failed; a refused document throws and is not written.
        public bool Write(string path)
        {
            string compact = Document.ToJsonString();
            if (GalaxiaNative.Takes(path)) return GalaxiaNative.PackageWrite(path, compact);
            return AtomicFile.Managed.Write(path, new UTF8Encoding(false).GetBytes(Managed.Format(Document)));
        }

        private static JsonObject Node(string compact) => (JsonObject)JsonNode.Parse(compact)!;

        // The C# rule, held to the library's by the parity class - see EmuSen_RustPlatform.md §3.9.
        internal static class Managed
        {
            private static readonly JsonDocumentOptions Reading = new() { CommentHandling = JsonCommentHandling.Skip, AllowTrailingCommas = true, MaxDepth = 64 };
            private static readonly JsonSerializerOptions Writing = new() { WriteIndented = true };

            public static JsonObject Parse(string text)
            {
                JsonDocument doc;
                try { doc = JsonDocument.Parse(text, Reading); }
                catch (JsonException bad) { throw new PackageRefusedException($"the manifest is not JSON: {bad.Message}", newer: false); }
                using (doc)
                {
                    Check(doc.RootElement);
                    return (JsonObject)JsonNode.Parse(doc.RootElement.GetRawText(), documentOptions: Reading)!;
                }
            }

            public static string Format(JsonObject document)
            {
                using JsonDocument doc = JsonDocument.Parse(document.ToJsonString());
                Check(doc.RootElement);
                return document.ToJsonString(Writing) + Environment.NewLine;
            }

            private static PackageRefusedException Invalid(string words) => new(words, newer: false);

            private static PackageRefusedException Missing(string path) => Invalid($"{path} is missing");

            private static PackageRefusedException Wrong(string path, string want) => Invalid($"{path} must be {want}");

            private static string Child(string path, string name) => path.Length == 0 ? name : $"{path}.{name}";

            // The last member of a name, as the library's reader takes it.
            private static JsonElement? Get(JsonElement o, string name)
            {
                JsonElement? found = null;
                foreach (JsonProperty p in o.EnumerateObject()) if (p.NameEquals(name)) found = p.Value;
                return found;
            }

            private static bool IsText(JsonElement e) => e.ValueKind == JsonValueKind.String;

            private static string? Text(JsonElement? e) => e is { ValueKind: JsonValueKind.String } s ? s.GetString() : null;

            private static void WellFormed(JsonElement e, string path)
            {
                switch (e.ValueKind)
                {
                    case JsonValueKind.String:
                        try { e.GetString(); }
                        catch (InvalidOperationException) { throw Invalid($"{path} is not valid Unicode"); }
                        break;
                    case JsonValueKind.Array:
                        int i = 0;
                        foreach (JsonElement item in e.EnumerateArray()) WellFormed(item, $"{path}[{i++}]");
                        break;
                    case JsonValueKind.Object:
                        var seen = new HashSet<string>(StringComparer.Ordinal);
                        foreach (JsonProperty p in e.EnumerateObject())
                        {
                            string name;
                            try { name = p.Name; }
                            catch (InvalidOperationException) { throw Invalid($"{(path.Length == 0 ? "the manifest" : path)} has a member name that is not valid Unicode"); }
                            string at = Child(path, name);
                            if (!seen.Add(name)) throw Invalid($"{at} is written twice");
                            WellFormed(p.Value, at);
                        }
                        break;
                }
            }

            // Well formed, then format, then version, then every known field in the schema's order: the library's order exactly.
            public static void Check(JsonElement doc)
            {
                if (doc.ValueKind != JsonValueKind.Object) throw Invalid("the manifest is not a JSON object");
                WellFormed(doc, "");
                JsonElement? format = Get(doc, "format");
                if (format is null) throw Missing("format");
                if (Text(format) != FormatName) throw Wrong("format", "\"emusen-package\"");
                JsonElement? version = Get(doc, "version");
                if (version is null) throw Missing("version");
                string v = Text(version) is { } t && TwoNumbers(t) ? t : throw Wrong("version", "two numbers, major.minor");
                string major = v[..v.IndexOf('.')].TrimStart('0');
                if (major.Length == 0) throw Wrong("version", "1.x");
                if (major != "1") throw new PackageRefusedException($"the package format is {v}, written by a newer EmuSen; this build reads 1.x", newer: true);
                Optional(doc, "title", "", (e, p) => { if (!IsText(e)) throw Wrong(p, "a string or null"); });
                Required(doc, "system", "", NonEmpty);
                Required(doc, "mode", "", (e, p) => OneOf(e, p, ["linked", "self-contained"], "linked or self-contained"));
                Required(doc, "game", "", Entry);
                Optional(doc, "dump", "", Dump);
                Optional(doc, "companions", "", (e, p) => Array(e, p, Entry));
                Optional(doc, "patches", "", (e, p) => Array(e, p, Patch));
                Optional(doc, "patched", "", (e, p) =>
                {
                    Hashes(e, p, "an object or null");
                    Optional(e, "patches_sha1", p, (h, q) => Hex(h, q, 40));
                });
                if (Get(doc, "identity") is JsonElement identity) OneOf(identity, "identity", ["original", "patched"], "original or patched");
                Optional(doc, "container", "", (e, p) => { if (e.ValueKind != JsonValueKind.Object) throw Wrong(p, "an object or null"); });
            }

            private static bool TwoNumbers(string s)
            {
                int dot = s.IndexOf('.');
                if (dot <= 0 || dot == s.Length - 1) return false;
                return s.Remove(dot, 1).All(char.IsAsciiDigit) && s.IndexOf('.', dot + 1) < 0;
            }

            private static void Optional(JsonElement o, string name, string parent, Action<JsonElement, string> rule)
            {
                if (Get(o, name) is JsonElement e && e.ValueKind != JsonValueKind.Null) rule(e, Child(parent, name));
            }

            private static void Required(JsonElement o, string name, string parent, Action<JsonElement, string> rule)
            {
                string path = Child(parent, name);
                rule(Get(o, name) ?? throw Missing(path), path);
            }

            private static void NonEmpty(JsonElement e, string path)
            {
                if (!(Text(e) is { Length: > 0 })) throw Wrong(path, "a non-empty string");
            }

            private static void OneOf(JsonElement e, string path, string[] allowed, string want)
            {
                if (!(Text(e) is { } s && allowed.Contains(s, StringComparer.Ordinal))) throw Wrong(path, want);
            }

            private static void Whole(JsonElement e, string path, string want)
            {
                if (!(e.ValueKind == JsonValueKind.Number && e.TryGetInt64(out long n) && n >= 0)) throw Wrong(path, want);
            }

            private static void Hex(JsonElement e, string path, int digits)
            {
                if (!(Text(e) is { } s && s.Length == digits && s.All(char.IsAsciiHexDigit))) throw Wrong(path, $"{digits} hexadecimal digits or null");
            }

            private static void Array(JsonElement e, string path, Action<JsonElement, string> item)
            {
                if (e.ValueKind != JsonValueKind.Array) throw Wrong(path, "an array or null");
                int i = 0;
                foreach (JsonElement x in e.EnumerateArray()) item(x, $"{path}[{i++}]");
            }

            private static void Hashes(JsonElement e, string path, string want)
            {
                if (e.ValueKind != JsonValueKind.Object) throw Wrong(path, want);
                Optional(e, "crc32", path, (h, p) => Hex(h, p, 8));
                Optional(e, "md5", path, (h, p) => Hex(h, p, 32));
                Optional(e, "sha1", path, (h, p) => Hex(h, p, 40));
            }

            private static bool Absolute(string p) =>
                p.StartsWith('/') || p.StartsWith('\\') || (p.Length >= 3 && char.IsAsciiLetter(p[0]) && p[1] == ':' && (p[2] == '/' || p[2] == '\\'));

            private static bool Inside(string p) =>
                !p.StartsWith('/') && !p.StartsWith('\\') && !(p.Length >= 2 && char.IsAsciiLetter(p[0]) && p[1] == ':') && !p.Split('/', '\\').Contains("..");

            private static void Entry(JsonElement e, string path)
            {
                if (e.ValueKind != JsonValueKind.Object) throw Wrong(path, "an object");
                Required(e, "role", path, NonEmpty);
                Required(e, "where", path, (w, p) => OneOf(w, p, ["library", "package", "compressed", "absolute"], "library, package, compressed or absolute"));
                Required(e, "path", path, NonEmpty);
                string place = Text(Get(e, "where"))!, file = Text(Get(e, "path"))!, at = Child(path, "path");
                if (place == "absolute" && !Absolute(file)) throw Wrong(at, "an absolute path");
                if (place != "absolute" && !Inside(file)) throw Wrong(at, "a path inside its root, with no .. and not absolute");
                Optional(e, "size", path, (s, p) => Whole(s, p, "a whole number of 0 or more, or null"));
                Optional(e, "seen", path, (s, p) =>
                {
                    if (s.ValueKind != JsonValueKind.Object) throw Wrong(p, "an object or null");
                    Required(s, "size", p, (n, q) => Whole(n, q, "a whole number of 0 or more"));
                    Required(s, "mtime", p, (t, q) => { if (!IsText(t)) throw Wrong(q, "a string"); });
                });
            }

            private static void Dump(JsonElement e, string path)
            {
                if (e.ValueKind != JsonValueKind.Object) throw Wrong(path, "an object or null");
                Optional(e, "file", path, (h, p) => Hashes(h, p, "an object or null"));
                Optional(e, "transformed", path, (h, p) => Hashes(h, p, "an object or null"));
                Optional(e, "tracks", path, (t, p) => Array(t, p, Track));
                Optional(e, "sheet", path, (s, p) =>
                {
                    Hashes(s, p, "an object or null");
                    Required(s, "file", p, NonEmpty);
                });
                Optional(e, "form", path, (f, p) => OneOf(f, p, ["cue-bin", "lone-track", "iso"], "cue-bin, lone-track, iso or null"));
            }

            private static void Track(JsonElement e, string path)
            {
                Hashes(e, path, "an object");
                Required(e, "file", path, NonEmpty);
                Optional(e, "size", path, (s, p) => Whole(s, p, "a whole number of 0 or more, or null"));
            }

            private static void Patch(JsonElement e, string path)
            {
                if (e.ValueKind != JsonValueKind.Object) throw Wrong(path, "an object");
                Required(e, "file", path, NonEmpty);
                Required(e, "format", path, (f, p) => OneOf(f, p, ["ips", "bps", "ups"], "ips, bps or ups"));
                Optional(e, "sha1", path, (h, p) => Hex(h, p, 40));
                Optional(e, "source_sha1", path, (h, p) => Hex(h, p, 40));
                Optional(e, "source_crc32", path, (h, p) => Hex(h, p, 8));
                Optional(e, "target_crc32", path, (h, p) => Hex(h, p, 8));
            }
        }
    }
}
