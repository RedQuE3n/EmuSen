using System;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Net.Http;
using System.Threading;
using System.Threading.Tasks;

namespace EmuSen.Mistress.Library
{
    // libretro's slang shader pack, fetched from the address RetroArch's own updater uses, only when the player asks - see EmuSen_Settings_Reference.md §4.41.
    public static class SlangPackDownload
    {
        public const string PackAddress = "https://buildbot.libretro.com/assets/frontend/shaders_slang.zip";

        // Beside the presets, so the build a folder holds is read from the folder itself.
        public const string StampFile = ".emusen-pack";

        public static string DefaultDirectory => Path.Combine(EmuSen.Galaxia.Library.DataStore.Shaders, "RetroArch");

        // The build date the stamp records, or null when no pack is there.
        public static string? Installed(string directory)
        {
            string stamp = Path.Combine(directory, StampFile);
            return File.Exists(stamp) ? File.ReadAllText(stamp).Trim() : null;
        }

        // Every preset under the pack, as paths relative to it with forward slashes, sorted.
        public static string[] Presets(string directory) => !Directory.Exists(directory)
            ? Array.Empty<string>()
            : Directory.EnumerateFiles(directory, "*.slangp", SearchOption.AllDirectories)
                .Select(p => Path.GetRelativePath(directory, p).Replace('\\', '/'))
                .OrderBy(p => p, StringComparer.OrdinalIgnoreCase).ToArray();

        // The build date on success; the pack is unpacked beside its final name and swapped in, so a failed download leaves the old one.
        public static async Task<string> FetchAsync(HttpClient http, string directory, IProgress<(long Read, long? Total)>? progress = null, CancellationToken cancel = default)
        {
            string parent = Path.GetDirectoryName(Path.GetFullPath(directory))!;
            Directory.CreateDirectory(parent);
            string zip = directory + ".zip.part", staged = directory + ".part", old = directory + ".old";
            try
            {
                string built;
                using (HttpResponseMessage response = await http.GetAsync(PackAddress, HttpCompletionOption.ResponseHeadersRead, cancel))
                {
                    response.EnsureSuccessStatusCode();
                    built = (response.Content.Headers.LastModified ?? DateTimeOffset.UtcNow).UtcDateTime.ToString("yyyy-MM-dd HH:mm 'UTC'");
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
                // ExtractToDirectory refuses an entry that would land outside the folder.
                await Task.Run(() => ZipFile.ExtractToDirectory(zip, staged), cancel);
                if (!Directory.EnumerateFiles(staged, "*.slangp", SearchOption.AllDirectories).Any())
                    throw new InvalidDataException("The download holds no .slangp presets.");
                File.WriteAllText(Path.Combine(staged, StampFile), built);

                if (Directory.Exists(old)) Directory.Delete(old, recursive: true);
                if (Directory.Exists(directory)) Directory.Move(directory, old);
                Directory.Move(staged, directory);
                if (Directory.Exists(old)) Directory.Delete(old, recursive: true);
                return built;
            }
            finally
            {
                if (File.Exists(zip)) File.Delete(zip);
                if (Directory.Exists(staged)) Directory.Delete(staged, recursive: true);
            }
        }
    }
}
