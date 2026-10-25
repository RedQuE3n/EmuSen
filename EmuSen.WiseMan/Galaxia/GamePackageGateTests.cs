using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Media;
using EmuSen.Galaxia.Models;
using EmuSen.Galaxia.Native;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Galaxia
{
    // Step 5's gate through the platform library, as a frontend reaches it: pkg make and pkg migrate leave the library's every file as it was, a claimed file opens as its package, a moved one is found again by its hashes, and pkg check catches a changed file and a missing one - see EmuSen_GamePackages.md §7 and §14.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class GamePackageGateTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenPackageGate_" + Guid.NewGuid().ToString("N"));
        private readonly string _library;

        public GamePackageGateTests()
        {
            Assert.True(LibraryToolJob.Available, LibraryToolJob.Report);
            _library = Path.Combine(_root, "Roms");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (Exception e) when (e is IOException or UnauthorizedAccessException) { }
        }

        private static void Put(string path, byte[] bytes)
        {
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllBytes(path, bytes);
        }

        // Every file of the library: its path, size, modification time and MD5.
        private string Listing() => string.Join("\n", Directory.EnumerateFiles(_library, "*", SearchOption.AllDirectories).Order(StringComparer.Ordinal)
            .Select(f => $"{f} {new FileInfo(f).Length} {File.GetLastWriteTimeUtc(f).Ticks} {RomHash.Managed.Md5(f)}"));

        private static List<JsonElement> Run(string request)
        {
            var events = new List<JsonElement>();
            using IToolJob job = LibraryToolJob.Start(request);
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (!job.Ended)
            {
                if (job.Next(200) is { } line) events.Add(JsonDocument.Parse(line).RootElement.Clone());
                Assert.True(clock.Elapsed < TimeSpan.FromMinutes(2), "the job did not end");
            }
            return events;
        }

        private static string State(List<JsonElement> events, string named) =>
            events.Where(e => e.GetProperty("event").GetString() == "package")
                .FirstOrDefault(e => (e.TryGetProperty("source", out var s) && s.GetString() == named) || (e.TryGetProperty("package", out var p) && p.GetString() == named))
                is { ValueKind: JsonValueKind.Object } found ? found.GetProperty("state").GetString()! : "absent";

        private static string Shown(List<JsonElement> events) => string.Join("\n", events.Select(e => e.GetRawText()));

        [Fact]
        public void Making_and_migrating_packages_leaves_the_library_as_it_was_and_check_catches_a_changed_and_a_missing_file()
        {
            string snes = Path.Combine(_library, "snes", "Cobalt Harbor (Synthetic).sfc"), nes = Path.Combine(_library, "nes", "USA", "Granite Choir (Synthetic).nes");
            Put(snes, SyntheticRom.BuildBlank());
            Put(nes, SyntheticNesRom.Build(prgBanks: 2, chrBanks: 1, mapper: 1, battery: true));
            Directory.CreateDirectory(Path.Combine(_library, "psx"));
            SyntheticPsxDisc.Build(audioSeconds: 1).WriteSet(Path.Combine(_library, "psx"), "Ruby Lantern (Synthetic)");
            string states = Path.Combine(_library, "States");
            Put(Path.Combine(states, "Cobalt Harbor (Synthetic).state"), new byte[] { 1, 2, 3 });
            Put(Path.Combine(states, "Cobalt Harbor (Synthetic).resume.state"), new byte[] { 4 });
            Put(Path.Combine(states, "Cobalt Harbor (Synthetic).png"), new byte[] { 5 });
            new AppSettings { RomDirectory = _library, StateDirectory = states }.Save();
            Put(Path.Combine(DataStore.Saves, "SNES", "Cobalt Harbor (Synthetic).srm"), new byte[] { 9, 9 });
            Put(Path.Combine(ConfigStore.Directory, "cheats", "SNES", "Cobalt Harbor (Synthetic).json"), "{\"cheats\":[]}"u8.ToArray());
            string before = Listing();

            List<JsonElement> made = Run("{\"op\":\"pkg-make\",\"paths\":[\"/\"]}");
            foreach (string source in new[] { "snes/Cobalt Harbor (Synthetic).sfc", "nes/USA/Granite Choir (Synthetic).nes", "psx/Ruby Lantern (Synthetic).cue" })
                Assert.True(State(made, source) == "made", $"{source}\n{Shown(made)}");
            List<JsonElement> migrated = Run("{\"op\":\"pkg-migrate\",\"paths\":[\"/\"]}");
            string cobalt = Path.Combine(DataStore.Packages, "SNES", "Cobalt Harbor (Synthetic).sfc");
            Assert.True(State(migrated, "SNES/Cobalt Harbor (Synthetic).sfc") == "migrated", Shown(migrated));
            Assert.Equal(new byte[] { 9, 9 }, File.ReadAllBytes(Path.Combine(cobalt, "saves", "Cobalt Harbor (Synthetic).srm")));
            Assert.Equal(new byte[] { 1, 2, 3 }, File.ReadAllBytes(Path.Combine(cobalt, "states", "Cobalt Harbor (Synthetic).state")));
            Assert.True(File.Exists(Path.Combine(cobalt, "states", "Cobalt Harbor (Synthetic).resume.state")) && File.Exists(Path.Combine(cobalt, "states", "Cobalt Harbor (Synthetic).png")));
            Assert.True(File.Exists(Path.Combine(cobalt, "cheats.json")));
            Assert.Equal(before, Listing());

            // One game, one save: the plain path and the package name the same package, and the save, states and cheats it keeps.
            var places = PackagePlaces.Current();
            Assert.Equal(cobalt, GamePackages.Resolve(snes, places).Package);
            Assert.Equal(cobalt, GamePackages.Resolve(cobalt, places).Package);
            Assert.Equal(snes, GamePackages.Resolve(cobalt, places).Game);
            Assert.Equal(SaveLibrary.SramPathFor(snes, "SNES", GamePackages.RootFor(snes)), SaveLibrary.SramPathFor(GamePackages.Resolve(cobalt, places).Game, "SNES", cobalt));
            Assert.Equal(Path.Combine(cobalt, "cheats.json"), GameCheats.For("SNES", snes, GamePackages.RootFor(snes))!.PackageFile);
            Assert.Null(GamePackages.RootFor(Path.Combine(_library, "nowhere.sfc")));

            List<JsonElement> checkedAll = Run("{\"op\":\"pkg-check\",\"paths\":[]}");
            Assert.True(checkedAll.Count(e => e.GetProperty("event").GetString() == "package" && e.GetProperty("state").GetString() == "verified") == 3, Shown(checkedAll));

            // Moved: ls names where it is, check rewrites the manifest, and the plain path at its new place opens as the package.
            string moved = Path.Combine(_library, "nes", "Europe", "Granite Choir (Synthetic) (E).nes");
            Directory.CreateDirectory(Path.GetDirectoryName(moved)!);
            File.Move(nes, moved);
            List<JsonElement> listed = Run("{\"op\":\"pkg-ls\",\"paths\":[],\"console\":\"nes\"}");
            Assert.True(State(listed, "NES/USA/Granite Choir (Synthetic).nes") == "moved", Shown(listed));
            List<JsonElement> found = Run("{\"op\":\"pkg-check\",\"paths\":[\"NES\"]}");
            Assert.True(State(found, "NES/USA/Granite Choir (Synthetic).nes") == "moved", Shown(found));
            Assert.Contains("nes/Europe/Granite Choir (Synthetic) (E).nes", File.ReadAllText(Path.Combine(DataStore.Packages, "NES", "USA", "Granite Choir (Synthetic).nes", "package.json")));
            Assert.Equal(Path.Combine(DataStore.Packages, "NES", "USA", "Granite Choir (Synthetic).nes"), GamePackages.RootFor(moved));

            // Changed and missing.
            byte[] changed = File.ReadAllBytes(snes);
            changed[0x200] ^= 0xFF;
            File.WriteAllBytes(snes, changed);
            File.Delete(Directory.EnumerateFiles(Path.Combine(_library, "psx"), "*.bin").Order(StringComparer.Ordinal).Last());
            List<JsonElement> caught = Run("{\"op\":\"pkg-check\",\"paths\":[]}");
            Assert.True(State(caught, "SNES/Cobalt Harbor (Synthetic).sfc") == "changed", Shown(caught));
            Assert.True(State(caught, "PlayStation/Ruby Lantern (Synthetic).cue") == "missing", Shown(caught));
            Assert.True(State(caught, "NES/USA/Granite Choir (Synthetic).nes") == "verified", Shown(caught));
        }

        private static string Hash(string file) => Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(File.ReadAllBytes(file)));

        // The stores step 5 left (§18): pkg migrate copies a state's record, the game's own settings with its crop, and its art into the package, leaves every original byte for byte as it was, and the readers find them by either path and after the package moves.
        [Fact]
        public void Migrating_copies_the_state_records_settings_crop_and_art_in_and_leaves_their_originals_as_they_were()
        {
            string snes = Path.Combine(_library, "snes", "Amber Quay (Synthetic).sfc");
            Put(snes, SyntheticRom.BuildBlank());
            string states = Path.Combine(_library, "States"), state = Path.Combine(states, "Amber Quay (Synthetic).slot3.state");
            Put(state, new byte[] { 7, 7, 7 });
            new AppSettings { RomDirectory = _library, StateDirectory = states }.Save();
            // states.db's row for the state, as the runtime keeps it.
            string statesDb = Path.Combine(DataStore.Library, "states.db");
            using (var records = EmuSen.Crystal.Managed.StateRecords.Open(statesDb))
                records.Write(state, new StateRecord { Console = "SNES", Core = "VenusRT (Rust)", StateVersion = 4, Build = "an old build", SavedAt = new DateTime(2026, 9, 1, 8, 0, 0, DateTimeKind.Utc), RomFile = "Amber Quay (Synthetic).sfc", RomMd5 = "abc", RomBytes = 9 });
            Microsoft.Data.Sqlite.SqliteConnection.ClearAllPools();
            // games.db's crop, the runtime's file of the game's own settings, and the media store's art.
            string gamesDb = Path.Combine(DataStore.Library, "games.db");
            using (var games = EmuSen.Mistress.Library.GameRecords.OpenManaged(gamesDb))
            {
                games.OpenedAs = p => (p, null);
                games.SetCrop(snes, new EmuSen.Mistress.Library.GameCrop(2.5, 0, 1, 12));
            }
            Microsoft.Data.Sqlite.SqliteConnection.ClearAllPools();
            string plain = GameSettings.PlainPath("snes", snes);
            Put(plain, "{\n  \"Engine\": \"VenusRT (Rust)\"\n}"u8.ToArray());
            string cover = Path.Combine(DataStore.Media, "snes", "covers", "Amber Quay (Synthetic).png"), video = Path.Combine(DataStore.Media, "snes", "videos", "Amber Quay (Synthetic).mp4");
            Put(cover, new byte[] { 0x89, 1, 2 });
            Put(video, new byte[] { 3, 4 });
            string[] originals = { state, statesDb, gamesDb, plain, cover, video };
            var before = originals.Select(Hash).ToArray();
            string listing = Listing();

            Assert.True(State(Run("{\"op\":\"pkg-make\",\"paths\":[\"snes\"]}"), "snes/Amber Quay (Synthetic).sfc") == "made");
            string package = Path.Combine(DataStore.Packages, "SNES", "Amber Quay (Synthetic).sfc");
            List<JsonElement> migrated = Run("{\"op\":\"pkg-migrate\",\"paths\":[\"SNES\"]}");
            Assert.True(State(migrated, "SNES/Amber Quay (Synthetic).sfc") == "migrated", Shown(migrated));
            JsonElement outcome = migrated.Single(e => e.GetProperty("event").GetString() == "package");
            Assert.True(outcome.GetProperty("records").GetInt64() == 1, Shown(migrated));

            string copied = Path.Combine(package, "states", "Amber Quay (Synthetic).slot3.state");
            StateRecord? record = EmuSen.Crystal.Managed.StateRecords.ReadPackaged(Path.Combine(package, "states", "Amber Quay (Synthetic).slot3.json"), copied);
            Assert.NotNull(record);
            Assert.Equal(("VenusRT (Rust)", 4, "an old build", "abc", 9L), (record!.Core, record.StateVersion, record.Build, record.RomMd5, record.RomBytes));
            string settings = File.ReadAllText(Path.Combine(package, "settings.json"));
            Assert.Contains("\"Engine\": \"VenusRT (Rust)\"", settings);
            Assert.Contains("\"Crop\": \"2.5 0 1 12\"", settings);
            Assert.Equal(new byte[] { 0x89, 1, 2 }, File.ReadAllBytes(Path.Combine(package, "media", "covers", "Amber Quay (Synthetic).png")));
            Assert.Equal(new byte[] { 3, 4 }, File.ReadAllBytes(Path.Combine(package, "media", "videos", "Amber Quay (Synthetic).mp4")));
            Assert.Equal(before, originals.Select(Hash).ToArray());
            Assert.Equal(listing, Listing());

            // The readers, by the plain path and by the package: the record, the crop and the engine, and the art.
            using (var states2 = EmuSen.Crystal.Managed.StateRecords.Open(statesDb))
                Assert.Equal(record, states2.Read(copied));
            Microsoft.Data.Sqlite.SqliteConnection.ClearAllPools();
            using (var games = EmuSen.Mistress.Library.GameRecords.OpenManaged(gamesDb))
            {
                games.SystemOf = _ => "snes";
                Assert.Equal(new EmuSen.Mistress.Library.GameCrop(2.5, 0, 1, 12), games.Crop(snes));
                Assert.Equal(new EmuSen.Mistress.Library.GameCrop(2.5, 0, 1, 12), games.Crop(package));
            }
            Microsoft.Data.Sqlite.SqliteConnection.ClearAllPools();
            Assert.Equal("VenusRT (Rust)", EmuSen.Crystal.Managed.ManagedRuntime.GameSetting(snes, "Engine"));
            Assert.Equal("VenusRT (Rust)", EmuSen.Crystal.Managed.ManagedRuntime.GameSetting(package, "Engine"));
            string Cover(string where) => new EmuSen.Mistress.Scraping.MediaSources((_, _) => null, DataStore.Media, null, null, romDirectory: _library,
                packages: new Dictionary<string, string> { [snes] = where }).Locate("snes", snes, "cover").Path!;
            Assert.Equal(Path.Combine(package, "media", "covers", "Amber Quay (Synthetic).png"), Cover(package));

            // Migrating again copies nothing more and changes nothing.
            List<JsonElement> again = Run("{\"op\":\"pkg-migrate\",\"paths\":[\"SNES\"]}");
            Assert.True(State(again, "SNES/Amber Quay (Synthetic).sfc") == "nothing", Shown(again));
            Assert.Equal(before, originals.Select(Hash).ToArray());

            // The package moved: its record, settings and art go with it.
            string moved = Path.Combine(DataStore.Packages, "SNES", "Elsewhere", "Amber Quay (Synthetic).sfc");
            Directory.CreateDirectory(Path.GetDirectoryName(moved)!);
            Directory.Move(package, moved);
            Assert.Equal(record, EmuSen.Crystal.Managed.StateRecords.ReadPackaged(Path.Combine(moved, "states", "Amber Quay (Synthetic).slot3.json"), Path.Combine(moved, "states", "Amber Quay (Synthetic).slot3.state")));
            Assert.Equal(moved, GamePackages.RootFor(snes));
            Assert.Equal("VenusRT (Rust)", EmuSen.Crystal.Managed.ManagedRuntime.GameSetting(snes, "Engine"));
            Assert.Equal(Path.Combine(moved, "media", "covers", "Amber Quay (Synthetic).png"), Cover(moved));
            Assert.Equal(listing, Listing());
        }
    }
}
