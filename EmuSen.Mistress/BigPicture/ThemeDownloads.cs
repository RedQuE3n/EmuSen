using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Net.Http;
using System.Security.Cryptography;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using EmuSen.Galaxia.Library;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.Mistress.BigPicture
{
    // Where a downloaded theme came from, the commit, and when; read from themes.db since §25, and from a stage (f) stamp file only to import it.
    public sealed record ThemeStamp(string Owner, string Repository, string Branch, string? Commit, DateTime Downloaded)
    {
        public ThemeHost Host { get; init; } = ThemeHost.GitHub;
        public DateTimeOffset? Committed { get; init; }
        public string? Id { get; init; }
        public ThemeSource Source => new(Owner, Repository, Branch, Host);
    }

    // A theme folder the player can choose: one Mistress downloaded (with its stamp), or one read in place (without).
    public sealed record InstalledTheme(string Directory, string Name, ThemeStamp? Stamp)
    {
        public bool Downloaded => Stamp is not null;
    }

    // Files of a downloaded theme that differ from what its download wrote; theme-customizations is never counted, since an update keeps it.
    public sealed record ThemeLocalChanges(bool Known, IReadOnlyList<string> Modified, IReadOnlyList<string> Missing)
    {
        public static ThemeLocalChanges Unknown { get; } = new(false, [], []);
        public bool Any => Modified.Count + Missing.Count > 0;
    }

    public sealed class ThemeLocalChangesException(string directory, ThemeLocalChanges changes)
        : InvalidOperationException($"{Path.GetFileName(directory)} has local changes to {changes.Modified.Count + changes.Missing.Count} files; an update would replace them.")
    {
        public ThemeLocalChanges Changes { get; } = changes;
    }

    // Downloading, updating and removing themes under home/Themes from GitHub or GitLab, with every record in themes.db - see EmuSen_BigPicture.md §6, §16 and §25.
    public static class ThemeDownloads
    {
        public const string StampFile = ".emusen-theme";
        public const string Customizations = "theme-customizations";

        public static string Root => DataStore.Themes;

        // A repository name is a folder name here, so one that could climb out of home/Themes is refused.
        public static string DirectoryFor(ThemeSource source)
        {
            string name = source.Repository;
            if (name.Length == 0 || name is "." or ".." || name.StartsWith('.') || name.IndexOfAny(Path.GetInvalidFileNameChars()) >= 0 || name.Contains('/') || name.Contains('\\'))
                throw new InvalidDataException($"{name} cannot be a theme folder's name.");
            return Path.Combine(Root, name);
        }

        public static bool SamePath(string a, string b) =>
            string.Equals(Path.GetFullPath(a).TrimEnd(Path.DirectorySeparatorChar), Path.GetFullPath(b).TrimEnd(Path.DirectorySeparatorChar), StringComparison.Ordinal);

        private static bool DirectlyUnderRoot(string directory) =>
            string.Equals(Path.GetDirectoryName(Path.GetFullPath(directory).TrimEnd(Path.DirectorySeparatorChar)), Path.GetFullPath(Root).TrimEnd(Path.DirectorySeparatorChar), StringComparison.Ordinal);

        private static string Full(string directory) => Path.GetFullPath(directory).TrimEnd(Path.DirectorySeparatorChar);

        // The id a folder's stamp carries; the row of that id in themes.db is the folder's record.
        public static string? FolderId(string directory)
        {
            string file = Path.Combine(directory, StampFile);
            try
            {
                if (!File.Exists(file)) return null;
                using JsonDocument doc = JsonDocument.Parse(File.ReadAllText(file));
                return doc.RootElement.ValueKind == JsonValueKind.Object && doc.RootElement.TryGetProperty("Id", out JsonElement id) && id.ValueKind == JsonValueKind.String ? id.GetString() : null;
            }
            catch (Exception ex) when (ex is JsonException or IOException or UnauthorizedAccessException) { return null; }
        }

        private static void WriteFolderStamp(string directory, string id) =>
            File.WriteAllText(Path.Combine(directory, StampFile), JsonSerializer.Serialize(new { Id = id, Record = "home/Themes/" + ThemeRecords.FileName }, new JsonSerializerOptions { WriteIndented = true }));

        // A row alone names a path, which a folder copied in by hand can take; the folder's own id must match it too.
        public static InstalledRecord? RecordOf(string directory)
        {
            if (!Directory.Exists(directory) || !DirectlyUnderRoot(directory)) return null;
            ImportLegacy(directory);
            if (FolderId(directory) is not { } id || !File.Exists(ThemeRecords.DefaultPath)) return null;
            using ThemeRecords db = ThemeRecords.Open();
            return db.InstalledAt(directory) is { } row && row.Id == id ? row : null;
        }

        public static ThemeStamp? Stamp(string directory) => RecordOf(directory) is { } r ? StampOf(r) : null;

        private static ThemeStamp StampOf(InstalledRecord r) =>
            new(r.Source.Owner, r.Source.Repository, r.Source.Branch, r.Commit, r.Updated.UtcDateTime) { Host = r.Source.Host, Committed = r.Committed, Id = r.Id };

        // A folder Mistress downloaded: directly under home/Themes, with a row in themes.db and the same id in its stamp; nothing else may be removed or replaced.
        public static bool IsDownloaded(string directory) => RecordOf(directory) is not null;

        // Stage (f) wrote the whole record into the folder; such a folder is moved into themes.db once, keeping its source and commit.
        private static void ImportLegacy(string directory)
        {
            string file = Path.Combine(directory, StampFile);
            ThemeStamp? legacy;
            try
            {
                if (!File.Exists(file)) return;
                string text = File.ReadAllText(file);
                using (JsonDocument doc = JsonDocument.Parse(text))
                    if (doc.RootElement.ValueKind != JsonValueKind.Object || !doc.RootElement.TryGetProperty("Owner", out _)) return;
                legacy = JsonSerializer.Deserialize<ThemeStamp>(text);
            }
            catch (Exception ex) when (ex is JsonException or IOException or UnauthorizedAccessException) { return; }
            if (legacy is null || !DirectlyUnderRoot(directory)) return;
            string id = Guid.NewGuid().ToString("N");
            var when = new DateTimeOffset(DateTime.SpecifyKind(legacy.Downloaded, DateTimeKind.Utc));
            using (ThemeRecords db = ThemeRecords.Open())
                db.RememberInstall(new InstalledRecord(id, Full(directory), legacy.Source, legacy.Commit, null, when, when, HasManifest: false), null);
            WriteFolderStamp(directory, id);
        }

        // Every downloaded theme, by folder, then the folder being read in place when it is none of them.
        public static IReadOnlyList<InstalledTheme> Installed(string? inPlace = null)
        {
            var themes = new List<InstalledTheme>();
            if (Directory.Exists(Root))
                foreach (string dir in Directory.GetDirectories(Root).Where(d => !d.EndsWith(".part") && !d.EndsWith(".old") && !Path.GetFileName(d).StartsWith('.')).Order(StringComparer.Ordinal))
                    if (Stamp(dir) is { } stamp) themes.Add(new InstalledTheme(dir, NameOf(dir), stamp));
            if (!string.IsNullOrWhiteSpace(inPlace) && Directory.Exists(inPlace) && !themes.Any(t => SamePath(t.Directory, inPlace)))
                themes.Add(new InstalledTheme(Path.GetFullPath(inPlace), NameOf(inPlace), null));
            return themes;
        }

        public static string NameOf(string directory) => ThemeCapabilitiesReader.Read(directory).ThemeName;

        // Why a folder is not a theme that loads, or null when it is one: capabilities.xml without error, and a theme.xml that loads for a system.
        public static string? Validate(string directory)
        {
            ThemeCapabilities capabilities = ThemeCapabilitiesReader.Read(directory);
            if (capabilities.Diagnostics.FirstOrDefault(d => d.Severity == ThemeSeverity.Error) is { } error) return error.Message;
            string? system = File.Exists(Path.Combine(directory, "theme.xml")) ? "snes"
                : Directory.GetDirectories(directory).Select(Path.GetFileName).FirstOrDefault(d => d is not null && File.Exists(Path.Combine(directory, d, "theme.xml")));
            if (system is null) return "it holds no theme.xml";
            ResolvedTheme theme = ThemeLoader.Load(capabilities, new ThemeSystem(system, system, system), new ThemeChoices());
            return theme.IsThemed ? null : theme.Errors.First().Message;
        }

        // --- local changes ---

        // Size and time first, as git's index does; a file whose size or time moved is hashed, and one hashed equal is re-timed so the next look is cheap.
        public static ThemeLocalChanges LocalChanges(string directory)
        {
            if (RecordOf(directory) is not { HasManifest: true } row) return ThemeLocalChanges.Unknown;
            var modified = new List<string>();
            var missing = new List<string>();
            using ThemeRecords db = ThemeRecords.Open();
            foreach (ThemeFileRecord f in db.Files(row.Id))
            {
                var info = new FileInfo(Path.Combine(directory, f.Path));
                if (!info.Exists) { missing.Add(f.Path); continue; }
                long ticks = info.LastWriteTimeUtc.Ticks;
                if (info.Length == f.Bytes && ticks == f.Modified) continue;
                if (info.Length == f.Bytes && Hash(info.FullName) == f.Sha256) db.Restat(row.Id, f.Path, info.Length, ticks);
                else modified.Add(f.Path);
            }
            return new ThemeLocalChanges(true, modified, missing);
        }

        private static bool Tracked(string relative) =>
            relative != StampFile && !relative.StartsWith(Customizations + "/", StringComparison.Ordinal) && relative != Customizations;

        private static IReadOnlyList<ThemeFileRecord> Manifest(string root, CancellationToken cancel)
        {
            var files = new List<ThemeFileRecord>();
            foreach (string file in Directory.EnumerateFiles(root, "*", SearchOption.AllDirectories))
            {
                cancel.ThrowIfCancellationRequested();
                string relative = Path.GetRelativePath(root, file).Replace(Path.DirectorySeparatorChar, '/');
                if (!Tracked(relative)) continue;
                var info = new FileInfo(file);
                files.Add(new ThemeFileRecord(relative, info.Length, info.LastWriteTimeUtc.Ticks, Hash(file)));
            }
            return files;
        }

        private static string Hash(string file)
        {
            using FileStream s = File.OpenRead(file);
            return Convert.ToHexStringLower(SHA256.HashData(s));
        }

        // --- the hosts ---

        // The branch's newest commit and its date, for "update available" and "last updated"; null when the host could not say.
        public static async Task<(string Sha, DateTimeOffset? Date)?> HeadAsync(HttpClient http, ThemeSource source, CancellationToken cancel = default)
        {
            using JsonDocument? json = await ThemeHostJson.GetAsync(http, source.CommitAddress, cancel);
            if (json is null) return null;
            JsonElement r = json.RootElement;
            if (source.Host == ThemeHost.GitHub)
            {
                if (!r.TryGetProperty("sha", out JsonElement sha) || sha.ValueKind != JsonValueKind.String) return null;
                string? date = r.TryGetProperty("commit", out JsonElement c) && c.TryGetProperty("committer", out JsonElement who) && who.TryGetProperty("date", out JsonElement d) && d.ValueKind == JsonValueKind.String ? d.GetString() : null;
                return (sha.GetString()!, ThemeHostJson.Date(date));
            }
            if (!r.TryGetProperty("id", out JsonElement id) || id.ValueKind != JsonValueKind.String) return null;
            return (id.GetString()!, ThemeHostJson.Date(r.TryGetProperty("committed_date", out JsonElement cd) && cd.ValueKind == JsonValueKind.String ? cd.GetString() : null));
        }

        public static async Task<string?> LatestCommitAsync(HttpClient http, ThemeSource source, CancellationToken cancel = default) =>
            (await HeadAsync(http, source, cancel))?.Sha;

        // --- download, update, remove ---

        // Downloads the branch's archive and swaps it in only when it loads; the old theme, its theme-customizations and files the player added survive - see EmuSen_BigPicture.md §16.4 and §25.
        public static async Task<ThemeStamp> FetchAsync(HttpClient http, ThemeSource source, IProgress<(long Read, long? Total)>? progress = null, CancellationToken cancel = default,
            IProgress<(int Done, int Total)>? unpacked = null, bool replaceLocalChanges = false)
        {
            string directory = DirectoryFor(source);
            Directory.CreateDirectory(Root);
            Recover(directory);
            InstalledRecord? before = null;
            if (Directory.Exists(directory))
            {
                before = RecordOf(directory) ?? throw new InvalidOperationException($"{directory} was not downloaded by Mistress, so it is not replaced.");
                if (!before.Source.SameRepository(source)) throw new InvalidOperationException($"{directory} holds {before.Source.Url}, not {source.Url}, so it is not replaced.");
                if (!replaceLocalChanges && LocalChanges(directory) is { Any: true } changes) throw new ThemeLocalChangesException(directory, changes);
            }
            string zip = directory + ".zip.part", staged = directory + ".part", old = directory + ".old";
            try
            {
                (string Sha, DateTimeOffset? Date)? head = null;
                try { head = await HeadAsync(http, source, cancel); }
                catch (HttpRequestException) { }

                using (var request = new HttpRequestMessage(HttpMethod.Get, source.ArchiveAddress))
                using (HttpResponseMessage response = await http.SendAsync(request, HttpCompletionOption.ResponseHeadersRead, cancel))
                {
                    response.EnsureSuccessStatusCode();
                    long? total = response.Content.Headers.ContentLength;
                    await using Stream body = await response.Content.ReadAsStreamAsync(cancel);
                    await using FileStream file = File.Create(zip);
                    var buffer = new byte[1 << 16];
                    long read = 0;
                    for (int n; (n = await body.ReadAsync(buffer, cancel)) > 0;)
                    {
                        await file.WriteAsync(buffer.AsMemory(0, n), cancel);
                        read += n;
                        progress?.Report((read, total));
                    }
                }

                if (Directory.Exists(staged)) Directory.Delete(staged, recursive: true);
                IReadOnlyList<ThemeFileRecord> files = await Task.Run(() =>
                {
                    Extract(zip, staged, unpacked, cancel);
                    string found = ThemeRoot(staged) ?? throw new InvalidDataException("The download holds no capabilities.xml.");
                    if (Validate(found) is { } why) throw new InvalidDataException($"The download is not a theme that loads: {why}");
                    return Manifest(found, cancel);
                }, cancel);
                string root = ThemeRoot(staged)!;
                string id = before?.Id ?? Guid.NewGuid().ToString("N");
                WriteFolderStamp(root, id);
                cancel.ThrowIfCancellationRequested();

                IReadOnlyList<ThemeFileRecord>? oldFiles = null;
                if (before is { HasManifest: true })
                    using (ThemeRecords db = ThemeRecords.Open()) oldFiles = db.Files(before.Id);
                if (Directory.Exists(directory)) Directory.Move(directory, old);
                Directory.Move(root, directory);
                KeepCustomizations(old, directory);
                KeepAdded(old, directory, oldFiles);
                if (Directory.Exists(old)) Directory.Delete(old, recursive: true);

                DateTimeOffset now = DateTimeOffset.UtcNow;
                var record = new InstalledRecord(id, Full(directory), source, head?.Sha, head?.Date, before?.Installed ?? now, now, HasManifest: true);
                using (ThemeRecords db = ThemeRecords.Open()) db.RememberInstall(record, files);
                return StampOf(record);
            }
            finally
            {
                if (File.Exists(zip)) File.Delete(zip);
                if (Directory.Exists(staged)) Directory.Delete(staged, recursive: true);
            }
        }

        // Every entry by hand, so a cancel stops between entries and an entry that would land outside the folder is refused.
        private static void Extract(string zip, string staged, IProgress<(int Done, int Total)>? unpacked, CancellationToken cancel)
        {
            string top = Path.GetFullPath(staged) + Path.DirectorySeparatorChar;
            Directory.CreateDirectory(staged);
            using ZipArchive archive = ZipFile.OpenRead(zip);
            int done = 0, total = archive.Entries.Count;
            foreach (ZipArchiveEntry entry in archive.Entries)
            {
                cancel.ThrowIfCancellationRequested();
                unpacked?.Report((done++, total));
                string destination = Path.GetFullPath(Path.Combine(staged, entry.FullName));
                if (!destination.StartsWith(top, StringComparison.Ordinal)) throw new InvalidDataException($"{entry.FullName} would land outside the theme folder.");
                if (entry.FullName.EndsWith('/')) { Directory.CreateDirectory(destination); continue; }
                Directory.CreateDirectory(Path.GetDirectoryName(destination)!);
                entry.ExtractToFile(destination, overwrite: true);
            }
        }

        // A host's archive holds one folder named for the repository and branch; a zip of the theme's own files is accepted too.
        private static string? ThemeRoot(string staged)
        {
            if (File.Exists(Path.Combine(staged, ThemeCapabilitiesReader.FileName))) return staged;
            string[] dirs = Directory.GetDirectories(staged);
            return dirs.Length == 1 && File.Exists(Path.Combine(dirs[0], ThemeCapabilitiesReader.FileName)) ? dirs[0] : null;
        }

        // The player's theme-customizations move from the old folder into the new one, replacing any the archive carried.
        private static void KeepCustomizations(string old, string directory)
        {
            string kept = Path.Combine(old, Customizations), arrived = Path.Combine(directory, Customizations);
            if (!Directory.Exists(kept)) return;
            if (Directory.Exists(arrived)) Directory.Delete(arrived, recursive: true);
            Directory.Move(kept, arrived);
        }

        // Files the player added, which the old download did not write, move across unless the new one writes the same path (USERGUIDE.md "Theme downloader").
        private static void KeepAdded(string old, string directory, IReadOnlyList<ThemeFileRecord>? oldFiles)
        {
            if (oldFiles is null || !Directory.Exists(old)) return;
            var written = oldFiles.Select(f => f.Path).ToHashSet(StringComparer.Ordinal);
            foreach (string file in Directory.EnumerateFiles(old, "*", SearchOption.AllDirectories).ToList())
            {
                string relative = Path.GetRelativePath(old, file).Replace(Path.DirectorySeparatorChar, '/');
                if (!Tracked(relative) || written.Contains(relative)) continue;
                string target = Path.Combine(directory, relative);
                if (File.Exists(target)) continue;
                Directory.CreateDirectory(Path.GetDirectoryName(target)!);
                File.Move(file, target);
            }
        }

        // A swap cut short leaves .old: the theme goes back, or its customizations go forward, before anything else runs.
        private static void Recover(string directory)
        {
            string old = directory + ".old";
            if (!Directory.Exists(old)) return;
            if (!Directory.Exists(directory)) Directory.Move(old, directory);
            else
            {
                if (!Directory.Exists(Path.Combine(directory, Customizations))) KeepCustomizations(old, directory);
                if (!Directory.Exists(Path.Combine(old, Customizations))) Directory.Delete(old, recursive: true);
            }
        }

        // Only a folder Mistress downloaded; a folder read in place, or one copied in by hand, is never written, let alone removed (§6).
        public static void Remove(string directory)
        {
            if (RecordOf(directory) is not { } row) throw new InvalidOperationException($"{directory} was not downloaded by Mistress, so it is not removed.");
            Directory.Delete(directory, recursive: true);
            using ThemeRecords db = ThemeRecords.Open();
            db.ForgetInstall(row.Id);
        }
    }

    // A host's JSON answer, with GitHub's hourly limit reported as such rather than as a failure of the theme.
    public static class ThemeHostJson
    {
        public static async Task<JsonDocument?> GetAsync(HttpClient http, string address, CancellationToken cancel)
        {
            using var request = new HttpRequestMessage(HttpMethod.Get, address);
            if (address.StartsWith("https://api.github.com/", StringComparison.Ordinal)) request.Headers.Accept.ParseAdd("application/vnd.github+json");
            using HttpResponseMessage response = await http.SendAsync(request, cancel);
            if ((int)response.StatusCode is 403 or 429 && response.Headers.TryGetValues("x-ratelimit-remaining", out IEnumerable<string>? left) && left.FirstOrDefault() == "0")
            {
                string reset = response.Headers.TryGetValues("x-ratelimit-reset", out IEnumerable<string>? r) && long.TryParse(r.FirstOrDefault(), out long at)
                    ? $" until {DateTimeOffset.FromUnixTimeSeconds(at).ToLocalTime():HH:mm}" : "";
                throw new HttpRequestException($"GitHub's limit of 60 requests an hour for a computer without an account is used up{reset}.");
            }
            if (!response.IsSuccessStatusCode) return null;
            return JsonDocument.Parse(await response.Content.ReadAsStringAsync(cancel));
        }

        public static DateTimeOffset? Date(string? text) =>
            DateTimeOffset.TryParse(text, CultureInfo.InvariantCulture, DateTimeStyles.AssumeUniversal, out DateTimeOffset d) ? d.ToUniversalTime() : null;
    }
}
