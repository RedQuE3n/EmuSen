using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;
using Microsoft.Data.Sqlite;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // media.db's second migration and the store's one layout step: flat pictures of foldered games carried into their folders, nothing lost - see EmuSen_BigPicture.md §30.
    public class MediaStoreFoldersTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenMediaStoreFolders", Guid.NewGuid().ToString("N"));
        private static readonly DateTimeOffset Now = new(2026, 9, 27, 12, 0, 0, TimeSpan.Zero);

        private string Media => Path.Combine(_root, "Media");
        private string Roms => Path.Combine(_root, "Roms");

        public void Dispose()
        {
            try { Directory.Delete(_root, recursive: true); } catch { }
        }

        private string Rom(string relative) => Path.Combine(Roms, relative.Replace('/', Path.DirectorySeparatorChar));

        private string Put(string relative, int seed)
        {
            string path = Path.Combine(Media, relative.Replace('/', Path.DirectorySeparatorChar));
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            var bytes = new byte[256 + seed];
            new Random(seed).NextBytes(bytes);
            File.WriteAllBytes(path, bytes);
            return path;
        }

        // Every file below the store but media.db, by its path relative to the store, with its SHA-256.
        private Dictionary<string, string> Fingerprint() => Directory.EnumerateFiles(Media, "*", SearchOption.AllDirectories)
            .Where(f => !Path.GetFileName(f).StartsWith(MediaStore.FileName, StringComparison.Ordinal))
            .ToDictionary(f => Path.GetRelativePath(Media, f).Replace(Path.DirectorySeparatorChar, '/'), f => Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(f))));

        // A game as a scrape recorded it: its file's hash under each path, and one row per kind naming its flat file.
        private static void Scraped(MediaStore store, string md5, IEnumerable<string> roms, params (string Type, string Relative)[] media)
        {
            foreach (string rom in roms) store.RememberFile(rom, 64, 1, md5);
            store.Record(new ScrapedRecord(md5, 64, ScrapeState.Found) { FetchedAt = Now.UtcDateTime });
            foreach ((string type, string relative) in media) store.RecordMedia(md5, 64, new StoredMedia(type, relative.Replace('/', Path.DirectorySeparatorChar), "us", null), Now);
        }

        [Fact]
        public void Flat_pictures_of_foldered_games_go_into_their_folders_once_and_no_picture_is_lost()
        {
            Directory.CreateDirectory(Roms);
            using MediaStore store = MediaStore.Open(Media);
            Put("nes/covers/Alpha.png", 1);
            Put("nes/screenshots/Alpha.png", 2);
            Put("nes/covers/Bravo.png", 3);
            Put("nes/covers/Charlie.png", 4);
            Put("nes/covers/Delta.png", 5);
            Put("nes/covers/Echo.png", 6);
            string echoThere = Put("nes/covers/USA/Echo.png", 7);
            string stray = Put("nes/covers/Stray.png", 8);
            Scraped(store, "a", [Rom("NES/USA/Alpha.nes")], ("cover", "nes/covers/Alpha.png"), ("screenshot", "nes/screenshots/Alpha.png"));
            Scraped(store, "b", [Rom("NES/Bravo.nes")], ("cover", "nes/covers/Bravo.png"));
            Scraped(store, "c", [Rom("NES/USA/Charlie.nes"), Rom("NES/Europe/Charlie.nes")], ("cover", "nes/covers/Charlie.png"));
            Scraped(store, "d", [Rom("NES/Delta.nes"), Rom("NES/Hacks/Delta.nes")], ("cover", "nes/covers/Delta.png"));
            Scraped(store, "e", [Rom("NES/USA/Echo.nes")], ("cover", "nes/covers/Echo.png"));
            Dictionary<string, string> before = Fingerprint();

            int carried = store.PutInFolders(p => GameFolders.Of(Roms, p), Now);

            Dictionary<string, string> after = Fingerprint();
            // No picture's bytes are gone: every file there before is there after, by content.
            Assert.Empty(before.Values.Except(after.Values));
            Assert.Equal(5, carried);
            Assert.Equal(new Dictionary<string, string>
            {
                ["nes/covers/USA/Alpha.png"] = before["nes/covers/Alpha.png"],
                ["nes/screenshots/USA/Alpha.png"] = before["nes/screenshots/Alpha.png"],
                ["nes/covers/Bravo.png"] = before["nes/covers/Bravo.png"],
                ["nes/covers/Europe/Charlie.png"] = before["nes/covers/Charlie.png"],
                ["nes/covers/USA/Charlie.png"] = before["nes/covers/Charlie.png"],
                ["nes/covers/Delta.png"] = before["nes/covers/Delta.png"],
                ["nes/covers/Hacks/Delta.png"] = before["nes/covers/Delta.png"],
                ["nes/covers/Echo.png"] = before["nes/covers/Echo.png"],
                ["nes/covers/USA/Echo.png"] = before["nes/covers/USA/Echo.png"],
                ["nes/covers/Stray.png"] = before["nes/covers/Stray.png"],
            }, after);
            Assert.True(File.Exists(stray));
            Assert.Equal(before["nes/covers/USA/Echo.png"], Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(echoThere))));

            // Each row names a file that exists: its own folder's, or the flat one a game at the top still shares.
            string Row(string md5, string type) => store.Media(md5, 64).Single(m => m.Type == type).RelativePath.Replace(Path.DirectorySeparatorChar, '/');
            Assert.Equal(("nes/covers/USA/Alpha.png", "nes/screenshots/USA/Alpha.png"), (Row("a", "cover"), Row("a", "screenshot")));
            Assert.Equal("nes/covers/Bravo.png", Row("b", "cover"));
            Assert.Equal("nes/covers/Europe/Charlie.png", Row("c", "cover"));
            Assert.Equal("nes/covers/Delta.png", Row("d", "cover"));
            Assert.Equal("nes/covers/USA/Echo.png", Row("e", "cover"));

            string Log((string From, string To, string How) m) => $"{m.From.Replace(Path.DirectorySeparatorChar, '/')} {m.How} {m.To.Replace(Path.DirectorySeparatorChar, '/')}";
            Assert.Equal(
            [
                "nes/covers/Alpha.png moved nes/covers/USA/Alpha.png", "nes/covers/Charlie.png copied nes/covers/Europe/Charlie.png",
                "nes/covers/Charlie.png moved nes/covers/USA/Charlie.png", "nes/covers/Delta.png copied nes/covers/Hacks/Delta.png",
                "nes/covers/Echo.png found nes/covers/USA/Echo.png", "nes/screenshots/Alpha.png moved nes/screenshots/USA/Alpha.png",
            ], store.Moves().Select(Log));
            Assert.True(store.Done(MediaStore.FoldersStep));

            // Once: a second call, and a reopened file, change nothing, even with a new flat picture of a foldered game.
            Put("nes/covers/Foxtrot.png", 9);
            Scraped(store, "f", [Rom("NES/USA/Foxtrot.nes")], ("cover", "nes/covers/Foxtrot.png"));
            Dictionary<string, string> settled = Fingerprint();
            Assert.Equal(0, store.PutInFolders(p => GameFolders.Of(Roms, p), Now));
            using (MediaStore again = MediaStore.Open(Media)) Assert.Equal(0, again.PutInFolders(p => GameFolders.Of(Roms, p), Now));
            Assert.Equal(settled, Fingerprint());
            Assert.Equal("nes/covers/Foxtrot.png", Row("f", "cover"));
        }

        [Fact]
        public void A_file_of_the_first_schema_is_migrated_in_place_with_its_rows_kept_and_a_step_interrupted_after_a_move_is_finished()
        {
            Directory.CreateDirectory(Roms);
            using (MediaStore store = MediaStore.Open(Media))
            {
                Put("snes/covers/Golf.png", 10);
                Scraped(store, "g", [Rom("SNES/Sports/Golf.sfc")], ("cover", "snes/covers/Golf.png"));
            }
            using (var db = new SqliteConnection($"Data Source={Path.Combine(Media, MediaStore.FileName)};Pooling=False"))
            {
                db.Open();
                using SqliteCommand c = db.CreateCommand();
                c.CommandText = "DROP TABLE store_move; DROP TABLE store_step; PRAGMA user_version = 1;";
                c.ExecuteNonQuery();
            }
            // The picture already moved and the row not yet written, as a crash between the two would leave it.
            Directory.CreateDirectory(Path.Combine(Media, "snes", "covers", "Sports"));
            File.Move(Path.Combine(Media, "snes", "covers", "Golf.png"), Path.Combine(Media, "snes", "covers", "Sports", "Golf.png"));

            using MediaStore reopened = MediaStore.Open(Media);
            using (var db = new SqliteConnection($"Data Source={Path.Combine(Media, MediaStore.FileName)};Pooling=False"))
            {
                db.Open();
                using SqliteCommand c = db.CreateCommand();
                c.CommandText = "PRAGMA user_version";
                Assert.Equal(2L, c.ExecuteScalar());
            }
            Assert.False(reopened.Done(MediaStore.FoldersStep));
            Assert.Single(reopened.Media("g", 64));
            Assert.Equal(0, reopened.PutInFolders(p => GameFolders.Of(Roms, p), Now));
            Assert.Equal(Path.Combine("snes", "covers", "Sports", "Golf.png"), reopened.Media("g", 64).Single().RelativePath);
            Assert.Equal("found", reopened.Moves().Single().How);
            Assert.True(reopened.Done(MediaStore.FoldersStep));
        }

        [Theory]
        [InlineData("NES/USA/Game.nes", "USA")]
        [InlineData("NES/Hacks/Mario/Game.nes", "Hacks/Mario")]
        [InlineData("NES/Game.nes", "")]
        [InlineData("Game.nes", "")]
        public void A_game_s_folder_is_what_lies_between_its_console_folder_and_its_file(string relative, string folder)
        {
            Assert.Equal(folder, GameFolders.Of(Roms, Rom(relative)));
            Assert.Equal("", GameFolders.Of(Roms, Path.Combine(_root, "Elsewhere", "NES", "USA", "Game.nes")));
            Assert.Equal("", GameFolders.Of(null, Rom(relative)));
        }
    }
}
