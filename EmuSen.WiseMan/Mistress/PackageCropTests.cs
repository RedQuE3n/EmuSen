using System;
using System.IO;
using EmuSen.Crystal.Client.Library;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Media;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // A packaged game's crop in its package's settings.json, by its plain path and by its package, through both halves of games.db's records: the row read until the package holds one, never changed, and the crop kept when the package moves - see EmuSen_GamePackages.md §18.2.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class PackageCropTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenPackageCrop_" + Guid.NewGuid().ToString("N"));

        public PackageCropTests()
        {
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (Exception e) when (e is IOException or UnauthorizedAccessException) { }
        }

        [Theory]
        [InlineData(false)]
        [InlineData(true)]
        public void A_packaged_game_s_crop_is_its_package_s_by_either_path_the_row_read_until_then_and_kept_when_it_moves(bool native)
        {
            if (native) Assert.True(GameLibrary.Ready, GameLibrary.Report);
            string library = Path.Combine(_root, "Roms"), game = Path.Combine(library, "snes", "Crop (Synthetic).sfc");
            Directory.CreateDirectory(Path.GetDirectoryName(game)!);
            File.WriteAllBytes(game, SyntheticRom.BuildBlank());
            new AppSettings { RomDirectory = library }.Save();
            string plain = GameSettings.PlainPath("snes", game);
            Directory.CreateDirectory(Path.GetDirectoryName(plain)!);
            File.WriteAllText(plain, "{\n  \"Engine\": \"VenusRT (Rust)\"\n}");
            string db = Path.Combine(_root, "games.db");
            using GameRecords records = native ? GameRecords.OpenNative(db) : GameRecords.OpenManaged(db);
            records.SystemOf = _ => "snes";
            records.SetCrop(game, new GameCrop(1, 2, 3, 4));
            GameCrop? Row() { var opened = records.OpenedAs; records.OpenedAs = p => (p, null); try { return records.Crop(game); } finally { records.OpenedAs = opened; } }

            string package = Path.Combine(DataStore.Packages, "SNES", "Crop (Synthetic).sfc"), settings = Path.Combine(package, "settings.json");
            EmuSen.WiseMan.Crystal.CrystalParityTests.PackageFor(package, library, "snes/Crop (Synthetic).sfc", "snes");
            Assert.Equal(new GameCrop(1, 2, 3, 4), records.Crop(game));
            Assert.Equal(new GameCrop(1, 2, 3, 4), records.Crop(package));

            records.SetCrop(package, new GameCrop(5, 0, 2.5, 0));
            Assert.Equal(new GameCrop(5, 0, 2.5, 0), records.Crop(game));
            Assert.Equal(new GameCrop(5, 0, 2.5, 0), records.Crop(package));
            string written = File.ReadAllText(settings);
            Assert.Contains("\"Crop\": \"5 0 2.5 0\"", written);
            Assert.Contains("\"Engine\": \"VenusRT (Rust)\"", written);
            Assert.Equal("{\n  \"Engine\": \"VenusRT (Rust)\"\n}", File.ReadAllText(plain));
            Assert.Equal(new GameCrop(1, 2, 3, 4), Row());

            records.ClearCrop(game);
            Assert.Null(records.Crop(game));
            Assert.Null(records.Crop(package));
            Assert.Equal(new GameCrop(1, 2, 3, 4), Row());

            records.SetCrop(game, new GameCrop(7, 6.5, 0, 1));
            string moved = Path.Combine(DataStore.Packages, "SNES", "Elsewhere", "Crop (Synthetic).sfc");
            Directory.CreateDirectory(Path.GetDirectoryName(moved)!);
            Directory.Move(package, moved);
            Assert.Equal(new GameCrop(7, 6.5, 0, 1), records.Crop(game));
            Assert.Equal(new GameCrop(7, 6.5, 0, 1), records.Crop(moved));
            Assert.Equal(new GameCrop(1, 2, 3, 4), Row());
        }
    }
}
