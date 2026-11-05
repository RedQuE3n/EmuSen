using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Security.Cryptography;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Media;
using Avalonia.VisualTree;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Mistress.BigPicture;

namespace EmuSen.WiseMan.Mistress
{
    // Q45, Use Another Game's Cover...: the player's choice first in the order, kept in games.db, undone, and never a file touched - see EmuSen_Settings_Reference.md §4.65.
    [Collection(TestCollections.ProcessGlobals)]
    public class CoverChoiceTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(CoverChoiceTests).GetTypeInfo().Assembly);

        private const BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenCoverChoiceTests", Guid.NewGuid().ToString("N"));

        private readonly List<IDisposable> _held = new();

        public void Dispose()
        {
            foreach (IDisposable d in Enumerable.Reverse(_held)) d.Dispose();
            try { Directory.Delete(_root, recursive: true); } catch { }
        }

        // --- the order ---

        private string Touch(params string[] parts)
        {
            string path = Path.Combine([_root, .. parts]);
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllBytes(path, new byte[100]);
            return path;
        }

        private const string Hack = "/roms/Hack of Mario (USA).nes", Base = "/roms/Mario (USA).nes", Other = "/roms/Other (USA).sfc", Loop = "/roms/Loop (USA).nes";

        // The library's sources over a store holding nothing, the choices kept in a games.db of their own as the window keeps them, the player's covers in Artwork or none.
        private MediaSources Sources(Dictionary<string, string> choices, bool handPlacedHack = true)
        {
            Directory.CreateDirectory(_root);
            string db = Path.Combine(_root, $"games{_held.Count}.db");
            using (GameRecords records = GameRecords.Open(db))
                foreach (var (game, from) in choices) records.SetCoverChoice(game, from, new DateTime(2026, 9, 27));
            var library = EmuSen.Crystal.Client.Library.GameLibrary.Open(db);
            _held.Add(library);
            string settings = System.Text.Json.JsonSerializer.Serialize(new Dictionary<string, object?>
            {
                ["developer_files"] = false, ["openemu"] = false, ["sources_alone"] = true, ["art"] = Path.Combine(_root, handPlacedHack ? "Artwork" : "NoArtwork"),
                ["cores"] = RuntimeCatalog.Consoles.Select(c => c.Systems).ToArray(), ["test"] = new { offline = true },
            });
            var native = NativeScraping.Open(Path.Combine(_root, "Media"), settings, library, null);
            _held.Add(native);
            return new MediaSources(native, string.Join(",", choices));
        }

        [Fact]
        public void A_chosen_game_s_cover_comes_before_even_the_player_s_own_and_only_for_covers()
        {
            Touch("Artwork", "NES", "Hack of Mario (USA).png");
            string baseCover = Touch("Media", "nes", "covers", "Mario (USA).png");
            Touch("Media", "nes", "screenshots", "Mario (USA).png");
            var choices = new Dictionary<string, string> { [Hack] = Base };

            Assert.Equal((MediaSource.OtherGame, baseCover), Sources(choices).Locate("nes", Hack, "cover"));
            Assert.Equal(MediaSource.None, Sources(choices).Locate("nes", Hack, "screenshot").Source);
            Assert.Equal(MediaSource.HandPlaced, Sources([]).Locate("nes", Hack, "cover").Source);
            Assert.NotEqual(Sources([]).Stamp(new ThemeSystem("nes", "nes", "nes")), Sources(choices).Stamp(new ThemeSystem("nes", "nes", "nes")));
        }

        [Fact]
        public void A_chosen_game_across_consoles_a_chain_a_loop_and_a_game_with_nothing_to_give()
        {
            string otherCover = Touch("Media", "snes", "covers", "Other (USA).png");
            string loopCover = Touch("Media", "nes", "covers", "Loop (USA).png");

            // Another console's shelf is looked up under its own system.
            Assert.Equal((MediaSource.OtherGame, otherCover), Sources(new() { [Hack] = Other }).Locate("nes", Hack, "cover"));
            // A chain is followed: the hack shows what Mario shows, which is Other's.
            Assert.Equal(otherCover, Sources(new() { [Hack] = Base, [Base] = Other }).Cover("nes", Hack));
            // A loop stops at the game reached twice, which shows its own cover.
            Assert.Equal(loopCover, Sources(new() { [Hack] = Loop, [Loop] = Hack }, handPlacedHack: false).Cover("nes", Hack));
            // A chosen game with no cover leaves the game its own.
            Touch("Artwork", "NES", "Hack of Mario (USA).png");
            Assert.Equal(MediaSource.HandPlaced, Sources(new() { [Hack] = Base }).Locate("nes", Hack, "cover").Source);
            Assert.Equal(MediaSource.None, Sources(new() { [Hack] = Base }, handPlacedHack: false).Locate("nes", Hack, "cover").Source);
        }

        // --- where it is kept ---

        [Fact]
        public void The_choice_is_a_games_db_row_that_clear_keeps_a_rename_follows_and_null_removes()
        {
            Directory.CreateDirectory(_root);
            using GameRecords records = GameRecords.Open(Path.Combine(_root, "games.db"));
            DateTime now = new(2026, 9, 27);
            foreach (string game in (string[])[Hack, Base]) records.SetFavourite(game, false);
            records.SetCoverChoice(Hack, Base, now);
            records.SaveEdits(Hack, new Dictionary<string, string?> { [GameMetadata.Name] = "My Hack" }, now);
            Assert.Equal(Base, records.CoverChoice(Hack));
            Assert.Equal(Base, records.AllEdits()[Hack][GameMetadata.CoverFrom]);

            // The editor's Clear is metadata; the borrowed cover is the player's cover and stays.
            records.ClearEdits(Hack);
            Assert.Equal(Base, records.CoverChoice(Hack));
            Assert.False(records.Edits(Hack).ContainsKey(GameMetadata.Name));
            // The editor's draft has no field for it, so a Save never writes or removes it.
            Assert.Empty(new MetadataDraft(Hack, null, records.Edits(Hack), null).Changes());

            // The chosen game renamed: the choice follows it. The choosing game renamed: its row moves with it.
            records.Move(Base, "/roms/Mario Renamed (USA).nes");
            Assert.Equal("/roms/Mario Renamed (USA).nes", records.CoverChoice(Hack));
            records.Move(Hack, "/roms/Hack Renamed (USA).nes");
            Assert.Equal("/roms/Mario Renamed (USA).nes", records.CoverChoice("/roms/Hack Renamed (USA).nes"));
            Assert.Null(records.CoverChoice(Hack));

            records.SetCoverChoice("/roms/Hack Renamed (USA).nes", null, now);
            Assert.Null(records.CoverChoice("/roms/Hack Renamed (USA).nes"));
            records.SetCoverChoice(Base, Base, now);
            Assert.Null(records.CoverChoice(Base));
        }

        [Fact]
        public void The_picker_lists_the_games_nearest_by_name_first_and_filters_by_the_search()
        {
            CoverCandidate C(string t) => new("/r/" + t, t, "NES", "/c/" + t);
            CoverCandidate[] all = [C("Zelda (USA)"), C("Super Mario Bros. (World)"), C("Super Metroid (USA)"), C("Super Mario Bros. 3 (USA)"), C("Adventure (USA)")];
            Assert.Equal(["Super Mario Bros. (World)", "Super Mario Bros. 3 (USA)", "Super Metroid (USA)", "Adventure (USA)", "Zelda (USA)"],
                CoverCandidates.Rank("Super Mario Bros. (Hack) [Luigi]", all, null).Select(c => c.Title));
            Assert.Equal(["Super Metroid (USA)"], CoverCandidates.Rank("Super Mario Bros. (Hack)", all, "metroid").Select(c => c.Title));
            Assert.Equal(CoverCandidates.Shown, CoverCandidates.Rank("x", Enumerable.Range(0, 200).Select(i => C($"Game {i:000}")), null).Count);
        }

        // --- the windows ---

        private static string Rom(ThemedSession s, int n) => Path.Combine(s.RomDirectory, ThemedSession.SnesGames[n] + ".sfc");

        // Pictures in the player's art folder for games 0 and 3, as Add Cover Art would put them, and the index read again.
        private static (string First, string Fourth) Covers(ThemedSession s)
        {
            string first = Path.Combine(DataStore.Artwork, "SNES", ThemedSession.SnesGames[0] + ".png");
            string fourth = Path.Combine(DataStore.Artwork, "SNES", ThemedSession.SnesGames[3] + ".png");
            Directory.CreateDirectory(Path.GetDirectoryName(first)!);
            File.Copy(SceneAssets.Halves("choice-first", 120, 160, Colors.Crimson, Colors.Gold), first, overwrite: true);
            File.Copy(SceneAssets.Halves("choice-fourth", 120, 160, Colors.Teal, Colors.White), fourth, overwrite: true);
            typeof(MainWindow).GetMethod("ScanArtwork", Hidden)!.Invoke(s.Window, null);
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (CoverShown(s, 0) != first && clock.ElapsedMilliseconds < 10_000) { s.Settle(); System.Threading.Thread.Sleep(5); }
            Assert.Equal(first, CoverShown(s, 0));
            return (first, fourth);
        }

        internal static string? CoverShown(ThemedSession s, int n) =>
            (string?)typeof(MainWindow).GetMethod("CoverPathFor", Hidden)!.Invoke(s.Window, [new RomEntry(Rom(s, n)) { }]);

        private static string? ThemedCover(ThemedSession s, int n) =>
            ((ISceneMedia)typeof(MainWindow).GetMethod("ThemedMedia", Hidden)!.Invoke(s.Window, null)!)
                .Find(new ThemeSystem("snes", "Super Nintendo", "snes"), new SceneGame(ThemedSession.SnesGames[n], Rom(s, n)), "cover");

        // The picture the themed gamelist's cover image is drawing now.
        private static string? DrawnCover(ThemedSession s)
        {
            s.Settle();
            return (s.Themed.Stage!.Current.Scene.Find("image", "cover")?.Control as FittedImage)?.Source;
        }

        // Every file under the ROM folder and the art folder, with its bytes and time: what the choice must never change.
        private static Dictionary<string, (string, DateTime)> Fingerprint(ThemedSession s) =>
            new[] { s.RomDirectory, DataStore.Artwork }.Where(Directory.Exists)
                .SelectMany(d => Directory.EnumerateFiles(d, "*", SearchOption.AllDirectories))
                .ToDictionary(f => f, f => (Convert.ToHexString(MD5.HashData(File.ReadAllBytes(f))), File.GetLastWriteTimeUtc(f)));

    }
}
