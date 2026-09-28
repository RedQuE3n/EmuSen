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

        public void Dispose()
        {
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

        private MediaSources Sources(Dictionary<string, string> choices, bool handPlacedHack = true) => new(
            (console, title) => handPlacedHack && console == "NES" && title == "Hack of Mario (USA)" ? Path.Combine(_root, "Artwork", "NES", "Hack of Mario (USA).png") : null,
            Path.Combine(_root, "Media"), null, null, p => choices.GetValueOrDefault(p), string.Join(",", choices));

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
            Assert.Equal(MediaSource.HandPlaced, Sources(new() { [Hack] = Base }).Locate("nes", Hack, "cover").Source);
            Touch("Artwork", "NES", "Hack of Mario (USA).png");
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

        private static IReadOnlyList<string> ListedIn(CoverPickerWindow picker) => picker.Listed.Select(c => c.Title).ToList();

        [Fact]
        public Task From_the_desktop_s_context_menu_a_chosen_cover_is_shown_everywhere_kept_across_a_restart_and_undone() => Session.Dispatch(() =>
        {
            using ThemedSession s = DesktopGameOptionsTests.Desktop(AppSettings.LibraryList);
            (string first, string fourth) = Covers(s);
            Dictionary<string, (string, DateTime)> before = Fingerprint(s);
            Assert.Null(CoverShown(s, 2));

            SelectRow(s, 2);
            IReadOnlyList<string> labels = OpenFromContextMenu(s, "Use Another _Game's Cover...");
            Assert.DoesNotContain("Use Its O_wn Cover", labels);
            CoverPickerWindow picker = Assert.IsType<CoverPickerWindow>(s.Window.CoverPickerShown);
            Assert.Contains(picker, s.Window.OwnedWindows);
            // Only games with a cover to give, and never the game itself.
            Assert.Equal([ThemedSession.SnesGames[0], ThemedSession.SnesGames[3]], ListedIn(picker).Order());

            picker.Entries.Single(b => ((CoverCandidate)b.Tag!).Path == Rom(s, 3)).RaiseEvent(new Avalonia.Interactivity.RoutedEventArgs(Button.ClickEvent));
            s.Settle();
            Assert.Null(s.Window.CoverPickerShown);
            Assert.Equal(fourth, CoverShown(s, 2));
            Assert.Equal(fourth, ThemedCover(s, 2));
            // What the player placed for the chosen game is not moved, and games without a choice are unchanged.
            Assert.Equal(first, CoverShown(s, 0));
            Assert.Null(CoverShown(s, 1));
            Assert.Equal(Rom(s, 3), ThemedGameOptionsTests.StoredEdits(s, ThemedSession.SnesGames[2] + ".sfc")[GameMetadata.CoverFrom]);

            // Chosen over the player's own: a picture placed for game 0 still loses to game 3's when game 0 borrows it.
            SelectRow(s, 0);
            OpenFromContextMenu(s, "Use Another _Game's Cover...");
            picker = s.Window.CoverPickerShown!;
            Assert.DoesNotContain(ThemedSession.SnesGames[0], ListedIn(picker));
            picker.Choose(picker.Listed.Single(c => c.Path == Rom(s, 3)));
            s.Settle();
            Assert.Equal(fourth, CoverShown(s, 0));

            // A game whose cover already comes from this one is not offered, so no choice makes a loop.
            SelectRow(s, 3);
            OpenFromContextMenu(s, "Use Another _Game's Cover...");
            Assert.DoesNotContain(ThemedSession.SnesGames[2], ListedIn(s.Window.CoverPickerShown!));
            Assert.DoesNotContain(ThemedSession.SnesGames[0], ListedIn(s.Window.CoverPickerShown!));
            s.Window.CoverPickerShown!.Close();
            s.Settle();

            // Kept in games.db: another window reads it.
            s.Window.Close();
            var again = new MainWindow { Width = 1280, Height = 800 };
            again.Show();
            s.Settle();
            try
            {
                var entry = new RomEntry(Rom(s, 2));
                var clock = System.Diagnostics.Stopwatch.StartNew();
                string? shown = null;
                while ((shown = (string?)typeof(MainWindow).GetMethod("CoverPathFor", Hidden)!.Invoke(again, [entry])) != fourth && clock.ElapsedMilliseconds < 10_000)
                { Avalonia.Threading.Dispatcher.UIThread.RunJobs(); System.Threading.Thread.Sleep(5); }
                Assert.Equal(fourth, shown);
            }
            finally { again.Close(); }

            Assert.Equal(before, Fingerprint(s));
        }, default);

        [Fact]
        public Task Use_its_own_cover_undoes_the_choice_from_the_context_menu_and_from_the_picker() => Session.Dispatch(() =>
        {
            using ThemedSession s = DesktopGameOptionsTests.Desktop(AppSettings.LibraryList);
            (string first, string fourth) = Covers(s);
            Dictionary<string, (string, DateTime)> before = Fingerprint(s);

            SelectRow(s, 0);
            OpenFromContextMenu(s, "Use Another _Game's Cover...");
            s.Window.CoverPickerShown!.Choose(s.Window.CoverPickerShown!.Listed.Single(c => c.Path == Rom(s, 3)));
            s.Settle();
            Assert.Equal(fourth, CoverShown(s, 0));

            SelectRow(s, 0);
            Assert.Contains("Use Its O_wn Cover", OpenFromContextMenu(s, "Use Its O_wn Cover"));
            Assert.Equal(first, CoverShown(s, 0));
            Assert.False(ThemedGameOptionsTests.StoredEdits(s, ThemedSession.SnesGames[0] + ".sfc").ContainsKey(GameMetadata.CoverFrom));

            SelectRow(s, 0);
            OpenFromContextMenu(s, "Use Another _Game's Cover...");
            s.Window.CoverPickerShown!.Choose(s.Window.CoverPickerShown!.Listed.Single(c => c.Path == Rom(s, 3)));
            s.Settle();
            SelectRow(s, 0);
            OpenFromContextMenu(s, "Use Another _Game's Cover...");
            CoverPickerWindow picker = s.Window.CoverPickerShown!;
            Assert.Contains(ThemedSession.SnesGames[3], DesktopGameOptionsTests.Named<Avalonia.Controls.TextBlock>(picker, "CoverPickerHint").Text);
            DesktopGameOptionsTests.Click(s, DesktopGameOptionsTests.Named<Button>(picker, "CoverPickerUseOwn"));
            Assert.Null(s.Window.CoverPickerShown);
            Assert.Equal(first, CoverShown(s, 0));
            Assert.Equal(before, Fingerprint(s));
        }, default);

        [Fact]
        public Task By_pad_in_big_picture_the_options_open_the_picker_as_a_sheet_searched_on_the_on_screen_keyboard() => ThemedLibraryPadTests.Run(s =>
        {
            (_, string fourth) = Covers(s);
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.Down();
            s.Pad.Down();
            Assert.Equal(ThemedSession.SnesGames[2], s.Game);

            ThemedGameOptionsTests.Choose(s, "Use Another Game's Cover...");
            Assert.IsType<CoverPickerWindow>(ThemedGameOptionsTests.Sheets(s).Current);
            Assert.Empty(s.Window.OwnedWindows);

            // Every control of the sheet is reached by the d-pad.
            Control sheet = ThemedGameOptionsTests.Sheet(s);
            HashSet<Avalonia.Input.InputElement> reached = PadAudit.Reachable(sheet, s.Pad);
            Assert.Empty(PadAudit.Operable(sheet).Where(c => !reached.Contains(c)).Select(PadAudit.Describe));

            ThemedGameOptionsTests.Type(s, "CoverPickerSearch", "dune");
            CoverPickerWindow picker = (CoverPickerWindow)ThemedGameOptionsTests.Sheets(s).Current!;
            Assert.Equal([ThemedSession.SnesGames[3]], ListedIn(picker));
            PadAudit.Reach(ThemedGameOptionsTests.Sheet(s), s.Pad, e => e is Button { Tag: CoverCandidate c } && c.Path == Rom(s, 3));
            s.Pad.A();
            s.Settle();
            Assert.False(ThemedGameOptionsTests.Sheets(s).IsPresenting);
            Assert.Equal(fourth, ThemedCover(s, 2));
            Assert.Equal(fourth, CoverShown(s, 2));
            Assert.Equal(ThemedSession.SnesGames[2], s.Game);
            Assert.Equal(fourth, DrawnCover(s));

            // The options now offer the way back, and it works by pad too.
            ThemedGameOptionsTests.Choose(s, "Use Its Own Cover");
            Assert.Null(CoverShown(s, 2));
            Assert.Null(ThemedCover(s, 2));
            Assert.Null(DrawnCover(s));
        });

        [Fact]
        public Task By_pad_in_the_built_in_big_screen_the_pad_menu_s_game_options_reach_the_picker() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(settings: a => a.LibraryStyle = AppSettings.LibraryStyleMistress);
            (_, string fourth) = Covers(s);
            SelectRow(s, 2);
            typeof(MainWindow).GetMethod("ShowLibraryGameOptions", Hidden)!.Invoke(s.Window, [DesktopGameOptionsTests.List(s.Window).Selected!]);
            s.Settle();
            Assert.IsType<GameOptionsWindow>(ThemedGameOptionsTests.Sheets(s).Current);
            PadAudit.Reach(ThemedGameOptionsTests.Sheet(s), s.Pad, e => e is Button { Content: "Use Another Game's Cover..." });
            s.Pad.A();
            s.Settle();
            Assert.IsType<CoverPickerWindow>(ThemedGameOptionsTests.Sheets(s).Current);
            PadAudit.Reach(ThemedGameOptionsTests.Sheet(s), s.Pad, e => e is Button { Tag: CoverCandidate c } && c.Path == Rom(s, 3));
            s.Pad.A();
            s.Settle();
            Assert.Equal(fourth, CoverShown(s, 2));
        }, default);

        private static void SelectRow(ThemedSession s, int n)
        {
            var list = DesktopGameOptionsTests.List(s.Window);
            list.SelectedIndex = list.Models.ToList().FindIndex(e => e.FullPath == Rom(s, n));
            s.Settle();
            Assert.Equal(Rom(s, n), list.Selected!.FullPath);
        }

        // A right-click on the selected row, which opens the list's context menu, and the labelled entry chosen.
        private static IReadOnlyList<string> OpenFromContextMenu(ThemedSession s, string label)
        {
            var list = DesktopGameOptionsTests.List(s.Window);
            Control row = list.ContainerFromIndex(list.SelectedIndex)!;
            Avalonia.Point centre = row.TranslatePoint(new Avalonia.Point(row.Bounds.Width / 2, row.Bounds.Height / 2), s.Window)!.Value;
            s.Window.MouseDown(centre, Avalonia.Input.MouseButton.Right);
            s.Window.MouseUp(centre, Avalonia.Input.MouseButton.Right);
            s.Settle();
            return DesktopGameOptionsTests.ChooseFromContextMenu(s, list, label);
        }
    }
}
