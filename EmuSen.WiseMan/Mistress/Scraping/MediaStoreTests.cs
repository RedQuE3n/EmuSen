using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using EmuSen.Crystal.Client.Scrape;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.Mistress.Scraping;
using EmuSen.WiseMan.Fixtures;
using Microsoft.Data.Sqlite;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // media.db's schema, what it answers by path, a closed handle, and the order a picture is looked for in, through the binding; the queue's order and a retry's due time are emusen-scrape's store_queue tests - see EmuSen_BigPicture.md §5.6, §17 and EmuSen_Backend.md §40.5.
    public class MediaStoreTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenMediaStoreTests", Guid.NewGuid().ToString("N"));
        private static readonly DateTimeOffset Now = new(2026, 9, 26, 12, 0, 0, TimeSpan.Zero);

        public void Dispose()
        {
            try { Directory.Delete(_root, recursive: true); } catch { }
        }

        private string Store => Path.Combine(_root, "Media");

        private static string Settings(IReadOnlyDictionary<string, object?>? more = null)
        {
            var s = new Dictionary<string, object?> { ["developer_files"] = false, ["openemu"] = false, ["test"] = new { offline = true, clock = NativeScraping.Ticks(Now) } };
            foreach (var (key, value) in more ?? new Dictionary<string, object?>()) s[key] = value;
            return JsonSerializer.Serialize(s);
        }

        private static void Execute(string db, string sql)
        {
            using var c = new SqliteConnection($"Data Source={db};Pooling=False");
            c.Open();
            using SqliteCommand command = c.CreateCommand();
            command.CommandText = sql;
            command.ExecuteNonQuery();
        }

        [Fact]
        public void A_new_file_is_stamped_with_the_schema_and_a_newer_one_is_refused()
        {
            MediaService.Open(Store, Settings()).Dispose();
            string db = Path.Combine(Store, MediaStore.FileName);
            using (var c = new SqliteConnection($"Data Source={db};Pooling=False"))
            {
                c.Open();
                using SqliteCommand command = c.CreateCommand();
                command.CommandText = "PRAGMA user_version";
                Assert.Equal((long)MediaDbSchema.Version, command.ExecuteScalar());
            }
            Execute(db, $"PRAGMA user_version = {MediaDbSchema.Version + 1}");
            Assert.Equal(MediaServiceException.NewerFile, Assert.Throws<MediaServiceException>(() => MediaService.Open(Store, Settings())).Status);
        }

        [Fact]
        public void A_game_s_text_and_media_are_found_by_the_path_last_hashed_to_it()
        {
            MediaService.Open(Store, Settings()).Dispose();
            string db = Path.Combine(Store, MediaStore.FileName);
            Execute(db, """
                INSERT INTO scrape_file (path, bytes, modified, md5) VALUES ('/r/a.sfc', 10, 99, 'aa'), ('/r/b.sfc', 20, 99, 'bb');
                INSERT INTO scrape_game (md5, bytes, status, name, description, rating, release_date, fetched_at) VALUES ('aa', 10, 'Found', 'A', 'About A', 0.800000011920929, '1991-08-23', '2026-09-26T12:00:00.000Z');
                INSERT INTO scrape_game (md5, bytes, status, fetched_at) VALUES ('bb', 20, 'Unknown', '2026-09-26T12:00:00.000Z');
                INSERT INTO scrape_media (md5, bytes, type, path, region, sha1, fetched_at) VALUES ('aa', 10, 'cover', 'snes/covers/a.png', 'us', 'ff', '2026-09-26T12:00:00.000Z');
                """);
            using NativeScraping store = NativeScraping.Open(Store, Settings(), null, null);
            Assert.Equal("About A", store.FoundByPath()["/r/a.sfc"].Description);
            Assert.Equal(new DateTime(1991, 8, 23), store.FoundByPath()["/r/a.sfc"].ReleaseDate);
            Assert.False(store.FoundByPath().ContainsKey("/r/b.sfc"));
            Assert.Equal((ScrapeState.Found, true), store.OutcomesByPath()["/r/a.sfc"]);
            Assert.Equal((ScrapeState.Unknown, false), store.OutcomesByPath()["/r/b.sfc"]);
            Assert.Equal("snes/covers/a.png", Assert.Single(store.Media("aa", 10)).RelativePath);
        }

        [Fact]
        public void A_closed_store_answers_nothing_and_writes_nothing()
        {
            var store = NativeScraping.Open(Store, Settings(), null, null);
            store.Dispose();
            Assert.Throws<ObjectDisposedException>(() => store.Queue());
            Assert.Throws<ObjectDisposedException>(() => store.Start([("/r/a.sfc", "snes")], ScrapePriority.Shown, true, false, false, null, true, new ScrapeChoices()));
        }

        // --- the order a picture is looked for in ---

        private string Touch(params string[] parts)
        {
            string path = Path.Combine([_root, .. parts]);
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllBytes(path, new byte[100]);
            return path;
        }

        private const string Rom = "/roms/F-Zero (USA).sfc";

        // A handle for the sources alone, so the order is asked of a store that holds nothing; OpenEmu's covers sit in the store's openemu folder.
        private MediaSources Sources(bool esde = true, bool openEmu = true)
        {
            var native = NativeScraping.Open(Store, Settings(new Dictionary<string, object?>
            {
                ["art"] = Path.Combine(_root, "Artwork"), ["esde_media"] = esde ? Path.Combine(_root, "Esde") : null, ["openemu"] = openEmu, ["sources_alone"] = true,
                ["cores"] = EmuSen.Mistress.Library.RuntimeCatalog.Consoles.Select(c => c.Systems).ToArray(),
            }), null, null);
            _open.Add(native);
            return new MediaSources(native, "");
        }

        private readonly List<NativeScraping> _open = new();

        [Fact]
        public void A_cover_is_the_player_s_then_screen_scraper_s_then_the_es_de_folder_s_then_open_emu_s()
        {
            try
            {
                Assert.Equal(MediaSource.None, Sources().Locate("snes", Rom, "cover").Source);
                string openEmu = Touch("Media", "openemu", "SNES", "F-Zero (USA).png");
                Assert.Equal((MediaSource.OpenEmu, openEmu), Sources().Locate("snes", Rom, "cover"));
                string esde = Touch("Esde", "snes", "covers", "F-Zero (USA).jpg");
                Assert.Equal((MediaSource.EsdeFolder, esde), Sources().Locate("snes", Rom, "cover"));
                string scraped = Touch("Media", "snes", "covers", "F-Zero (USA).png");
                Assert.Equal((MediaSource.ScreenScraper, scraped), Sources().Locate("snes", Rom, "cover"));
                string hand = Touch("Artwork", "SNES", "F-Zero (USA).png");
                Assert.Equal((MediaSource.HandPlaced, hand), Sources().Locate("snes", Rom, "cover"));
            }
            finally { _open.ForEach(n => n.Dispose()); }
        }

        [Fact]
        public void Open_emu_s_covers_are_not_shown_when_its_failover_is_off_and_an_es_de_folder_only_when_one_is_set()
        {
            try
            {
                Touch("Media", "openemu", "SNES", "F-Zero (USA).png");
                Assert.Equal(MediaSource.None, Sources(openEmu: false).Locate("snes", Rom, "cover").Source);
                Touch("Esde", "snes", "covers", "F-Zero (USA).png");
                Assert.Equal(MediaSource.None, Sources(esde: false, openEmu: false).Locate("snes", Rom, "cover").Source);
            }
            finally { _open.ForEach(n => n.Dispose()); }
        }

        [Fact]
        public void Other_media_are_screen_scraper_s_then_the_es_de_folder_s_and_never_the_player_s_cover_or_open_emu_s()
        {
            try
            {
                Touch("Artwork", "SNES", "F-Zero (USA).png");
                Touch("Media", "openemu", "SNES", "F-Zero (USA).png");
                Assert.Equal(MediaSource.None, Sources().Locate("snes", Rom, "screenshot").Source);
                string esde = Touch("Esde", "snes", "screenshots", "F-Zero (USA).png");
                Assert.Equal((MediaSource.EsdeFolder, esde), Sources().Locate("snes", Rom, "screenshot"));
                string scraped = Touch("Media", "snes", "screenshots", "F-Zero (USA).png");
                Assert.Equal((MediaSource.ScreenScraper, scraped), Sources().Locate("snes", Rom, "screenshot"));
                Assert.Equal(scraped, Sources().Find(new ThemeSystem("snes", "Super Nintendo", "snes"), new SceneGame("F-Zero (USA)", Rom), "screenshot"));
            }
            finally { _open.ForEach(n => n.Dispose()); }
        }
    }
}
