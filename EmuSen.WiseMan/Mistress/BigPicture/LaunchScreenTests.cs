using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Media;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The launch screen between the resume question and the game, under every look - see EmuSen_Interface.md §18.3.
    [Collection(TestCollections.ProcessGlobals)]
    public class LaunchScreenTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(LaunchScreenTests).GetTypeInfo().Assembly);

        private static readonly BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;

        private readonly ITestOutputHelper _out;

        public LaunchScreenTests(ITestOutputHelper output) => _out = output;

        internal const string Aurora = "Aurora Drift (Synthetic)", Brass = "Brass Lantern (Synthetic)", Cobalt = "Cobalt Harbor (Synthetic)";

        internal static void Stop(MainWindow w) => typeof(MainWindow).GetMethod("StopEmulationThread", Hidden)!.Invoke(w, null);

        internal static string? Running(MainWindow w) => (string?)typeof(MainWindow).GetField("_currentRomPath", Hidden)!.GetValue(w);

        internal static LaunchScreen? Screen(ThemedSession s) => (LaunchScreen?)typeof(MainWindow).GetProperty("LaunchScreenShown", Hidden)!.GetValue(s.Window);

        internal static LaunchScreen Host(ThemedSession s) => Avalonia.VisualTree.VisualExtensions.GetVisualDescendants(s.Window).OfType<LaunchScreen>().Single();

        private static bool GameShown(ThemedSession s) => s.Window.GetControl<Control>("GameFrame").IsVisible;

        private static string Rom(ThemedSession s, string game) => Path.Combine(s.RomDirectory, game + ".sfc");

        // An ES-DE media folder with a marquee for Aurora, a cover for Aurora and Brass, and nothing for Cobalt.
        internal static string Media(string root)
        {
            string media = Path.Combine(root, "downloaded_media");
            void Put(string kind, string game, int w, int h, Color left, Color right)
            {
                string path = Path.Combine(media, "snes", kind, game + ".png");
                Directory.CreateDirectory(Path.GetDirectoryName(path)!);
                File.Copy(SceneAssets.Halves($"launch-{kind}-{w}x{h}", w, h, left, right), path, overwrite: true);
            }
            Put("marquees", Aurora, 800, 300, Colors.MediumPurple, Colors.Gold);
            Put("covers", Aurora, 300, 400, Colors.Crimson, Colors.Orange);
            Put("covers", Brass, 300, 400, Colors.SeaGreen, Colors.Khaki);
            return media;
        }

        internal static ThemedSession Open(string duration, double width = 1280, double height = 800, Action<AppSettings>? more = null, Action<string>? roms = null)
        {
            string? media = null;
            return new ThemedSession(width, height, settings: a =>
            {
                a.BigPictureInterface.LaunchScreenDuration = duration;
                // The launch screen's own tests keep ES-DE's default opening, Scale-up, which ThemedSession turns off for the rest (§4.72.10).
                a.BigPictureInterface.MenuOpeningEffect = BigPictureInterface.OpeningScaleUp;
                a.EsdeMediaDirectory = media;
                more?.Invoke(a);
            }, roms: r => { media = Media(Path.GetDirectoryName(r)!); roms?.Invoke(r); });
        }

        // Normal's hold, which Open's sessions use.
        internal static readonly int Hold = (int)LaunchScreen.HoldFor(BigPictureInterface.LaunchNormal).TotalMilliseconds;

        // Each choice holds the game back for its least time, to within one 16 ms poll, and not a poll longer; ES-DE's Popup reads as Brief.
        [Theory]
        [InlineData(BigPictureInterface.LaunchNormal, 2000)]
        [InlineData(BigPictureInterface.LaunchBrief, 1000)]
        [InlineData(BigPictureInterface.LaunchLong, 3000)]
        [InlineData(BigPictureInterface.LaunchPopup, 1000)]
        public Task Each_choice_holds_the_game_for_its_least_time(string duration, int ms) => Session.Dispatch(() =>
        {
            using var s = Open(duration);
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Sounds.Clear();
            TimeSpan pressed = s.Now;
            s.Pad.A();
            Assert.Equal(["launch"], s.Sounds);
            LaunchScreen screen = Assert.IsType<LaunchScreen>(Screen(s));
            Assert.Equal(pressed + TimeSpan.FromMilliseconds(ms), screen.Due);
            Assert.Null(Running(s.Window));
            Assert.False(GameShown(s));

            s.Run(ms - 16);
            Assert.Null(Running(s.Window));
            Assert.NotNull(Screen(s));
            Assert.True(s.Shown);

            s.Run(16);
            Assert.Equal(Rom(s, Aurora), Running(s.Window));
            Assert.True(GameShown(s));
            Assert.Null(Screen(s));
            Assert.False(Host(s).IsVisible);
            Stop(s.Window);
        }, default);

        // A v1 engine's disc is loaded while the screen shows, a zip unpacked under it, and the game still starts at the screen's end - see Diamond_Players.md §12.8.3.
        [Fact]
        public Task A_disc_is_loaded_while_the_screen_shows_and_starts_at_its_end() => Session.Dispatch(() =>
        {
            string zip = "";
            using var s = new ThemedSession(settings: a => a.BigPictureInterface.LaunchScreenDuration = BigPictureInterface.LaunchNormal, roms: r =>
            {
                Directory.CreateDirectory(Path.Combine(r, "psx"));
                zip = Path.Combine(r, "psx", "Opal Reef.zip");
                using var archive = System.IO.Compression.ZipFile.Open(zip, System.IO.Compression.ZipArchiveMode.Create);
                using var entry = archive.CreateEntry("Opal Reef/Opal Reef.bin").Open();
                entry.Write(SyntheticPsxDisc.Build().DataTrack);
            });
            string marker = Path.Combine(EmuSen.Galaxia.Media.ArchiveImages.FolderFor(zip), ".unpacked");
            using var loaded = LoadsUnderTheScreen(s);
            for (int guard = 0; guard < 6 && s.System != "psx"; guard++) s.Pad.Right();
            s.Pad.A();
            s.Pad.A();
            Assert.NotNull(Screen(s));
            Assert.True(loaded.Wait(TimeSpan.FromSeconds(30)), "the disc was not loaded while the screen showed");
            Assert.True(File.Exists(marker));
            Assert.Null(Running(s.Window));
            s.Run(Hold - 16);
            Assert.Null(Running(s.Window));
            Assert.NotNull(Screen(s));
            s.Run(16);
            Assert.Equal(zip, Running(s.Window));
            Assert.True(GameShown(s));
            Assert.Null(Screen(s));
            Stop(s.Window);
        }, default);

        // A load that fails under the screen is reported at its end as any failed load: no game, the screen down, the core's words on the status line.
        [Fact]
        public Task A_disc_the_core_refuses_under_the_screen_is_reported_at_its_end() => Session.Dispatch(() =>
        {
            string bin = "";
            using var s = new ThemedSession(settings: a => a.BigPictureInterface.LaunchScreenDuration = BigPictureInterface.LaunchNormal, roms: r =>
            {
                Directory.CreateDirectory(Path.Combine(r, "psx"));
                bin = Path.Combine(r, "psx", "Opal Fault.bin");
                File.WriteAllBytes(bin, SyntheticPsxDisc.Build().DataTrack);
                File.WriteAllBytes(Path.ChangeExtension(bin, ".sbi"), [.. "SBI\0"u8, 0, 0, 0]);
            });
            Assert.NotNull(EmuSen.Mistress.Library.RuntimeCatalog.Route(bin).Engine);
            using var loaded = LoadsUnderTheScreen(s);
            for (int guard = 0; guard < 6 && s.System != "psx"; guard++) s.Pad.Right();
            s.Pad.A();
            s.Pad.A();
            Assert.NotNull(Screen(s));
            Assert.True(loaded.Wait(TimeSpan.FromSeconds(30)));
            s.Run(3000);
            Pump(50);
            Assert.Null(Running(s.Window));
            Assert.False(GameShown(s));
            Assert.Null(Screen(s));
            Assert.StartsWith("Failed to load Opal Fault.bin: Diamond refused the game", s.Window.GetControl<TextBlock>("StatusText").Text);
        }, default);

        // The window's own load under the screen put back, which ThemedSession takes away; set when a load has ended, well or not.
        private static System.Threading.ManualResetEventSlim LoadsUnderTheScreen(ThemedSession s)
        {
            var ended = new System.Threading.ManualResetEventSlim();
            var open = MistressGame.OwnOpen(s.Window);
            MistressGame.EarlyLoad(s.Window, what =>
            {
                try { return open(what); }
                finally { ended.Set(); }
            });
            return ended;
        }

        // A themed session with one synthetic disc whose load under the screen waits for the gate; the disc's path.
        private static ThemedSession HeldDisc(System.Threading.ManualResetEventSlim gate, System.Threading.ManualResetEventSlim loaded, List<EmuSen.Mistress.GameSession> cores, out string bin)
        {
            string path = "";
            var s = new ThemedSession(settings: a => a.BigPictureInterface.LaunchScreenDuration = BigPictureInterface.LaunchNormal, roms: r =>
            {
                Directory.CreateDirectory(Path.Combine(r, "psx"));
                path = Path.Combine(r, "psx", "Opal Wait.bin");
                File.WriteAllBytes(path, SyntheticPsxDisc.Build().DataTrack);
            });
            var open = MistressGame.OwnOpen(s.Window);
            MistressGame.EarlyLoad(s.Window, what =>
            {
                gate.Wait();
                var core = open(what);
                cores.Add(core);
                loaded.Set();
                return core;
            });
            for (int guard = 0; guard < 6 && s.System != "psx"; guard++) s.Pad.Right();
            s.Pad.A();
            s.Pad.A();
            bin = path;
            return s;
        }

        // Posted work run for a while on the wall clock, as the window's loop would run a continuation that a worker posts.
        private static void Pump(int ms = 300)
        {
            var until = System.Diagnostics.Stopwatch.StartNew();
            while (until.ElapsedMilliseconds < ms)
            {
                Avalonia.Threading.Dispatcher.UIThread.RunJobs();
                System.Threading.Thread.Sleep(5);
            }
        }

        // A load longer than the screen's time keeps the card up and starts the game when it is done - see Diamond_Players.md §12.8.3.
        [Fact]
        public Task A_load_longer_than_the_screen_keeps_the_card_and_starts_the_game_when_it_is_done() => Session.Dispatch(() =>
        {
            using var gate = new System.Threading.ManualResetEventSlim();
            using var loaded = new System.Threading.ManualResetEventSlim();
            var cores = new List<EmuSen.Mistress.GameSession>();
            using var s = HeldDisc(gate, loaded, cores, out string bin);
            s.Run(3000 + 160);
            Assert.Null(Running(s.Window));
            Assert.NotNull(Screen(s));
            gate.Set();
            Assert.True(loaded.Wait(TimeSpan.FromSeconds(30)));
            Pump();
            Assert.Equal(bin, Running(s.Window));
            Assert.True(GameShown(s));
            Assert.Null(Screen(s));
            Stop(s.Window);
        }, default);

        // A window closed while the load outlasts the screen starts no game when the load ends, and lets its core go.
        [Fact]
        public Task A_window_closed_while_the_load_outlasts_the_screen_starts_nothing() => Session.Dispatch(() =>
        {
            using var gate = new System.Threading.ManualResetEventSlim();
            using var loaded = new System.Threading.ManualResetEventSlim();
            var cores = new List<EmuSen.Mistress.GameSession>();
            using var s = HeldDisc(gate, loaded, cores, out _);
            s.Run(3000 + 160);
            Assert.Null(Running(s.Window));
            s.Window.Close();
            gate.Set();
            Assert.True(loaded.Wait(TimeSpan.FromSeconds(30)));
            Pump();
            Assert.Null(Running(s.Window));
            Assert.False(GameShown(s));
            Assert.True(System.Threading.SpinWait.SpinUntil(() => cores.Single().Crystal.Closed, TimeSpan.FromSeconds(10)), "the session opened for a closed window was kept");
        }, default);

        [Fact]
        public Task Disabled_starts_the_game_at_once_with_no_screen() => Session.Dispatch(() =>
        {
            using var s = Open(BigPictureInterface.LaunchDisabled);
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.A();
            Assert.Equal(Rom(s, Aurora), Running(s.Window));
            Assert.True(GameShown(s));
            Assert.Null(Screen(s));
            Assert.Empty(s.Window.GetVisualDescendantsOf<LaunchScreen>());
            Stop(s.Window);
        }, default);

        // The stored values stay ES-DE's and Normal is the default; Popup is no longer offered and reads as Brief.
        [Fact]
        public void Normal_is_the_default_and_the_stored_values_stay_compatible()
        {
            Assert.Equal("normal", new BigPictureInterface().LaunchScreenDuration);
            Assert.Equal(["normal", "brief", "long", "disabled"], InterfaceSettingsPane.LaunchScreenChoices.Select(c => c.Value));
            Assert.Equal(2000, LaunchScreen.HoldFor("normal").TotalMilliseconds);
            Assert.Equal(1000, LaunchScreen.HoldFor("popup").TotalMilliseconds);
            Assert.Equal(2000, LaunchScreen.HoldFor("anything else").TotalMilliseconds);
            Assert.False(LaunchScreen.Shows("disabled"));
        }

        [Fact]
        public Task The_screen_shows_the_game_s_name_its_system_and_its_marquee_else_its_cover_else_no_picture() => Session.Dispatch(() =>
        {
            using var s = Open(BigPictureInterface.LaunchNormal);
            ThemedLibraryPadTests.Enter(s, "snes");
            string media = s.Settings.EsdeMediaDirectory!;
            var expected = new (string Game, string? Art)[]
            {
                (Aurora, Path.Combine(media, "snes", "marquees", Aurora + ".png")),
                (Brass, Path.Combine(media, "snes", "covers", Brass + ".png")),
                (Cobalt, null),
            };
            foreach ((string game, string? art) in expected)
            {
                for (int guard = 0; guard < 10 && s.Game != game; guard++) s.Pad.Down();
                Assert.Equal((game, "gamelist"), (s.Game, s.View));
                s.Pad.A();
                LaunchScreen screen = Assert.IsType<LaunchScreen>(Screen(s));
                Assert.Equal(game, screen.GameName);
                Assert.StartsWith("Super Nintendo · ", screen.SystemName);
                Assert.Equal(art, screen.ArtPath);
                Assert.Equal(game == Aurora, screen.ShowsMarquee);
                s.Run(3000);
                Assert.Equal(Rom(s, game), Running(s.Window));
                ThemedLibraryFlowTests.QuitGame(s);
                // The card slides out before the view takes the pad (EmuSen_Interface.md §20.6.1).
                s.Run(200);
                Assert.Null(Running(s.Window));
                if (s.View != "gamelist") s.Pad.A();
                Assert.Equal("gamelist", s.View);
            }
        }, default);

        // The metadata editor's name is the one shown, as the list shows it.
        [Fact]
        public Task An_edited_name_is_the_name_shown() => Session.Dispatch(() =>
        {
            using var s = Open(BigPictureInterface.LaunchNormal);
            ThemedCollectionsTests.Records(s).SaveEdits(Rom(s, Aurora), new Dictionary<string, string?> { [EmuSen.Mistress.Library.GameMetadata.Name] = "Aurora Drift Deluxe" }, DateTime.Now);
            ThemedCollectionsTests.Refresh(s);
            ThemedLibraryPadTests.Enter(s, "snes");
            Assert.Equal("Aurora Drift Deluxe", s.Game);
            s.Pad.A();
            Assert.Equal("Aurora Drift Deluxe", Screen(s)!.GameName);
            s.Run(3000);
            Stop(s.Window);
        }, default);

        // ES-DE takes no input while its screen is up (§33): a direction, B and the menu chord change nothing and the game still starts on time.
        [Fact]
        public Task The_pad_does_nothing_while_the_screen_is_up() => Session.Dispatch(() =>
        {
            using var s = Open(BigPictureInterface.LaunchNormal);
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.A();
            s.Run(Hold / 2);
            s.Pad.Down();
            s.Pad.B();
            s.Pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
            s.Pad.Start();
            Assert.False(s.Window.GetControl<Control>("PadMenuPanel").IsVisible);
            Assert.Equal(Aurora, s.Game);
            Assert.Equal("gamelist", s.View);
            Assert.Null(Running(s.Window));
            s.Run(2500);
            Assert.Equal(Rom(s, Aurora), Running(s.Window));
            Assert.False(s.Window.GetControl<Control>("PadMenuPanel").IsVisible);
            Stop(s.Window);
        }, default);

        // §21.3's risk: the pad menu's way back to a suspended game, and East from the system view, go straight back to it.
        [Fact]
        public Task A_resume_from_the_pad_menu_or_East_is_not_delayed() => Session.Dispatch(() =>
        {
            using var s = Open(BigPictureInterface.LaunchLong);
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.A();
            s.Run(4500);
            Assert.True(GameShown(s));

            ThemedLibraryFlowTests.Choose(s, "Back to Library");
            Assert.True(s.Shown);
            ThemedCollectionsTests.Choose(s, MainWindow.ContinueRow);
            Assert.True(GameShown(s));
            Assert.False(s.Window.IsPaused);
            Assert.Null(Screen(s));

            ThemedLibraryFlowTests.Choose(s, "Back to Library");
            s.Run(200);
            s.Pad.B();
            Assert.Equal("system", s.View);
            s.Pad.B();
            Assert.True(GameShown(s));
            Assert.Null(Screen(s));
            Stop(s.Window);
        }, default);

        private static SheetLayer Sheets(ThemedSession s) => s.Window.GetControl<SheetLayer>("Sheets");

        // Q33: the question first, then the screen after either answer; a question cancelled shows nothing and starts nothing.
        [Fact]
        public Task The_screen_follows_the_resume_question_and_a_cancelled_question_shows_nothing() => Session.Dispatch(() =>
        {
            using var s = Open(BigPictureInterface.LaunchNormal, more: a =>
            {
                a.ResumeOnLaunch = AppSettings.ResumeAsk;
                a.StateDirectory = Path.Combine(Path.GetTempPath(), "EmuSenLaunchStates", Guid.NewGuid().ToString("N"));
            });
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.A();
            Assert.NotNull(Screen(s));
            s.Run(3000);
            ThemedLibraryFlowTests.QuitGame(s);
            s.Run(200);

            s.Pad.A();
            Assert.IsType<ResumeWindow>(Sheets(s).Current);
            Assert.Null(Screen(s));
            s.Run(3500);
            Assert.Null(Running(s.Window));
            s.Pad.B();
            Assert.False(Sheets(s).IsPresenting);
            Assert.Null(Screen(s));
            s.Run(3500);
            Assert.Null(Running(s.Window));
            Assert.True(s.Shown);

            s.Pad.A();
            Assert.IsType<ResumeWindow>(Sheets(s).Current);
            TimeSpan answered = s.Now;
            s.Pad.A();
            Assert.False(Sheets(s).IsPresenting);
            LaunchScreen screen = Assert.IsType<LaunchScreen>(Screen(s));
            Assert.Equal(answered + TimeSpan.FromMilliseconds(Hold), screen.Due);
            s.Run(3000);
            Assert.Equal(Rom(s, Aurora), Running(s.Window));
            Stop(s.Window);
        }, default);

        // A game that cannot load: the screen goes at its end, the library stays, the reason shows in the notice and is in the error log.
        [Fact]
        public Task A_game_that_fails_to_load_closes_the_screen_shows_the_error_and_logs_it() => Session.Dispatch(() =>
        {
            using var s = Open(BigPictureInterface.LaunchBrief);
            ThemedLibraryPadTests.Enter(s, "nes");
            string game = s.Game!;
            s.Pad.A();
            Assert.NotNull(Screen(s));
            s.Run(1700);
            Assert.Null(Screen(s));
            Assert.Null(Running(s.Window));
            Assert.False(GameShown(s));
            Assert.True(s.Shown);
            string? notice = s.Window.GetControl<NoticeLayer>("PadNotice").Current;
            Assert.NotNull(notice);
            Assert.StartsWith("Failed to load", notice);
            string log = File.ReadAllText(ErrorLog.PathFor(DateTime.Now));
            Assert.Contains($"[launch] Failed to load {game}", log);
            s.Pad.Down();
            Assert.NotEqual(game, s.Game);
        }, default);

        // Every route big picture starts a game by: a collection, the random entry and a folder link.
        [Fact]
        public Task A_collection_the_random_entry_and_a_folder_link_all_show_it() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(settings: a =>
            {
                a.BigPictureInterface.LaunchScreenDuration = BigPictureInterface.LaunchBrief;
                ThemedCollectionsTests.AllAuto(a);
            }, roms: ThemedFoldersTests.Library);

            ThemedCollectionsTests.Enter(s, "all");
            string first = s.Game!;
            s.Pad.A();
            Assert.Equal(first, Screen(s)?.GameName);
            s.Run(1700);
            Assert.NotNull(Running(s.Window));
            ThemedLibraryFlowTests.QuitGame(s);

            s.Pad.A();
            s.Pad.Press(EmuSen.Mistress.Input.UiButton.Random);
            s.Settle();
            string random = s.Game!;
            s.Pad.A();
            Assert.Equal(random, Screen(s)?.GameName);
            s.Run(1700);
            ThemedLibraryFlowTests.QuitGame(s);

            string racing = Path.Combine(s.RomDirectory, "SNES", "Racing");
            ThemedCollectionsTests.Records(s).SaveEdits(racing, new Dictionary<string, string?> { [EmuSen.Mistress.Library.GameMetadata.FolderLink] = ThemedFoldersTests.VelvetRally + ".sfc" }, DateTime.Now);
            ThemedCollectionsTests.Refresh(s);
            for (int guard = 0; guard < 6 && s.View != "system"; guard++) s.Pad.B();
            ThemedCollectionsTests.Enter(s, "snes");
            Assert.Equal("Racing", s.Game);
            s.Pad.A();
            Assert.Equal(ThemedFoldersTests.VelvetRally, Screen(s)?.GameName);
            s.Run(1700);
            Assert.Equal(Path.Combine(racing, ThemedFoldersTests.VelvetRally + ".sfc"), Running(s.Window));
            Stop(s.Window);
        }, default);

        private static (byte R, byte G, byte B) At(RenderedFrame f, double x, double y)
        {
            int i = ((int)y * f.Width + (int)x) * 4;
            return (f.Rgba[i], f.Rgba[i + 1], f.Rgba[i + 2]);
        }

        private static Rect InWindow(ThemedSession s, Rect r)
        {
            Point o = Host(s).TranslatePoint(default, s.Window)!.Value;
            return r.Translate(new Vector(o.X, o.Y));
        }

        private static int DifferingOutside(RenderedFrame a, RenderedFrame b, Rect keep)
        {
            int n = 0;
            for (int y = 0; y < a.Height; y++)
                for (int x = 0; x < a.Width; x++)
                {
                    if (keep.Contains(new Point(x + 0.5, y + 0.5))) continue;
                    int i = (y * a.Width + x) * 4;
                    if (a.Rgba[i] != b.Rgba[i] || a.Rgba[i + 1] != b.Rgba[i + 1] || a.Rgba[i + 2] != b.Rgba[i + 2]) n++;
                }
            return n;
        }

        // Outside a box: how many pixels were not black before, and how many of those the shade and blur changed.
        private static (int Lit, int Changed) LitAndChanged(RenderedFrame before, RenderedFrame after, Rect keep)
        {
            int lit = 0, changed = 0;
            for (int y = 0; y < before.Height; y++)
                for (int x = 0; x < before.Width; x++)
                {
                    if (keep.Contains(new Point(x + 0.5, y + 0.5))) continue;
                    int i = (y * before.Width + x) * 4;
                    if (before.Rgba[i] + before.Rgba[i + 1] + before.Rgba[i + 2] == 0) continue;
                    lit++;
                    if (before.Rgba[i] != after.Rgba[i] || before.Rgba[i + 1] != after.Rgba[i + 1] || before.Rgba[i + 2] != after.Rgba[i + 2]) changed++;
                }
            return (lit, changed);
        }

        private static readonly Color Panel = MenuPanel.PanelColorProperty.GetDefaultValue(typeof(MenuPanel));

        private static (byte, byte, byte) Rgb(Color c) => (c.R, c.G, c.B);

        // EmuSen's own look on a stepped clock, with the media folder of Open.
        internal static ThemedSession OwnLook(double width = 1280, double height = 800, Action<string>? roms = null) =>
            Open(BigPictureInterface.LaunchNormal, width, height, a => a.LibraryStyle = AppSettings.LibraryStyleMistress, roms);

        private static void PlayOwn(ThemedSession s, string game)
        {
            EmuSen.WiseMan.Fixtures.OwnLook.Play(s.Window, s.Pad, game);
            s.Settle();
        }

        // The own look shows it: the name, the console and engine, the marquee large in a 480 by 240 box with its aspect kept, and the game only after Pan.
        [Fact]
        public Task The_own_look_shows_it_with_the_marquee_large_and_starts_after_Pan() => Session.Dispatch(() =>
        {
            using var s = OwnLook();
            PlayOwn(s, Rom(s, Aurora));
            LaunchScreen screen = Assert.IsType<LaunchScreen>(Screen(s));
            Assert.Equal(Aurora, screen.GameName);
            Assert.StartsWith("Super Nintendo · ", screen.SystemName);
            Assert.Equal(MainWindow.LoadingLine, screen.Line);
            Assert.True(screen.ShowsMarquee);
            Assert.Equal(Path.Combine(s.Settings.EsdeMediaDirectory!, "snes", "marquees", Aurora + ".png"), screen.ArtPath);
            Assert.Equal(new Size(480, 180), screen.ArtSize);
            Rect art = screen.ArtControl.Bounds;
            Assert.Equal(800.0 / 300, art.Width / art.Height, 2);
            Assert.Null(Running(s.Window));
            s.Run(Hold - 16);
            Assert.Null(Running(s.Window));
            s.Run(16);
            Assert.Equal(Rom(s, Aurora), Running(s.Window));
            Assert.Null(Screen(s));
            Stop(s.Window);
        }, default);

        // With no marquee the cover stands there, 240 wide, its aspect kept.
        [Fact]
        public Task The_own_look_falls_back_to_the_cover() => Session.Dispatch(() =>
        {
            using var s = OwnLook();
            PlayOwn(s, Rom(s, Brass));
            LaunchScreen screen = Assert.IsType<LaunchScreen>(Screen(s));
            Assert.False(screen.ShowsMarquee);
            Assert.Equal(Path.Combine(s.Settings.EsdeMediaDirectory!, "snes", "covers", Brass + ".png"), screen.ArtPath);
            Assert.Equal(new Size(240, 320), screen.ArtSize);
            // The fixture's covers share one crimson and orange picture, whose mean is 240, 113, 58.
            Assert.InRange(screen.Hue.R, 230, 250);
            Assert.InRange(screen.Hue.G, 105, 121);
            Assert.InRange(screen.Hue.B, 50, 66);
            s.Run(Hold);
            Assert.Equal(Rom(s, Brass), Running(s.Window));
            Stop(s.Window);
        }, default);

        // The screen is held past Pan until the game is loaded, then the frame.
        [Fact]
        public Task The_own_look_holds_it_until_the_game_is_loaded() => Session.Dispatch(() =>
        {
            string bin = "";
            using var s = OwnLook(roms: r =>
            {
                Directory.CreateDirectory(Path.Combine(r, "psx"));
                bin = Path.Combine(r, "psx", "Opal Wait.bin");
                File.WriteAllBytes(bin, SyntheticPsxDisc.Build().DataTrack);
            });
            using var gate = new System.Threading.ManualResetEventSlim();
            using var loaded = new System.Threading.ManualResetEventSlim();
            var open = MistressGame.OwnOpen(s.Window);
            MistressGame.EarlyLoad(s.Window, what => { gate.Wait(); try { return open(what); } finally { loaded.Set(); } });
            PlayOwn(s, bin);
            Assert.NotNull(Screen(s));
            s.Run(Hold + 160);
            Assert.Null(Running(s.Window));
            Assert.NotNull(Screen(s));
            gate.Set();
            Assert.True(loaded.Wait(TimeSpan.FromSeconds(30)));
            Pump();
            Assert.Equal(bin, Running(s.Window));
            Assert.True(GameShown(s));
            Assert.Null(Screen(s));
            Stop(s.Window);
        }, default);

        // The desktop's list shows the cover where a game has no marquee; a state chosen on a page names its slot.
        [Fact]
        public Task The_desktop_shows_it_with_the_cover_where_there_is_no_marquee() => Session.Dispatch(() =>
        {
            using var s = Open(BigPictureInterface.LaunchNormal, more: a => a.BigScreen = false);
            EmuSen.WiseMan.Fixtures.OwnLook.Select(s.Window, Rom(s, Brass));
            typeof(MainWindow).GetMethod("LaunchSelectedLibraryEntry", Hidden)!.Invoke(s.Window, null);
            s.Settle();
            LaunchScreen screen = Assert.IsType<LaunchScreen>(Screen(s));
            Assert.Equal(Brass, screen.GameName);
            Assert.False(screen.ShowsMarquee);
            Assert.Equal(new Size(240, 320), screen.ArtSize);
            s.Run(Hold);
            Assert.Equal(Rom(s, Brass), Running(s.Window));
            Stop(s.Window);
        }, default);

        // The group sits on the mockup's line, 330 units down, art then words, scaled with the window.
        [Theory]
        [InlineData(1280, 800)]
        [InlineData(1920, 1080)]
        public Task The_art_and_the_words_sit_where_the_mockup_draws_them(int w, int h) => Session.Dispatch(() =>
        {
            using var s = OwnLook(w, h);
            PlayOwn(s, Rom(s, Brass));
            s.Run(Hold / 2);
            LaunchScreen screen = Screen(s)!;
            double unit = Math.Min(w / 1280.0, h / 800.0);
            Rect art = new(screen.ArtControl.TranslatePoint(default, s.Window)!.Value, screen.ArtControl.Bounds.Size * unit);
            Assert.Equal(LaunchScreen.CentreY * unit + (h - 800 * unit) / 2, art.Center.Y, 0);
            Assert.Equal(240 * unit, art.Width, 0);
            Assert.InRange(screen.Progress, 0.2, 0.4);
            s.Run(Hold);
            Stop(s.Window);
        }, default);

        // Under Reduced motion the bar stands at its value from the start.
        [Fact]
        public Task Under_reduced_motion_the_bar_does_not_move() => Session.Dispatch(() =>
        {
            using var s = OwnLook();
            EmuSen.LunaP.Theme.LunaMotion.Reduced = true;
            try
            {
                PlayOwn(s, Rom(s, Aurora));
                LaunchScreen screen = Screen(s)!;
                Assert.Equal(LaunchScreen.BarShown, screen.Progress);
                Assert.False(screen.Moving(s.Now));
                s.Run(Hold);
                Assert.Equal(Rom(s, Aurora), Running(s.Window));
            }
            finally { EmuSen.LunaP.Theme.LunaMotion.Reduced = false; }
            Stop(s.Window);
        }, default);

        // A disc game from the own look starts once the hold is over with the window's own loop running and no pad tick, so a screen held for a disc is released.
        [Fact]
        public Task A_disc_game_starts_once_the_hold_is_over() => Session.Dispatch(() =>
        {
            string bin = "";
            using var s = OwnLook(roms: r =>
            {
                Directory.CreateDirectory(Path.Combine(r, "psx"));
                bin = Path.Combine(r, "psx", "Opal Plain.bin");
                File.WriteAllBytes(bin, SyntheticPsxDisc.Build().DataTrack);
            });
            PlayOwn(s, bin);
            Assert.NotNull(Screen(s));
            s.Now += TimeSpan.FromMilliseconds(Hold);
            // The dispatcher's own loop for the hold and a second more, so its timers and frames run (LunaP §88.7).
            var loop = new Avalonia.Threading.DispatcherFrame();
            var stop = new Avalonia.Threading.DispatcherTimer { Interval = TimeSpan.FromMilliseconds(Hold + 1000) };
            stop.Tick += (_, _) => { stop.Stop(); loop.Continue = false; };
            stop.Start();
            Avalonia.Threading.Dispatcher.UIThread.PushFrame(loop);
            Assert.Equal(bin, Running(s.Window));
            Assert.True(GameShown(s));
            Assert.Null(Screen(s));
            Stop(s.Window);
        }, default);

        // Start Console shows the console with no game's name and no picture.
        [Fact]
        public void Start_Console_s_card_names_the_console_and_no_game()
        {
            string console = EmuSen.Mistress.Library.RuntimeCatalog.ConsolesThatStartEmpty.FirstOrDefault() ?? "PlayStation";
            LaunchCard card = MainWindow.LaunchCardForConsole(console);
            Assert.Null(card.Name);
            Assert.Null(card.Art);
            Assert.Equal(MainWindow.NoDiscLine, card.Line);
        }

        // A window closed while the screen shows starts nothing afterwards, when its timer would have come due (§15.14's lesson).
        [Fact]
        public Task Closing_the_window_while_the_screen_shows_starts_nothing() => Session.Dispatch(() =>
        {
            using var s = Open(BigPictureInterface.LaunchNormal);
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.A();
            Assert.NotNull(Screen(s));
            s.Window.Close();
            s.Now += TimeSpan.FromSeconds(4);
            typeof(MainWindow).GetMethod("AdvanceLaunchScreen", Hidden)!.Invoke(s.Window, null);
            Avalonia.Threading.Dispatcher.UIThread.RunJobs();
            Assert.Null(Screen(s));
            Assert.Null(Running(s.Window));
        }, default);

        // The desktop's list and desktop big picture (F10) both show it.
        [Fact]
        public Task F10_big_picture_and_the_desktop_list_both_show_it() => Session.Dispatch(() =>
        {
            using var s = Open(BigPictureInterface.LaunchNormal, more: a => a.BigScreen = false);
            Assert.False(s.Shown);
            s.Window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
            typeof(MainWindow).GetMethod("LaunchSelectedLibraryEntry", Hidden)!.Invoke(s.Window, null);
            s.Settle();
            Assert.NotNull(Screen(s));
            Assert.Null(Running(s.Window));
            s.Run(Hold);
            Assert.NotNull(Running(s.Window));
            Assert.True(GameShown(s));
            Assert.Null(Screen(s));
            Stop(s.Window);
            typeof(MainWindow).GetMethod("ShowLibrary", Hidden)!.Invoke(s.Window, null);
            s.Settle();

            BigPictureSwitchTests.Press(s, Avalonia.Input.Key.F10);
            Assert.True(s.Shown);
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.A();
            Assert.Equal(Aurora, Screen(s)?.GameName);
            Assert.Null(Running(s.Window));
            s.Run(3000);
            Assert.Equal(Rom(s, Aurora), Running(s.Window));
            Stop(s.Window);
        }, default);

        // The row on Theme Settings' Interface tab writes appsettings.json, and the next start reads it.
        [Fact]
        public Task The_Interface_tab_s_row_stores_the_setting() => Session.Dispatch(() =>
        {
            using var s = Open(BigPictureInterface.LaunchNormal);
            ThemedSwitchesTests.Choose(s, "LaunchScreenDuration", "Disabled");
            s.Run(200);
            Assert.Equal(BigPictureInterface.LaunchDisabled, s.Settings.BigPictureInterface.LaunchScreenDuration);
            Assert.Equal(BigPictureInterface.LaunchDisabled, AppSettings.Load().BigPictureInterface.LaunchScreenDuration);
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.A();
            Assert.Null(Screen(s));
            Assert.Equal(Rom(s, Aurora), Running(s.Window));
            Stop(s.Window);
        }, default);
    }

    internal static class VisualQueries
    {
        public static IEnumerable<T> GetVisualDescendantsOf<T>(this Visual v) => Avalonia.VisualTree.VisualExtensions.GetVisualDescendants(v).OfType<T>().Where(c => c is Visual { IsVisible: true });
    }
}
