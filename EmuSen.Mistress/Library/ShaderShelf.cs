using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Threading.Tasks;
using EmuSen.Galaxia.Library;
using EmuSen.Serenity.Shaders;
using EmuSen.Serenity.Slang;

namespace EmuSen.Mistress.Library
{
    // The shaders Settings offers a console: the built-in filters, the downloaded slang pack's presets, each one's parameters read once and kept, and the pack's download - see EmuSen_BigPicture.md §52.8.
    public sealed class ShaderShelf
    {
        public const int CacheLimit = 8;

        private readonly RecentCache<string, IReadOnlyList<SlangParameter>> _read = new(CacheLimit, StringComparer.Ordinal);

        public ShaderShelf(string? pack = null)
        {
            Pack = pack ?? SlangPackDownload.DefaultDirectory;
            Load();
        }

        public string Pack { get; }

        public IReadOnlyList<ShaderEntry> Presets { get; private set; } = [];

        // The pack's build stamp, null while none is downloaded.
        public string? Built { get; private set; }

        // The download's progress or its failure, null when there is nothing to say.
        public string? Said { get; private set; }

        public Task? Downloading { get; private set; }

        public int Reads { get; private set; }

        public void Load()
        {
            Presets = ShaderCatalog.Presets(SlangPackDownload.Presets(Pack));
            Built = SlangPackDownload.Installed(Pack);
        }

        // Whether a preset's parameters are kept, for a test of the cache.
        public bool IsRead(string relative) => _read.Contains(Path.GetFullPath(Path.Combine(Pack, relative)));

        public bool Has(ShaderEntry entry) => !entry.IsPreset || File.Exists(Path.Combine(Pack, entry.Relative!));

        // A shader's parameters: a built-in filter's own, a preset's read from the pack once and kept; none for a preset the pack no longer has.
        public IReadOnlyList<SlangParameter> ParametersOf(string stored)
        {
            ShaderEntry entry = ShaderCatalog.For(stored);
            if (!entry.IsPreset) return ScreenFilters.Find(entry.Stored).Filter?.Parameters ?? [];
            string path = Path.GetFullPath(Path.Combine(Pack, entry.Relative!));
            if (_read.TryGet(path, out var known)) return known;
            if (!File.Exists(path)) return [];
            Reads++;
            IReadOnlyList<SlangParameter> read = SlangParameters.Read(SlangPreset.Load(path));
            _read.Add(path, read, used: true);
            return read;
        }

        // Fetches libretro's slang pack into the folder, telling changed of each step and of the end.
        public Task Download(Func<HttpClient> http, Action changed) => Downloading ??= DownloadAsync(http, changed);

        private async Task DownloadAsync(Func<HttpClient> http, Action changed)
        {
            Said = "Downloading libretro's slang pack...";
            changed();
            var progress = new Progress<(long Read, long? Total)>(p =>
            {
                Said = p.Total is long total ? $"Downloading... {p.Read / 1048576.0:F1} of {total / 1048576.0:F1} MB" : $"Downloading... {p.Read / 1048576.0:F1} MB";
                changed();
            });
            try
            {
                // Its own client with a long timeout: 54 MB outlasts the thirty seconds a cover lookup is given.
                using HttpClient client = http();
                client.Timeout = TimeSpan.FromMinutes(30);
                await SlangPackDownload.FetchAsync(client, Pack, progress);
                _read.Clear();
                Said = null;
            }
            catch (Exception ex) when (ex is HttpRequestException or IOException or InvalidDataException or TaskCanceledException or UnauthorizedAccessException)
            {
                ErrorLog.Error("shaders", "Could not download the shader pack", ex);
                Said = $"Could not download the pack: {ex.Message}";
            }
            Load();
            Downloading = null;
            changed();
        }
    }
}
