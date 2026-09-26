using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net;
using System.Net.Http;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Interactivity;
using Avalonia.LogicalTree;
using Avalonia.Threading;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;
using Avalonia.VisualTree;
using EmuSen.Mistress.Scraping;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // A window over the fake ScreenScraper, with the confirm steps answered by the test - shared by the status window's and the sign-in's tests.
    public abstract class ScrapeWindowFixture : IDisposable
    {
        protected static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ScrapeWindowFixture).GetTypeInfo().Assembly);

        protected const BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;

        private static readonly FieldInfo ConfirmField = typeof(MainWindow).GetField("ConfirmScrape", BindingFlags.Static | BindingFlags.NonPublic)!;
        private static readonly FieldInfo CancelField = typeof(MainWindow).GetField("ConfirmCancelScrape", BindingFlags.Static | BindingFlags.NonPublic)!;

        protected readonly string Root = Path.Combine(Path.GetTempPath(), "EmuSenScrapeStatusTests", Guid.NewGuid().ToString("N"));
        protected readonly FakeScreenScraper Server = new();
        protected readonly FakeScrapeClock Clock = new(new DateTimeOffset(2026, 7, 14, 10, 0, 0, TimeSpan.Zero));
        protected readonly string RomDir;
        protected readonly List<Window> Windows = new();
        protected readonly ITestOutputHelper Out;
        private readonly object _realConfirm = ConfirmField.GetValue(null)!;
        private readonly object _realCancel = CancelField.GetValue(null)!;
        protected bool CancelAnswer = true;
        protected int CancelAsked;
        private int _released;

        protected ScrapeWindowFixture(ITestOutputHelper output)
        {
            Out = output;
            RomDir = Path.Combine(Root, "Roms");
            Directory.CreateDirectory(RomDir);
            ConfigStore.OverrideDirectory = Path.Combine(Root, "Config");
            DataStore.OverrideDirectory = Path.Combine(Root, "Home");
            NoNetwork.HttpFactory.SetValue(null, (Func<HttpClient>)(() => new HttpClient(Server)));
            NoNetwork.ScrapeClock.SetValue(null, Clock);
            ConfirmField.SetValue(null, (Func<MainWindow, string, string, Task<bool>>)((_, _, _) => Task.FromResult(true)));
            CancelField.SetValue(null, (Func<Window, Task<bool>>)(_ => { CancelAsked++; return Task.FromResult(CancelAnswer); }));
            OnlineCoverTests.BuildOpenVgdb(OpenVgdb.DefaultPath, ("SNES", "F-Zero (USA)", "00", null));
            Server.Other = url => url.Contains("thumbnails.libretro.com") ? OnlineCoverTests.FakeServer.Png() : new HttpResponseMessage(HttpStatusCode.NotFound);
        }

        public void Dispose()
        {
            ConfirmField.SetValue(null, _realConfirm);
            CancelField.SetValue(null, _realCancel);
            NoNetwork.Refuse();
            NoNetwork.ScrapeClock.SetValue(null, SystemScrapeClock.Instance);
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(Root, recursive: true); } catch { }
        }

        // A failed assertion still closes what the test opened, so nothing it started runs on into the next test.
        protected Task OnUi(Action body) => Session.Dispatch(() =>
        {
            try { body(); }
            finally
            {
                Server.Gate?.Release(1000);
                foreach (Window w in Windows) if (w.IsVisible) w.Close();
            }
        }, default);

        protected static void SetDeveloper(bool present) =>
            NoNetwork.DeveloperSource.SetValue(null, present ? (Func<DeveloperCredentials?>)(() => FakeScreenScraper.Developer) : NoNetwork.NoDeveloper);

        protected MainWindow Open(bool developer = true, Action<AppSettings>? settings = null, bool bigScreen = false)
        {
            SetDeveloper(developer);
            var app = new AppSettings { RomDirectory = RomDir, LibraryView = AppSettings.LibraryList, BigScreen = bigScreen, ResumeOnLaunch = AppSettings.ResumeNever };
            settings?.Invoke(app);
            app.Save();
            var window = new MainWindow { Width = 1280, Height = 800 };
            Windows.Add(window);
            window.Show();
            return window;
        }

        // A synthetic game of its own bytes, so each has its own MD5.
        protected string Game(string name, byte seed)
        {
            string path = Path.Combine(RomDir, name + ".sfc");
            File.WriteAllBytes(path, SyntheticRom.Build((0x100, [seed, (byte)(seed + 1), 7])));
            return path;
        }

        protected static string Md5(string path) => RomHashes.Of(path).Md5;

        protected static void WaitFor(Func<bool> condition, string what)
        {
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (!condition())
            {
                Dispatcher.UIThread.RunJobs();
                if (clock.ElapsedMilliseconds > 20_000) Assert.Fail("timed out waiting for " + what);
                Thread.Sleep(5);
            }
        }

        // Real time on the dispatcher, timers included; RunJobs alone runs no DispatcherTimer under the headless platform.
        protected static void Pump(int ms = 800)
        {
            var frame = new DispatcherFrame();
            DispatcherTimer.RunOnce(() => frame.Continue = false, TimeSpan.FromMilliseconds(ms));
            Dispatcher.UIThread.PushFrame(frame);
        }

        // The dispatcher run with its timers until the condition holds.
        protected static void PumpUntil(Func<bool> condition, string what, int ms = 20_000)
        {
            var watch = System.Diagnostics.Stopwatch.StartNew();
            var frame = new DispatcherFrame();
            var check = new DispatcherTimer(TimeSpan.FromMilliseconds(5), DispatcherPriority.Background, (_, _) => { if (condition() || watch.ElapsedMilliseconds > ms) frame.Continue = false; });
            check.Start();
            Dispatcher.UIThread.PushFrame(frame);
            check.Stop();
            if (!condition()) Assert.Fail("timed out waiting for " + what);
        }

        // Lets held requests through one at a time until the condition holds; a result is posted before its worker's next request, so the jobs are run before each release.
        protected void ReleaseUntil(Func<bool> condition, string what)
        {
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (true)
            {
                Dispatcher.UIThread.RunJobs();
                if (condition()) break;
                if (Server.Asked.Count > _released)
                {
                    Dispatcher.UIThread.RunJobs();
                    if (condition()) break;
                    _released++;
                    Server.Gate!.Release();
                }
                if (clock.ElapsedMilliseconds > 20_000) Assert.Fail($"timed out waiting for {what}: {Server.Asked.Count} asked, {_released} released, {Server.Gate!.CurrentCount} free; last {Server.Asked.LastOrDefault()}");
                Thread.Sleep(5);
            }
        }

        // Holds every request from now until released, counting from what was asked before.
        protected SemaphoreSlim Hold()
        {
            _released = Server.Asked.Count;
            return Server.Gate = new SemaphoreSlim(0);
        }

        protected static void Scrape(MainWindow w, ScrapeScope scope) => Assert.True(w.ConfirmAndScrapeAsync(scope).GetAwaiter().GetResult());

        protected static void RunEnds(MainWindow w) => WaitFor(() => w.Progress is { State: not ScrapeRunState.Running }, "the run to end");

        protected static Control RootOf(Window window) => SheetLayer.PresenterOf(window)?.SheetOf(window) ?? window;

        protected static T Named<T>(Window window, string name) where T : Control =>
            RootOf(window).GetLogicalDescendants().OfType<T>().FirstOrDefault(c => c.Name == name) ?? throw new InvalidOperationException($"no {typeof(T).Name} {name}");

        protected static string TextOf(Window window, string name) => Named<TextBlock>(window, name).Text ?? "";

        protected static void Click(Button button) => button.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));

        protected static SheetLayer Sheets(MainWindow w) => w.GetControl<SheetLayer>("Sheets");

        protected static List<PadMenuEntry> PadMenu(MainWindow w) => (List<PadMenuEntry>)typeof(MainWindow).GetField("_padMenuEntries", Hidden)!.GetValue(w)!;

        protected static void Invoke(MainWindow w, string method) => typeof(MainWindow).GetMethod(method, Hidden, Type.EmptyTypes)!.Invoke(w, null);

        protected static Delegate? ScrapeChangedHandlers(MainWindow w) => (Delegate?)typeof(MainWindow).GetField("ScrapeChanged", Hidden)!.GetValue(w);

        // Every text the window shows, the recent list's rows included.
        protected static string AllText(ScrapeStatusWindow status) =>
            string.Join("\n", RootOf(status).GetLogicalDescendants().OfType<TextBlock>().Select(t => t.Text)
                .Concat(Named<LunaList<ScrapeRecent>>(status, "ScrapeStatusRecent").Models.Select(ScrapeStatusWindow.Describe))
                .Append(Named<MeterRow>(status, "ScrapeStatusQuota").ValueText));
    }

    // The status window: it opens with a run, follows it as the fake server answers, says why it stopped, and lets go of everything when closed - see EmuSen_Settings_Reference.md §4.57.
    [Collection(TestCollections.ProcessGlobals)]
    public class ScrapeStatusWindowTests : ScrapeWindowFixture
    {
        public ScrapeStatusWindowTests(ITestOutputHelper output) : base(output) { }

        private (string Found, string Unknown, string Broken) ThreeGames()
        {
            string found = Game("A Found (USA)", 10), unknown = Game("B Unknown (USA)", 20), broken = Game("C Broken (USA)", 30);
            Server.Games.Add(new FakeGame(77, "Found", Md5(found)));
            Server.StatusByMd5[Md5(broken)] = (400, "Erreur : requete malformee devpassword=FAKEDEVPASSWORD&sspassword=FAKEMEMBERSECRET");
            return (found, unknown, broken);
        }

        [Fact]
        public Task A_run_opens_the_window_and_it_follows_progress_the_current_game_tallies_and_the_quota() => OnUi(() =>
        {
            (string found, _, _) = ThreeGames();
            Server.Gate = new SemaphoreSlim(0);
            MainWindow window = Open(settings: a => a.OpenEmuFallback = false);
            Assert.Null(window.ScrapeStatusShown);
            Scrape(window, new ScrapeScope());

            ScrapeStatusWindow status = Assert.IsType<ScrapeStatusWindow>(window.ScrapeStatusShown);
            Assert.Same(status, window.OwnedWindows.OfType<ScrapeStatusWindow>().Single());
            WaitFor(() => Server.Asked.Count == 1, "the first lookup");
            WaitFor(() => window.Progress!.Current is not null, "the worker's step");
            status.Refresh();
            Assert.Equal("Scraping 0 of 3 games", TextOf(status, "ScrapeStatusHeading"));
            Assert.Equal("A Found (USA)", TextOf(status, "ScrapeStatusGame"));
            Assert.Equal("Looking it up", TextOf(status, "ScrapeStatusStep"));
            Assert.Contains("estimated after the first game", TextOf(status, "ScrapeStatusTiming"));

            ReleaseUntil(() => window.Progress!.Current is { Step: ScrapeStep.Downloading }, "a download");
            status.Refresh();
            Assert.StartsWith("Downloading its ", TextOf(status, "ScrapeStatusStep"));

            ReleaseUntil(() => window.Progress!.Done == 1, "the first game");
            status.Refresh();
            Assert.Equal("Scraping 1 of 3 games", TextOf(status, "ScrapeStatusHeading"));
            Assert.Equal(1.0 / 3, Named<ProgressBar>(status, "ScrapeStatusProgress").Value, 3);
            Assert.Contains("left at this run's pace", TextOf(status, "ScrapeStatusTiming"));
            Assert.Equal("A Found (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image",
                ScrapeStatusWindow.Describe(Named<LunaList<ScrapeRecent>>(status, "ScrapeStatusRecent").Models.Single()));
            Assert.EndsWith(Path.Combine("snes", "miximages", "A Found (USA).png"), window.Progress!.LastPicture);

            ReleaseUntil(() => window.Progress!.State != ScrapeRunState.Running, "the run's end");
            Pump(300);
            Assert.True(status.IsVisible);
            Assert.Equal("Finished: 3 of 3 games", TextOf(status, "ScrapeStatusHeading"));
            Assert.Equal("Found 1 · Not found 1 · Failed 1 · Skipped 0 · Filled by OpenEmu 0", TextOf(status, "ScrapeStatusTallies"));
            Assert.StartsWith("Last failure: BadRequest", TextOf(status, "ScrapeStatusFailure"));
            Assert.Equal("No member account: EmuSen's developer credentials alone, with their limits.", TextOf(status, "ScrapeStatusMember"));
            QuotaSnapshot q = ((IScrapeHost)window).Quota!;
            Assert.Equal($"{q.RequestsToday:N0} of 20,000 requests used today · {20000 - q.RequestsToday:N0} left", TextOf(status, "ScrapeStatusRequests"));
            Assert.Equal("0%", Named<MeterRow>(status, "ScrapeStatusQuota").ValueText);
            Assert.Equal("1 thread · download limit 128 KB/s · 20,000 requests a day · 60 a minute", TextOf(status, "ScrapeStatusLimits"));
            Assert.Equal(3, Named<LunaList<ScrapeRecent>>(status, "ScrapeStatusRecent").Models.Count);
        });

        [Fact]
        public Task A_game_the_failover_fills_is_counted_and_its_row_says_so() => OnUi(() =>
        {
            string unknown = Game("F-Zero (USA)", 20);
            MainWindow window = Open();
            Scrape(window, ScrapeScope.ThisGame(unknown));
            RunEnds(window);
            ScrapeStatusWindow status = window.ScrapeStatusShown!;
            status.Refresh();
            Assert.Contains("Not found 1", TextOf(status, "ScrapeStatusTallies"));
            Assert.EndsWith("Filled by OpenEmu 1", TextOf(status, "ScrapeStatusTallies"));
            Assert.EndsWith("not found · cover from OpenEmu's sources", ScrapeStatusWindow.Describe(Named<LunaList<ScrapeRecent>>(status, "ScrapeStatusRecent").Models.Single()));
        });

        // The day used up (the 2% rule and 430), the service closed or refusing (423, 426, 403), each in its own words.
        [Theory]
        [InlineData(0, "today's requests are nearly used up (")]
        [InlineData(430, "today's requests are used up (430)")]
        [InlineData(423, "ScreenScraper's API is closed (423)")]
        [InlineData(426, "blocked this version of the software (426)")]
        [InlineData(403, "refused EmuSen's developer credentials (403)")]
        public Task A_quota_stop_says_plainly_why_and_that_the_rest_is_left_for_resume(int code, string why) => OnUi(() =>
        {
            string first = Game("A First (USA)", 10);
            Game("B Second (USA)", 20);
            Server.Games.Add(new FakeGame(77, "First", Md5(first)));
            if (code == 0) Server.User = FakeScreenScraper.Quota(maxThreads: 1, perMinute: 60, perDay: 100, koPerDay: 10, today: 98, koToday: 0);
            else Server.ForcedStatus = code;
            MainWindow window = Open(settings: a => a.OpenEmuFallback = false);
            Scrape(window, new ScrapeScope());
            RunEnds(window);
            ScrapeStatusWindow status = window.ScrapeStatusShown!;
            status.Refresh();
            Assert.StartsWith("Stopped after", TextOf(status, "ScrapeStatusHeading"));
            Assert.Contains(why, TextOf(status, "ScrapeStatusWhy"));
            Assert.StartsWith("Why it stopped: ", TextOf(status, "ScrapeStatusWhy"));
            Assert.Contains("left queued: Resume, in Preferences ▸ Scraping, goes on with them. Nothing resumes by itself.", TextOf(status, "ScrapeStatusSummary"));
            Assert.Equal("Close", Named<Button>(status, "ScrapeStatusHideButton").Content);
            Assert.False(Named<Button>(status, "ScrapeStatusCancelButton").IsVisible);
        });

        [Fact]
        public Task Without_the_developer_file_the_window_says_so_as_the_reason() => OnUi(() =>
        {
            string game = Game("A First (USA)", 10);
            MainWindow window = Open(developer: false, settings: a => a.OpenEmuFallback = false);
            Scrape(window, ScrapeScope.ThisGame(game));
            ScrapeStatusWindow status = window.ScrapeStatusShown!;
            status.Refresh();
            Assert.Equal("Why it stopped: EmuSen's developer file is not on this computer.", TextOf(status, "ScrapeStatusWhy"));
            Assert.Empty(Server.Asked);
        });

        [Fact]
        public Task Hide_closes_the_window_and_the_run_carries_on_in_the_status_line() => OnUi(() =>
        {
            ThreeGames();
            Server.Gate = new SemaphoreSlim(0);
            MainWindow window = Open(settings: a => a.OpenEmuFallback = false);
            Scrape(window, new ScrapeScope());
            ScrapeStatusWindow status = window.ScrapeStatusShown!;
            Assert.Equal("Hide", Named<Button>(status, "ScrapeStatusHideButton").Content);
            Click(Named<Button>(status, "ScrapeStatusHideButton"));
            Assert.False(status.IsVisible);
            Assert.Null(window.ScrapeStatusShown);
            Assert.True(window.ScrapeRunning);

            ReleaseUntil(() => window.Progress!.Done == 1, "a game after hiding");
            Assert.Equal("Scraping: 1 of 3.", window.GetControl<TextBlock>("StatusText").Text);
            ReleaseUntil(() => !window.ScrapeRunning, "the run's end");
            Assert.Equal(3, Server.JeuInfos.Count());
            Assert.Null(window.ScrapeStatusShown);
        });

        [Fact]
        public Task Cancel_asks_first_then_stops_and_leaves_the_rest_queued() => OnUi(() =>
        {
            ThreeGames();
            Server.Gate = new SemaphoreSlim(0);
            MainWindow window = Open(settings: a => a.OpenEmuFallback = false);
            Scrape(window, new ScrapeScope());
            ScrapeStatusWindow status = window.ScrapeStatusShown!;
            WaitFor(() => Server.Asked.Count == 1, "the first lookup");

            CancelAnswer = false;
            Click(Named<Button>(status, "ScrapeStatusCancelButton"));
            Pump(100);
            Assert.Equal(1, CancelAsked);
            Assert.True(window.ScrapeRunning);

            CancelAnswer = true;
            Click(Named<Button>(status, "ScrapeStatusCancelButton"));
            Pump(100);
            Assert.Equal(2, CancelAsked);
            int asked = Server.Asked.Count;
            Server.Gate.Release(100);
            Pump();
            Assert.Equal(asked, Server.Asked.Count);
            Assert.Equal(ScrapeRunState.Cancelled, window.Progress!.State);
            Assert.Equal(3, window.Interrupted);
            status.Refresh();
            Assert.Equal("Cancelled after 0 of 3 games", TextOf(status, "ScrapeStatusHeading"));
            Assert.Equal("Why it stopped: you cancelled it.", TextOf(status, "ScrapeStatusWhy"));
            Assert.Contains("3 games left queued", TextOf(status, "ScrapeStatusSummary"));
        });

        [Fact]
        public Task Pause_holds_every_request_until_resume_and_the_run_then_finishes() => OnUi(() =>
        {
            ThreeGames();
            Server.Gate = new SemaphoreSlim(0);
            MainWindow window = Open(settings: a => a.OpenEmuFallback = false);
            Scrape(window, new ScrapeScope());
            ScrapeStatusWindow status = window.ScrapeStatusShown!;
            WaitFor(() => Server.Asked.Count == 1, "the first lookup");

            Button pause = Named<Button>(status, "ScrapeStatusPauseButton");
            Assert.Equal("Pause", pause.Content);
            Click(pause);
            Assert.Equal("Resume", pause.Content);
            Assert.True(window.ScrapePaused);
            Assert.Equal("Scraping paused at 0 of 3.", window.GetControl<TextBlock>("StatusText").Text);

            Server.Gate.Release(100);
            Pump();
            Assert.Equal(1, Server.Asked.Count);
            Assert.True(window.ScrapeRunning);
            status.Refresh();
            Assert.Equal("Paused at 0 of 3 games", TextOf(status, "ScrapeStatusHeading"));

            Click(pause);
            Assert.Equal("Pause", pause.Content);
            RunEnds(window);
            Assert.Equal(ScrapeRunState.Done, window.Progress!.State);
            Assert.Equal(3, Server.JeuInfos.Count());
        });

        [Fact]
        public Task The_summary_stays_after_the_end_until_closed_and_reopens_from_the_status_line() => OnUi(() =>
        {
            ThreeGames();
            MainWindow window = Open(settings: a => a.OpenEmuFallback = false);
            Scrape(window, new ScrapeScope());
            RunEnds(window);
            Pump(400);
            ScrapeStatusWindow status = window.ScrapeStatusShown!;
            Assert.True(status.IsVisible);
            Assert.StartsWith("Took ", TextOf(status, "ScrapeStatusTiming"));
            Assert.Equal($"1 found, 1 not found, 1 failed, 0 skipped, 0 covers from OpenEmu's sources. 7 requests sent to ScreenScraper; it counts {((IScrapeHost)window).Quota!.RequestsToday:N0} of 20,000 today.",
                TextOf(status, "ScrapeStatusSummary"));
            Assert.Equal("Close", Named<Button>(status, "ScrapeStatusHideButton").Content);
            Click(Named<Button>(status, "ScrapeStatusHideButton"));
            Assert.Null(window.ScrapeStatusShown);

            TextBlock line = window.GetControl<TextBlock>("StatusText");
            Assert.StartsWith("Scraped 3 of 3", line.Text);
            Avalonia.Point at = line.TranslatePoint(new Avalonia.Point(4, 4), window)!.Value;
            window.MouseDown(at, Avalonia.Input.MouseButton.Left);
            window.MouseUp(at, Avalonia.Input.MouseButton.Left);
            Assert.Equal("Finished: 3 of 3 games", TextOf(window.ScrapeStatusShown!, "ScrapeStatusHeading"));
        });

        [Fact]
        public Task Opening_the_window_with_no_run_sends_nothing_and_resumes_nothing() => OnUi(() =>
        {
            string left = Game("A Left (USA)", 10);
            Server.Games.Add(new FakeGame(77, "Left", Md5(left)));
            using (MediaStore store = MediaStore.Open(DataStore.Media)) store.Enqueue(left, "snes", ScrapePriority.Library);
            MainWindow window = Open();
            window.ShowScrapeStatus();
            ScrapeStatusWindow status = window.ScrapeStatusShown!;
            Pump();
            Assert.Equal("No scraping in progress.", TextOf(status, "ScrapeStatusHeading"));
            Assert.False(Named<Button>(status, "ScrapeStatusPauseButton").IsVisible);
            Assert.Empty(Server.Asked);
            Assert.Equal(1, window.Interrupted);
            Assert.Null(window.Progress);

            Click(Named<Button>(status, "ScrapeStatusHideButton"));
            window.ShowScrapeStatus();
            Pump();
            Assert.Empty(Server.Asked);
        });

        // Without Stop() on Closed, the timer goes on redrawing and the host keeps the window's handler; this fails then - see EmuSen_BigPicture.md §15.14.
        [Fact]
        public Task A_closed_status_window_lets_go_of_the_run_and_stops_its_timer() => OnUi(() =>
        {
            ThreeGames();
            Server.Gate = new SemaphoreSlim(0);
            MainWindow window = Open(settings: a => a.OpenEmuFallback = false);
            Scrape(window, new ScrapeScope());
            ScrapeStatusWindow status = window.ScrapeStatusShown!;
            Pump(700);
            Assert.True(status.Polling);
            Assert.Contains(ScrapeChangedHandlers(window)!.GetInvocationList(), d => d.Target is ScrapeStatusWindow);
            int drawnWhileOpen = status.Refreshes;
            Assert.True(drawnWhileOpen >= 2, $"drawn {drawnWhileOpen} times in 0.6 s");

            status.Close();
            int drawn = status.Refreshes;
            Server.Gate.Release(1000);
            PumpUntil(() => !window.ScrapeRunning, "the run's end");
            Pump(800);
            Assert.False(status.Polling);
            Assert.Equal(drawn, status.Refreshes);
            Assert.DoesNotContain(ScrapeChangedHandlers(window)?.GetInvocationList() ?? [], d => d.Target is ScrapeStatusWindow);
        });

        // Owned desktop windows close with their owner anyway; a sheet is closed by nothing but StopScraping, so both are tested.
        [Theory]
        [InlineData(false)]
        [InlineData(true)]
        public Task Closing_Mistress_closes_the_status_window_and_its_timer(bool bigScreen) => OnUi(() =>
        {
            ThreeGames();
            Server.Gate = new SemaphoreSlim(0);
            MainWindow window = Open(settings: a => a.OpenEmuFallback = false, bigScreen: bigScreen);
            Scrape(window, new ScrapeScope());
            ScrapeStatusWindow status = window.ScrapeStatusShown!;
            Assert.Equal(bigScreen, SheetLayer.PresenterOf(status) is not null);
            window.Close();
            int drawn = status.Refreshes;
            Server.Gate.Release(100);
            Pump(600);
            Assert.False(status.Polling);
            Assert.False(status.IsVisible);
            Assert.Equal(drawn, status.Refreshes);
        });

        // A run's events arrive faster than the window draws: it draws on its timer, at most four times a second.
        [Fact]
        public Task A_fast_run_is_drawn_at_the_timer_s_pace_not_once_an_event() => OnUi(() =>
        {
            for (int i = 0; i < 40; i++)
            {
                string g = Game($"Game {i:00} (USA)", (byte)(10 + i * 3));
                Server.Games.Add(new FakeGame(100 + i, $"Game {i}", Md5(g)));
            }
            MainWindow window = Open(settings: a => { a.OpenEmuFallback = false; a.ScrapeScreenshots = a.ScrapeMarquees = a.ScrapeMiximages = false; });
            var watch = System.Diagnostics.Stopwatch.StartNew();
            Scrape(window, new ScrapeScope());
            ScrapeStatusWindow status = window.ScrapeStatusShown!;
            int drawnAtStart = status.Refreshes;
            PumpUntil(() => !window.ScrapeRunning, "the run's end");
            Pump(300);
            double seconds = watch.Elapsed.TotalSeconds;
            int drawn = status.Refreshes - drawnAtStart;
            Out.WriteLine($"{window.Progress!.Done} games, {window.Progress.Version} changes, {drawn} draws in {seconds:F2} s");
            Assert.Equal(40, window.Progress.Done);
            Assert.True(window.Progress.Version > drawn, $"{window.Progress.Version} changes, {drawn} draws");
            Assert.True(drawn <= seconds / ScrapeStatusWindow.RefreshEvery.TotalSeconds + 3, $"{drawn} draws in {seconds:F2} s");
        });

        [Fact]
        public Task Reopening_brings_the_one_window_forward_rather_than_opening_another() => OnUi(() =>
        {
            ThreeGames();
            Server.Gate = new SemaphoreSlim(0);
            MainWindow window = Open(settings: a => a.OpenEmuFallback = false);
            Scrape(window, new ScrapeScope());
            ScrapeStatusWindow first = window.ScrapeStatusShown!;
            window.ShowScrapeStatus();
            Assert.Same(first, window.ScrapeStatusShown);
            Assert.Single(window.OwnedWindows.OfType<ScrapeStatusWindow>());
        });

        [Fact]
        public Task The_pad_menu_and_the_scraping_tab_reopen_the_window_during_a_run() => OnUi(() =>
        {
            ThreeGames();
            Server.Gate = new SemaphoreSlim(0);
            MainWindow window = Open(settings: a => a.OpenEmuFallback = false);
            Scrape(window, new ScrapeScope());
            window.ScrapeStatusShown!.Close();

            Invoke(window, "OpenPadMenu");
            PadMenu(window).Single(e => e.Text() == "Scraping (0 of 3)...").Accept();
            Invoke(window, "ClosePadMenu");
            Assert.NotNull(window.ScrapeStatusShown);
            window.ScrapeStatusShown!.Close();

            var prefs = new PreferencesWindow(new AppSettings { RomDirectory = RomDir }, window);
            Windows.Add(prefs);
            prefs.Show();
            prefs.ShowTab(PreferencesWindow.ScrapingTab);
            Pump(100);
            Click(Named<Button>(prefs, "ScrapeStatusButton"));
            Assert.NotNull(window.ScrapeStatusShown);
            prefs.Close();
        });

        // In a big-screen session the window is a sheet, and every control on it is reached and pressed by the pad.
        [Fact]
        public Task In_a_big_screen_session_it_is_a_sheet_the_pad_reaches_and_works() => OnUi(() =>
        {
            ThreeGames();
            Server.Gate = new SemaphoreSlim(0);
            MainWindow window = Open(settings: a => a.OpenEmuFallback = false, bigScreen: true);
            var pad = new PadDriver(window);
            Scrape(window, new ScrapeScope());
            ReleaseUntil(() => window.Progress!.Done == 1, "a game, so the list has a row");
            ScrapeStatusWindow status = Assert.IsType<ScrapeStatusWindow>(Sheets(window).Current);
            Assert.Empty(window.OwnedWindows.OfType<ScrapeStatusWindow>());
            status.Refresh();
            window.UpdateLayout();

            Control sheet = RootOf(status);
            HashSet<Avalonia.Input.InputElement> reached = PadAudit.Reachable(sheet, pad);
            List<string> missing = PadAudit.Operable(sheet).Where(c => !reached.Contains(c)).Select(PadAudit.Describe).ToList();
            foreach (string m in missing) Out.WriteLine("unreachable: " + m);
            Assert.Empty(missing);
            Assert.Contains(reached, e => e is Button { Name: "ScrapeStatusPauseButton" });
            Assert.Contains(reached, e => e is Button { Name: "ScrapeStatusCancelButton" });
            Assert.Contains(reached, e => e is Button { Name: "ScrapeStatusHideButton" });
            Assert.Contains(reached, e => e is ListBoxItem);

            PadAudit.Reach(sheet, pad, e => e is Button { Name: "ScrapeStatusPauseButton" });
            pad.A();
            Assert.True(window.ScrapePaused);
            PadAudit.Reach(sheet, pad, e => e is Button { Name: "ScrapeStatusPauseButton" });
            pad.A();
            Assert.False(window.ScrapePaused);

            PadAudit.Reach(sheet, pad, e => e is Button { Name: "ScrapeStatusHideButton" });
            pad.A();
            Assert.False(Sheets(window).IsPresenting);
            Assert.True(window.ScrapeRunning);

            pad.Start();
            List<PadMenuEntry> entries = PadMenu(window);
            int at = entries.FindIndex(e => e.Text().StartsWith("Scraping (", StringComparison.Ordinal));
            Assert.True(at >= 0, string.Join(", ", entries.Select(e => e.Text())));
            pad.Down(at);
            pad.A();
            Assert.IsType<ScrapeStatusWindow>(Sheets(window).Current);
        });

        // No text the window shows holds a credential: every route passes the redactor, the window's own included.
        [Fact]
        public Task Nothing_the_window_shows_holds_a_credential() => OnUi(() =>
        {
            ThreeGames();
            new MemberAccount("FAKEMEMBERUSER", "FAKEMEMBERSECRET") { Verified = DateTime.UtcNow }.Save();
            MainWindow window = Open(settings: a => a.OpenEmuFallback = false);
            Scrape(window, new ScrapeScope());
            RunEnds(window);
            ScrapeStatusWindow status = window.ScrapeStatusShown!;
            status.Refresh();
            string all = AllText(status);
            Assert.Contains("Member: FAKEMEMBERUSER", all);
            Assert.Contains("devpassword=***", all);
            foreach (string secret in new[] { FakeScreenScraper.DevPassword, FakeScreenScraper.DevId, "FAKEMEMBERSECRET" })
                Assert.DoesNotContain(secret, all);
        });
    }
}
