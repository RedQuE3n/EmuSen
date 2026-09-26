using System;
using System.Collections.Generic;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Net.Http;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using EmuSen.Galaxia.Library;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.Mistress.BigPicture
{
    // A theme's GitHub repository and branch: the archive and the latest commit are asked of GitHub by these - see EmuSen_BigPicture.md §6.
    public sealed record ThemeSource(string Owner, string Repository, string Branch)
    {
        // The ES-DE edition the user chose (§10.1, Q1); fetched only when the player asks, never bundled.
        public static ThemeSource ArtBookNext { get; } = new("anthonycaccese", "art-book-next-es-de", "main");

        public string Url => $"https://github.com/{Owner}/{Repository}";
        public string ArchiveAddress => $"https://codeload.github.com/{Owner}/{Repository}/zip/refs/heads/{Branch}";
        public string CommitAddress => $"https://api.github.com/repos/{Owner}/{Repository}/commits/{Branch}";
    }

    // What .emusen-theme records beside a downloaded theme: where it came from, the commit, and when.
    public sealed record ThemeStamp(string Owner, string Repository, string Branch, string? Commit, DateTime Downloaded)
    {
        public ThemeSource Source => new(Owner, Repository, Branch);
    }

    // A theme folder the player can choose: one Mistress downloaded (with its stamp), or one read in place (without).
    public sealed record InstalledTheme(string Directory, string Name, ThemeStamp? Stamp)
    {
        public bool Downloaded => Stamp is not null;
    }

    // Downloading, updating and removing themes under home/Themes, and the checks a download passes before it replaces anything - see EmuSen_BigPicture.md §6 and §16.
    public static class ThemeDownloads
    {
        public const string StampFile = ".emusen-theme";
        public const string Customizations = "theme-customizations";

        private static readonly JsonSerializerOptions Json = new() { WriteIndented = true };

        public static string Root => DataStore.Themes;

        public static string DirectoryFor(ThemeSource source) => Path.Combine(Root, source.Repository);

        public static ThemeStamp? Stamp(string directory)
        {
            string file = Path.Combine(directory, StampFile);
            try { return File.Exists(file) ? JsonSerializer.Deserialize<ThemeStamp>(File.ReadAllText(file)) : null; }
            catch (JsonException) { return null; }
        }

        // A folder Mistress downloaded: directly under home/Themes and carrying its stamp; nothing else may be removed.
        public static bool IsDownloaded(string directory)
        {
            string full = Path.GetFullPath(directory).TrimEnd(Path.DirectorySeparatorChar);
            return string.Equals(Path.GetDirectoryName(full), Path.GetFullPath(Root).TrimEnd(Path.DirectorySeparatorChar), StringComparison.Ordinal)
                   && Stamp(full) is not null;
        }

        // Every downloaded theme, by name, then the folder being read in place when it is none of them.
        public static IReadOnlyList<InstalledTheme> Installed(string? inPlace = null)
        {
            var themes = new List<InstalledTheme>();
            if (Directory.Exists(Root))
                foreach (string dir in Directory.GetDirectories(Root).Where(d => !d.EndsWith(".part") && !d.EndsWith(".old")).Order(StringComparer.Ordinal))
                    if (Stamp(dir) is { } stamp) themes.Add(new InstalledTheme(dir, NameOf(dir), stamp));
            if (!string.IsNullOrWhiteSpace(inPlace) && Directory.Exists(inPlace) && !themes.Any(t => SamePath(t.Directory, inPlace)))
                themes.Add(new InstalledTheme(Path.GetFullPath(inPlace), NameOf(inPlace), null));
            return themes;
        }

        public static bool SamePath(string a, string b) =>
            string.Equals(Path.GetFullPath(a).TrimEnd(Path.DirectorySeparatorChar), Path.GetFullPath(b).TrimEnd(Path.DirectorySeparatorChar), StringComparison.Ordinal);

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

        // The branch's newest commit on GitHub, for "update available"; null when GitHub could not say.
        public static async Task<string?> LatestCommitAsync(HttpClient http, ThemeSource source, CancellationToken cancel = default)
        {
            using var request = new HttpRequestMessage(HttpMethod.Get, source.CommitAddress);
            request.Headers.Accept.ParseAdd("application/vnd.github+json");
            using HttpResponseMessage response = await http.SendAsync(request, cancel);
            if (!response.IsSuccessStatusCode) return null;
            using JsonDocument json = JsonDocument.Parse(await response.Content.ReadAsStringAsync(cancel));
            return json.RootElement.TryGetProperty("sha", out JsonElement sha) && sha.ValueKind == JsonValueKind.String ? sha.GetString() : null;
        }

        // Downloads the branch's archive and swaps it in only when it loads; the old theme and its theme-customizations survive any failure - see EmuSen_BigPicture.md §16.
        public static async Task<ThemeStamp> FetchAsync(HttpClient http, ThemeSource source, IProgress<(long Read, long? Total)>? progress = null, CancellationToken cancel = default)
        {
            string directory = DirectoryFor(source);
            Directory.CreateDirectory(Root);
            Recover(directory);
            string zip = directory + ".zip.part", staged = directory + ".part", old = directory + ".old";
            try
            {
                string? commit = null;
                try { commit = await LatestCommitAsync(http, source, cancel); }
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
                await Task.Run(() => Extract(zip, staged, cancel), cancel);
                string root = ThemeRoot(staged) ?? throw new InvalidDataException("The download holds no capabilities.xml.");
                if (Validate(root) is { } why) throw new InvalidDataException($"The download is not a theme that loads: {why}");
                var stamp = new ThemeStamp(source.Owner, source.Repository, source.Branch, commit, DateTime.UtcNow);
                File.WriteAllText(Path.Combine(root, StampFile), JsonSerializer.Serialize(stamp, Json));
                cancel.ThrowIfCancellationRequested();

                if (Directory.Exists(directory)) Directory.Move(directory, old);
                Directory.Move(root, directory);
                KeepCustomizations(old, directory);
                if (Directory.Exists(old)) Directory.Delete(old, recursive: true);
                return stamp;
            }
            finally
            {
                if (File.Exists(zip)) File.Delete(zip);
                if (Directory.Exists(staged)) Directory.Delete(staged, recursive: true);
            }
        }

        // Every entry by hand, so a cancel stops between entries and an entry that would land outside the folder is refused.
        private static void Extract(string zip, string staged, CancellationToken cancel)
        {
            string top = Path.GetFullPath(staged) + Path.DirectorySeparatorChar;
            Directory.CreateDirectory(staged);
            using ZipArchive archive = ZipFile.OpenRead(zip);
            foreach (ZipArchiveEntry entry in archive.Entries)
            {
                cancel.ThrowIfCancellationRequested();
                string destination = Path.GetFullPath(Path.Combine(staged, entry.FullName));
                if (!destination.StartsWith(top, StringComparison.Ordinal)) throw new InvalidDataException($"{entry.FullName} would land outside the theme folder.");
                if (entry.FullName.EndsWith('/')) { Directory.CreateDirectory(destination); continue; }
                Directory.CreateDirectory(Path.GetDirectoryName(destination)!);
                entry.ExtractToFile(destination, overwrite: true);
            }
        }

        // GitHub's archive holds one folder named for the repository and branch; a zip of the theme's own files is accepted too.
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

        // Only a folder Mistress downloaded; a folder read in place is never written, let alone removed (§6).
        public static void Remove(string directory)
        {
            if (!IsDownloaded(directory)) throw new InvalidOperationException($"{directory} was not downloaded by Mistress, so it is not removed.");
            Directory.Delete(directory, recursive: true);
        }
    }
}
