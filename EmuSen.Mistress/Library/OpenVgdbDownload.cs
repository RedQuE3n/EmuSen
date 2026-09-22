using System;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Net.Http;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;

namespace EmuSen.Mistress.Library
{
    // Fetches OpenVGDB's newest release from GitHub, as OpenEmu does on first use; only when the player has turned covers on - see EmuSen_Settings_Reference.md §4.39.
    public static class OpenVgdbDownload
    {
        public const string LatestRelease = "https://api.github.com/repos/OpenVGDB/OpenVGDB/releases/latest";

        // The release's tag on success; the database is written beside its final name and moved into place, so a failed download leaves nothing.
        public static async Task<string> FetchAsync(HttpClient http, string target, CancellationToken cancel = default)
        {
            using JsonDocument release = JsonDocument.Parse(await http.GetStringAsync(LatestRelease, cancel));
            string tag = release.RootElement.GetProperty("tag_name").GetString() ?? "?";
            string? zip = release.RootElement.GetProperty("assets").EnumerateArray()
                .Where(a => a.GetProperty("name").GetString()?.EndsWith(".zip", StringComparison.OrdinalIgnoreCase) == true)
                .Select(a => a.GetProperty("browser_download_url").GetString())
                .FirstOrDefault();
            if (zip is null) throw new InvalidDataException($"OpenVGDB {tag} has no zip to download.");

            Directory.CreateDirectory(Path.GetDirectoryName(target)!);
            string part = target + ".part";
            await using (Stream body = await http.GetStreamAsync(zip, cancel))
            {
                using var archive = new ZipArchive(body, ZipArchiveMode.Read);
                ZipArchiveEntry entry = archive.Entries.FirstOrDefault(e => e.Name.Equals(OpenVgdb.FileName, StringComparison.OrdinalIgnoreCase))
                    ?? throw new InvalidDataException($"OpenVGDB {tag}'s zip holds no {OpenVgdb.FileName}.");
                await using FileStream file = File.Create(part);
                await using Stream inside = entry.Open();
                await inside.CopyToAsync(file, cancel);
            }
            using (OpenVgdb? check = OpenVgdb.Open(part))
                if (check is null) { File.Delete(part); throw new InvalidDataException($"OpenVGDB {tag} did not open as a database."); }
            File.Move(part, target, overwrite: true);
            return tag;
        }
    }
}
