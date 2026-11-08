using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using EmuSen.Crystal.Client.Scrape;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;
using EmuSen.WiseMan.Fixtures;
using Microsoft.Data.Sqlite;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // media.db's second migration and the store's one layout step, taken by emusen-scrape when it opens the store: flat pictures of foldered games carried into their folders, nothing lost - see EmuSen_BigPicture.md §30 and EmuSen_Backend.md §40.5.
    [Collection(TestCollections.ProcessGlobals)]
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

        // A game as a scrape recorded it: its file's hash under each path, and one row per kind naming its flat file, written as the store keeps them.
        private void Scraped(string md5, IEnumerable<string> roms, params (string Type, string Relative)[] media)
        {
            if (!File.Exists(Db)) MediaService.Open(Media, Settings(null)).Dispose();
            const string at = "2026-09-27T12:00:00.000Z";
            var sql = new StringBuilder();
            foreach (string rom in roms) sql.Append($"INSERT OR REPLACE INTO scrape_file (path, bytes, modified, md5) VALUES ({Q(rom)}, 64, 1, {Q(md5)});");
            sql.Append($"INSERT OR REPLACE INTO scrape_game (md5, bytes, status, fetched_at) VALUES ({Q(md5)}, 64, 'Found', '{at}');");
            foreach ((string type, string relative) in media)
                sql.Append($"INSERT OR REPLACE INTO scrape_media (md5, bytes, type, path, region, sha1, fetched_at) VALUES ({Q(md5)}, 64, {Q(type)}, {Q(relative.Replace('/', Path.DirectorySeparatorChar))}, 'us', NULL, '{at}');");
            Execute(sql.ToString());
        }

        private static string Q(string text) => "'" + text.Replace("'", "''") + "'";

        private string Db => Path.Combine(Media, MediaStore.FileName);

        private void Execute(string sql)
        {
            using var c = new SqliteConnection($"Data Source={Db};Pooling=False");
            c.Open();
            using SqliteCommand command = c.CreateCommand();
            command.CommandText = sql;
            command.ExecuteNonQuery();
        }

        private List<string> Column(string sql)
        {
            using var c = new SqliteConnection($"Data Source={Db};Mode=ReadOnly;Pooling=False");
            c.Open();
            using SqliteCommand command = c.CreateCommand();
            command.CommandText = sql;
            using SqliteDataReader r = command.ExecuteReader();
            var rows = new List<string>();
            while (r.Read()) rows.Add(string.Join(" ", Enumerable.Range(0, r.FieldCount).Select(i => Convert.ToString(r.GetValue(i), System.Globalization.CultureInfo.InvariantCulture))));
            return rows;
        }

        private static string Settings(string? roms) => JsonSerializer.Serialize(new { roms, developer_files = false, openemu = false, test = new { offline = true, clock = NativeScraping.Ticks(Now) } });

        // The store opened by the library with the ROM folder named, which takes the layout step once.
        private NativeScraping Opened() => NativeScraping.Open(Media, Settings(Roms), null, null);

        private List<string> Moves() => Column("SELECT from_path, how, to_path FROM store_move ORDER BY from_path, to_path").Select(m => m.Replace(Path.DirectorySeparatorChar, '/')).ToList();

        private bool Done() => Column("SELECT name FROM store_step").Contains("folders");

        [Fact]
        public void Flat_pictures_of_foldered_games_go_into_their_folders_once_and_no_picture_is_lost()
        {
            Directory.CreateDirectory(Roms);
            Put("nes/covers/Alpha.png", 1);
            Put("nes/screenshots/Alpha.png", 2);
            Put("nes/covers/Bravo.png", 3);
            Put("nes/covers/Charlie.png", 4);
            Put("nes/covers/Delta.png", 5);
            Put("nes/covers/Echo.png", 6);
            string echoThere = Put("nes/covers/USA/Echo.png", 7);
            string stray = Put("nes/covers/Stray.png", 8);
            Scraped("a", [Rom("NES/USA/Alpha.nes")], ("cover", "nes/covers/Alpha.png"), ("screenshot", "nes/screenshots/Alpha.png"));
            Scraped("b", [Rom("NES/Bravo.nes")], ("cover", "nes/covers/Bravo.png"));
            Scraped("c", [Rom("NES/USA/Charlie.nes"), Rom("NES/Europe/Charlie.nes")], ("cover", "nes/covers/Charlie.png"));
            Scraped("d", [Rom("NES/Delta.nes"), Rom("NES/Hacks/Delta.nes")], ("cover", "nes/covers/Delta.png"));
            Scraped("e", [Rom("NES/USA/Echo.nes")], ("cover", "nes/covers/Echo.png"));
            Dictionary<string, string> before = Fingerprint();

            using NativeScraping store = Opened();

            Dictionary<string, string> after = Fingerprint();
            // No picture's bytes are gone: every file there before is there after, by content.
            Assert.Empty(before.Values.Except(after.Values));
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

            Assert.Equal(
            [
                "nes/covers/Alpha.png moved nes/covers/USA/Alpha.png", "nes/covers/Charlie.png copied nes/covers/Europe/Charlie.png",
                "nes/covers/Charlie.png moved nes/covers/USA/Charlie.png", "nes/covers/Delta.png copied nes/covers/Hacks/Delta.png",
                "nes/covers/Echo.png found nes/covers/USA/Echo.png", "nes/screenshots/Alpha.png moved nes/screenshots/USA/Alpha.png",
            ], Moves());
            Assert.True(Done());

            // Once: a reopened store changes nothing, even with a new flat picture of a foldered game.
            store.Dispose();
            Put("nes/covers/Foxtrot.png", 9);
            Scraped("f", [Rom("NES/USA/Foxtrot.nes")], ("cover", "nes/covers/Foxtrot.png"));
            Dictionary<string, string> settled = Fingerprint();
            using NativeScraping again = Opened();
            Assert.Equal(settled, Fingerprint());
            Assert.Equal("nes/covers/Foxtrot.png", again.Media("f", 64).Single(m => m.Type == "cover").RelativePath.Replace(Path.DirectorySeparatorChar, '/'));
        }

        [Fact]
        public void A_file_of_the_first_schema_is_migrated_in_place_with_its_rows_kept_and_a_step_interrupted_after_a_move_is_finished()
        {
            Directory.CreateDirectory(Roms);
            Put("snes/covers/Golf.png", 10);
            Scraped("g", [Rom("SNES/Sports/Golf.sfc")], ("cover", "snes/covers/Golf.png"));
            Execute("DROP TABLE store_move; DROP TABLE store_step; ALTER TABLE scrape_game DROP COLUMN kinds_known; ALTER TABLE scrape_file DROP COLUMN crc32; ALTER TABLE scrape_file DROP COLUMN sha1; ALTER TABLE scrape_file DROP COLUMN fingerprint; PRAGMA user_version = 1;");
            // The picture already moved and the row not yet written, as a crash between the two would leave it.
            Directory.CreateDirectory(Path.Combine(Media, "snes", "covers", "Sports"));
            File.Move(Path.Combine(Media, "snes", "covers", "Golf.png"), Path.Combine(Media, "snes", "covers", "Sports", "Golf.png"));

            using NativeScraping reopened = Opened();
            Assert.Equal(MediaDbSchema.Version.ToString(System.Globalization.CultureInfo.InvariantCulture), Column("PRAGMA user_version").Single());
            Assert.Equal(Path.Combine("snes", "covers", "Sports", "Golf.png"), reopened.Media("g", 64).Single().RelativePath);
            Assert.Equal("found", Moves().Single().Split(' ')[1]);
            Assert.True(Done());
        }

        // Clear on a foldered game: its folder's pictures go; a game of the same stem in another folder, or at the top, keeps its own.
        [Fact]
        public void Forgetting_a_foldered_game_deletes_only_the_pictures_in_its_own_folder()
        {
            string mine = Put("nes/covers/USA/Kid.png", 11), theirs = Put("nes/covers/Europe/Kid.png", 12), top = Put("nes/covers/Kid.png", 13);
            using NativeScraping store = NativeScraping.Open(Media, Settings(null), null, null);
            IReadOnlyList<string> gone = store.Forget(Rom("NES/USA/Kid.nes"), "nes", "USA");
            Assert.Equal([Path.GetFullPath(mine)], gone);
            Assert.True(File.Exists(theirs));
            Assert.True(File.Exists(top));
        }

        [Theory]
        [InlineData("USA", "USA", "")]
        [InlineData("USA2", "USA", null)]
        [InlineData("USA2/Sub", "USA", null)]
        [InlineData("Hacks/Mario", "Hacks", "Mario")]
        [InlineData("Hacks/Mario/Deep", "", "Hacks")]
        [InlineData("", "", "")]
        [InlineData("", "USA", null)]
        public void A_folder_s_child_is_found_only_below_it(string gameFolder, string folder, string? child) =>
            Assert.Equal(child, GameFolders.ChildWithin(gameFolder, folder));

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
