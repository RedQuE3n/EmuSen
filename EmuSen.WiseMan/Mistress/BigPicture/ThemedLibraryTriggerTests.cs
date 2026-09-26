using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Headless;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The noMedia and noVideos triggers against the media Mistress actually has - see EmuSen_BigPicture.md §16.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemedLibraryTriggerTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ThemedLibraryTriggerTests).GetTypeInfo().Assembly);

        private readonly ITestOutputHelper _out;

        public ThemedLibraryTriggerTests(ITestOutputHelper output) => _out = output;

        private static string NewMediaFolder()
        {
            string media = Path.Combine(Path.GetTempPath(), "EmuSenTriggerMedia", Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(media);
            return media;
        }

        private static void Put(string media, string system, string folder, string file)
        {
            string path = Path.Combine(media, system, folder, file);
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.Copy(SceneAssets.Halves("trigger-cover", 40, 60, Avalonia.Media.Colors.Teal, Avalonia.Media.Colors.Gold), path, overwrite: true);
        }

        // P52: USERGUIDE.md lists .jpg, .png and .webp for images and six extensions for videos; each is found, and a .jpeg is not.
        [Fact]
        public void An_ES_DE_media_folder_finds_the_extensions_the_user_guide_lists()
        {
            string media = NewMediaFolder();
            var folder = new EsdeMediaFolder(media);
            ThemeSystem snes = SyntheticTheme.Snes;
            string[] videos = [".mp4", ".mkv", ".avi", ".wmv", ".mov", ".webm"];
            foreach (string ext in videos) Put(media, "snes", "videos", "V" + ext);
            foreach (string ext in new[] { ".png", ".jpg", ".webp", ".jpeg" }) Put(media, "snes", "covers", "C" + ext);

            foreach (string ext in videos) Assert.NotNull(Find(folder, ext));
            Assert.EndsWith("C.png", folder.Find(snes, new SceneGame("C", "/roms/C.sfc"), "cover"));
            File.Delete(Path.Combine(media, "snes", "covers", "C.png"));
            File.Delete(Path.Combine(media, "snes", "covers", "C.jpg"));
            Assert.EndsWith("C.webp", folder.Find(snes, new SceneGame("C", "/roms/C.sfc"), "cover"));
            File.Delete(Path.Combine(media, "snes", "covers", "C.webp"));
            Assert.Null(folder.Find(snes, new SceneGame("C", "/roms/C.sfc"), "cover"));
            Directory.Delete(media, true);
        }

        // One video per extension, found alone, so every extension is shown to be looked for.
        private static string? Find(EsdeMediaFolder folder, string ext)
        {
            string dir = Path.Combine(folder.Root, "snes", "videos");
            foreach (string other in Directory.GetFiles(dir).Where(f => !f.EndsWith(ext))) File.Move(other, other + ".off");
            string? found = folder.Find(SyntheticTheme.Snes, new SceneGame("V", "/roms/V.sfc"), "video");
            foreach (string off in Directory.GetFiles(dir, "*.off")) File.Move(off, off[..^4]);
            return found is not null && found.EndsWith(ext) ? found : null;
        }

        // The presence a listing gives equals the presence asked game by game, for every type.
        [Fact]
        public void Presence_from_one_listing_equals_presence_asked_game_by_game()
        {
            string media = NewMediaFolder();
            var folder = new EsdeMediaFolder(media);
            var games = Enumerable.Range(0, 30).Select(i => new SceneGame($"Game {i}", $"/roms/Game {i}.sfc")).ToList();
            Put(media, "snes", "covers", "Game 3.png");
            Put(media, "snes", "screenshots", "Game 29.jpg");
            Put(media, "snes", "videos", "Game 7.webm");
            Put(media, "snes", "marquees", "Not a game.png");
            Put(media, "snes", "fanart", "Game 4.gif");
            var asked = ThemeCapabilities.MediaTypes.Where(t => games.Any(g => folder.Find(SyntheticTheme.Snes, g, t) is not null)).ToHashSet();
            var listed = ((ISceneMedia)folder).Present(SyntheticTheme.Snes, games);
            Assert.Equal(asked.Order(), listed.Order());
            Assert.Equal(new[] { "cover", "screenshot", "video" }, listed.Order());
            Directory.Delete(media, true);
        }

        // P53 on a full folder: 3,508 games, a cover for each and a screenshot for every other one, asked game by game and listed once.
        [Fact]
        public void A_full_media_folder_is_listed_once_rather_than_asked_game_by_game()
        {
            string media = NewMediaFolder();
            var folder = new EsdeMediaFolder(media);
            var games = Enumerable.Range(0, 3508).Select(i => new SceneGame($"Filler {i:D4}", $"/roms/Filler {i:D4}.sfc")).ToList();
            foreach (string dir in new[] { "covers", "screenshots" }) Directory.CreateDirectory(Path.Combine(media, "snes", dir));
            for (int i = 0; i < games.Count; i++)
            {
                File.WriteAllBytes(Path.Combine(media, "snes", "covers", $"Filler {i:D4}.png"), []);
                if (i % 2 == 1) File.WriteAllBytes(Path.Combine(media, "snes", "screenshots", $"Filler {i:D4}.jpg"), []);
            }

            ISceneMedia asker = new AskEachGame(folder);
            asker.Present(SyntheticTheme.Snes, games);
            ((ISceneMedia)folder).Present(SyntheticTheme.Snes, games);
            var clock = System.Diagnostics.Stopwatch.StartNew();
            var asked = asker.Present(SyntheticTheme.Snes, games);
            double askedMs = clock.Elapsed.TotalMilliseconds;
            clock.Restart();
            var listed = ((ISceneMedia)folder).Present(SyntheticTheme.Snes, games);
            double listedMs = clock.Elapsed.TotalMilliseconds;
            _out.WriteLine($"3,508 games, 5,262 files: asked game by game {askedMs:F1} ms, listed once {listedMs:F1} ms");
            Assert.Equal(asked.Order(), listed.Order());
            Assert.True(listedMs < askedMs, $"{listedMs} against {askedMs}");
            Directory.Delete(media, true);
        }

        // The interface's own game-by-game answer, which the listing replaces.
        private sealed class AskEachGame(EsdeMediaFolder folder) : ISceneMedia
        {
            public string? Find(ThemeSystem system, SceneGame game, string mediaType) => folder.Find(system, game, mediaType);
        }

        private static string TriggerTheme(SyntheticTheme theme)
        {
            ThemedSession.Write(theme);
            theme.Capabilities(
                "<variant name=\"full\"><label>Full</label>" +
                "<override><trigger>noMedia</trigger><mediaType>cover</mediaType><useVariant>bare</useVariant></override>" +
                "<override><trigger>noVideos</trigger><useVariant>novideo</useVariant></override></variant>" +
                "<variant name=\"bare\"><label>Bare</label></variant><variant name=\"novideo\"><label>No video</label></variant>");
            return theme.Root;
        }

        private static string? GamelistVariant(ThemedSession s) => s.Themed.Stage!.Current.Data.System.Theme.GamelistVariant;

        private static void Refresh(ThemedSession s)
        {
            typeof(MainWindow).GetMethod("RefreshLibrary", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(s.Window, null);
            s.Settle();
        }

        // P52: a system whose covers and videos are all in the ES-DE folder keeps the chosen variant; taking them away fires noVideos, then noMedia, at the next showing.
        [Fact]
        public Task The_gamelist_variant_follows_the_media_the_library_has() => Session.Dispatch(() =>
        {
            string media = NewMediaFolder();
            using var theme = new SyntheticTheme();
            TriggerTheme(theme);
            foreach (string g in ThemedSession.SnesGames) Put(media, "snes", "covers", g + ".png");
            foreach (string g in ThemedSession.SnesGames) Put(media, "snes", "videos", g + ".mp4");
            using var s = new ThemedSession(settings: a => a.EsdeMediaDirectory = media, themeDirectory: theme.Root);
            s.Themed.PressDirection(EmuSen.Mistress.Input.UiButton.Left, s.Now);
            s.Themed.ReleaseDirection(s.Now);
            s.Themed.Command(EmuSen.Mistress.Input.UiButton.Accept, s.Now);
            Assert.Equal("snes", s.System);
            Assert.Equal("gamelist", s.View);
            Assert.Equal("full", GamelistVariant(s));

            Directory.Delete(Path.Combine(media, "snes", "videos"), true);
            Refresh(s);
            Assert.Equal("novideo", GamelistVariant(s));

            Directory.Delete(Path.Combine(media, "snes", "covers"), true);
            Refresh(s);
            Assert.Equal("bare", GamelistVariant(s));

            Put(media, "snes", "covers", ThemedSession.SnesGames[2] + ".webp");
            Refresh(s);
            Assert.Equal("novideo", GamelistVariant(s));
            Assert.Equal("full", s.Themed.Stage.Current.Data.System.Theme.Selection.Variant);
            Directory.Delete(media, true);
        }, default);
    }
}
