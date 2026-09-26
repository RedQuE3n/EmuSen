using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.Mistress.Scraping;
using Microsoft.Data.Sqlite;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // media.db's schema, queue and day, and the order a picture is looked for in - see EmuSen_BigPicture.md §5.6 and §17.
    public class MediaStoreTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenMediaStoreTests", Guid.NewGuid().ToString("N"));
        private static readonly DateTimeOffset Now = new(2026, 9, 26, 12, 0, 0, TimeSpan.Zero);

        public void Dispose()
        {
            try { Directory.Delete(_root, recursive: true); } catch { }
        }

        [Fact]
        public void A_new_file_is_stamped_with_the_schema_and_a_newer_one_is_refused()
        {
            using (MediaStore.Open(_root)) { }
            using (var db = new SqliteConnection($"Data Source={Path.Combine(_root, MediaStore.FileName)};Pooling=False"))
            {
                db.Open();
                using SqliteCommand c = db.CreateCommand();
                c.CommandText = "PRAGMA user_version";
                Assert.Equal((long)MediaStore.SchemaVersion, c.ExecuteScalar());
                c.CommandText = $"PRAGMA user_version = {MediaStore.SchemaVersion + 1}";
                c.ExecuteNonQuery();
            }
            Assert.Throws<InvalidDataException>(() => MediaStore.Open(_root));
        }

        [Fact]
        public void The_queue_goes_by_priority_then_arrival_and_a_game_shown_moves_up()
        {
            using MediaStore store = MediaStore.Open(_root);
            store.Enqueue("/r/a.sfc", "snes", ScrapePriority.Library);
            store.Enqueue("/r/b.sfc", "snes", ScrapePriority.Library);
            store.Enqueue("/r/c.sfc", "snes", ScrapePriority.Console);
            store.Enqueue("/r/b.sfc", "snes", ScrapePriority.Shown);
            store.Enqueue("/r/c.sfc", "snes", ScrapePriority.Library);

            Assert.Equal(["/r/b.sfc", "/r/c.sfc", "/r/a.sfc"], store.Queue().Select(q => q.Path));
            Assert.Equal("/r/c.sfc", store.NextDue(Now, new HashSet<string> { "/r/b.sfc" })!.Path);
        }

        [Fact]
        public void A_game_waiting_to_be_retried_is_not_due_until_its_time_and_the_queue_outlives_the_file_being_closed()
        {
            using (MediaStore store = MediaStore.Open(_root))
            {
                store.Enqueue("/r/a.sfc", "snes", ScrapePriority.Shown);
                store.Enqueue("/r/b.sfc", "nes", ScrapePriority.Library);
                store.Retry("/r/a.sfc", Now.AddMinutes(5), countAttempt: true);
            }

            using MediaStore reopened = MediaStore.Open(_root);
            Assert.Equal("/r/b.sfc", reopened.NextDue(Now, new HashSet<string>())!.Path);
            QueuedGame later = reopened.NextDue(Now.AddMinutes(5), new HashSet<string> { "/r/b.sfc" })!;
            Assert.Equal(("/r/a.sfc", 1), (later.Path, later.Attempts));
            reopened.Dequeue("/r/b.sfc");
            Assert.Equal(1, reopened.QueueLength);
        }

        [Fact]
        public void A_game_s_text_and_media_are_found_by_the_path_last_hashed_to_it()
        {
            using MediaStore store = MediaStore.Open(_root);
            store.RememberFile("/r/a.sfc", 10, 99, "aa");
            store.RememberFile("/r/b.sfc", 20, 99, "bb");
            store.Record(new ScrapedRecord("aa", 10, ScrapeState.Found) { Name = "A", Description = "About A", Rating = 0.8f, ReleaseDate = new DateTime(1991, 8, 23), FetchedAt = Now.UtcDateTime });
            store.Record(new ScrapedRecord("bb", 20, ScrapeState.Unknown) { FetchedAt = Now.UtcDateTime });
            store.RecordMedia("aa", 10, new StoredMedia("cover", "snes/covers/a.png", "us", "ff"), Now);

            Assert.Equal("About A", store.FoundByPath()["/r/a.sfc"].Description);
            Assert.Equal(new DateTime(1991, 8, 23), store.FoundByPath()["/r/a.sfc"].ReleaseDate);
            Assert.False(store.FoundByPath().ContainsKey("/r/b.sfc"));
            Assert.Equal((ScrapeState.Found, true), store.OutcomesByPath()["/r/a.sfc"]);
            Assert.Equal((ScrapeState.Unknown, false), store.OutcomesByPath()["/r/b.sfc"]);
            Assert.Equal("aa", store.CachedMd5("/r/a.sfc", 10, 99));
            Assert.Null(store.CachedMd5("/r/a.sfc", 10, 100));
        }

        [Fact]
        public void A_closed_store_answers_nothing_and_writes_nothing()
        {
            MediaStore store = MediaStore.Open(_root);
            store.Dispose();
            Assert.False(store.IsOpen);
            Assert.False(store.Enqueue("/r/a.sfc", "snes", ScrapePriority.Shown));
            Assert.Null(store.NextDue(Now, new HashSet<string>()));
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

        private MediaSources Sources(string? hand, bool esde = true, bool openEmu = true) => new(
            (console, title) => console == "SNES" && title == "F-Zero (USA)" ? hand : null,
            Path.Combine(_root, "Media"), esde ? Path.Combine(_root, "Esde") : null, openEmu ? Path.Combine(_root, "OpenEmu") : null);

        [Fact]
        public void A_cover_is_the_player_s_then_screen_scraper_s_then_the_es_de_folder_s_then_open_emu_s()
        {
            Assert.Equal(MediaSource.None, Sources(null).Locate("snes", Rom, "cover").Source);
            string openEmu = Touch("OpenEmu", "SNES", "F-Zero (USA).png");
            Assert.Equal((MediaSource.OpenEmu, openEmu), Sources(null).Locate("snes", Rom, "cover"));
            string esde = Touch("Esde", "snes", "covers", "F-Zero (USA).jpg");
            Assert.Equal((MediaSource.EsdeFolder, esde), Sources(null).Locate("snes", Rom, "cover"));
            string scraped = Touch("Media", "snes", "covers", "F-Zero (USA).png");
            Assert.Equal((MediaSource.ScreenScraper, scraped), Sources(null).Locate("snes", Rom, "cover"));
            string hand = Touch("Artwork", "SNES", "F-Zero (USA).png");
            Assert.Equal((MediaSource.HandPlaced, hand), Sources(hand).Locate("snes", Rom, "cover"));
        }

        [Fact]
        public void Open_emu_s_covers_are_not_shown_when_its_failover_is_off_and_an_es_de_folder_only_when_one_is_set()
        {
            Touch("OpenEmu", "SNES", "F-Zero (USA).png");
            Assert.Equal(MediaSource.None, Sources(null, openEmu: false).Locate("snes", Rom, "cover").Source);
            Touch("Esde", "snes", "covers", "F-Zero (USA).png");
            Assert.Equal(MediaSource.None, Sources(null, esde: false, openEmu: false).Locate("snes", Rom, "cover").Source);
        }

        [Fact]
        public void Other_media_are_screen_scraper_s_then_the_es_de_folder_s_and_never_the_player_s_cover_or_open_emu_s()
        {
            string hand = Touch("Artwork", "SNES", "F-Zero (USA).png");
            Touch("OpenEmu", "SNES", "F-Zero (USA).png");
            Assert.Equal(MediaSource.None, Sources(hand).Locate("snes", Rom, "screenshot").Source);
            string esde = Touch("Esde", "snes", "screenshots", "F-Zero (USA).png");
            Assert.Equal((MediaSource.EsdeFolder, esde), Sources(hand).Locate("snes", Rom, "screenshot"));
            string scraped = Touch("Media", "snes", "screenshots", "F-Zero (USA).png");
            Assert.Equal((MediaSource.ScreenScraper, scraped), Sources(hand).Locate("snes", Rom, "screenshot"));
            Assert.Equal(scraped, Sources(hand).Find(new ThemeSystem("snes", "Super Nintendo", "snes"), new SceneGame("F-Zero (USA)", Rom), "screenshot"));
        }
    }
}
