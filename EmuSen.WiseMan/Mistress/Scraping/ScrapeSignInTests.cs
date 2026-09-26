using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Input;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Scraping;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // Log In checks a member account with one request and keeps it only when accepted; Log Out deletes it and stops it being sent; an old file shows as not checked - see EmuSen_Settings_Reference.md §4.60.
    [Collection(TestCollections.ProcessGlobals)]
    public class ScrapeSignInTests : ScrapeWindowFixture
    {
        private const string User = "FAKEMEMBERUSER";
        private const string Secret = "FAKEMEMBERSECRET";

        public ScrapeSignInTests(ITestOutputHelper output) : base(output) { }

        private PreferencesWindow Prefs(MainWindow window)
        {
            var prefs = new PreferencesWindow(new AppSettings { RomDirectory = RomDir }, window);
            Windows.Add(prefs);
            prefs.Show();
            prefs.ShowTab(PreferencesWindow.ScrapingTab);
            Pump(100);
            prefs.UpdateLayout();
            return prefs;
        }

        private static void Type(PreferencesWindow prefs, string user, string password)
        {
            Named<TextBox>(prefs, "ScreenScraperUserBox").Text = user;
            Named<TextBox>(prefs, "ScreenScraperPasswordBox").Text = password;
        }

        private static string Said(Window prefs) => TextOf(prefs, "ScreenScraperSignInMessage");

        private static void Press(Window prefs, string button, Func<bool> done)
        {
            Click(Named<Button>(prefs, button));
            WaitFor(done, button);
        }

        private static bool Answered(Window prefs) => Said(prefs) is { Length: > 0 } s && s != "Asking ScreenScraper...";

        private IEnumerable<string> UserInfos => Server.Asked.Where(u => u.Contains("/ssuserInfos.php"));

        [Fact]
        public Task Nothing_is_sent_until_Log_In_which_sends_one_request_and_keeps_the_account_0600() => OnUi(() =>
        {
            Server.Member = (User, Secret);
            MainWindow window = Open();
            PreferencesWindow prefs = Prefs(window);
            Assert.Equal("Not signed in: runs use EmuSen's developer credentials alone.", TextOf(prefs, "ScreenScraperMemberText"));
            Type(prefs, User, Secret);
            Pump(300);
            Assert.Empty(Server.Asked);
            Assert.False(File.Exists(MemberAccount.PathOf));

            Press(prefs, "ScreenScraperLogInButton", () => Answered(prefs));
            string url = Assert.Single(Server.Asked);
            Assert.Contains("/ssuserInfos.php", url);
            Assert.Equal((User, Secret, FakeScreenScraper.DevId), (FakeScreenScraper.Param(url, "ssid"), FakeScreenScraper.Param(url, "sspassword"), FakeScreenScraper.Param(url, "devid")));
            Assert.Equal($"Signed in as {User}.", Said(prefs));
            Assert.Equal($"Signed in as {User} · level 3 · 10 of 20,000 requests today · 1 thread · 128 KB/s", TextOf(prefs, "ScreenScraperMemberText"));
            Assert.False(Named<TextBox>(prefs, "ScreenScraperPasswordBox").IsVisible);
            Assert.True(Named<Button>(prefs, "ScreenScraperLogOutButton").IsVisible);
            Assert.False(Named<Button>(prefs, "ScreenScraperCheckButton").IsVisible);

            MemberAccount kept = MemberAccount.Load();
            Assert.Equal((User, Secret), (kept.User, kept.Password));
            Assert.NotNull(kept.Verified);
            Assert.Equal(UnixFileMode.UserRead | UnixFileMode.UserWrite, File.GetUnixFileMode(MemberAccount.PathOf));
            Assert.DoesNotContain("FAKEMEMBER", File.ReadAllText(ConfigStore.For("appsettings.json")));

            window.ShowScrapeStatus();
            Assert.Equal($"Member: {User}", TextOf(window.ScrapeStatusShown!, "ScrapeStatusMember"));
            prefs.Close();
            Pump(100);
            Assert.Single(Server.Asked);
        });

        // Each refusal in plain words, one request, nothing kept, and no credential in the words.
        [Theory]
        [InlineData(0, "", "ScreenScraper did not accept that name and password.")]
        [InlineData(403, "Erreur de login : Vérifier vos identifiants développeur !", "ScreenScraper refused EmuSen's developer credentials (403), so no account can be checked with this build.")]
        [InlineData(401, "Erreur : API fermée aux non membres", "ScreenScraper is too busy to check an account now (401). Try again in a few minutes.")]
        [InlineData(423, "Erreur : API fermée", "ScreenScraper's API is closed (423). Try again later.")]
        [InlineData(500, "boom devpassword=FAKEDEVPASSWORD&sspassword=FAKEMEMBERWRONG", "ScreenScraper did not answer normally: boom devpassword=***&sspassword=***")]
        public Task A_refused_log_in_says_why_in_plain_words_and_keeps_nothing(int code, string body, string said) => OnUi(() =>
        {
            Server.Member = (User, Secret);
            Server.ForcedUserStatus = code;
            Server.ForcedUserBody = body;
            MainWindow window = Open();
            PreferencesWindow prefs = Prefs(window);
            Type(prefs, User, "FAKEMEMBERWRONG");
            Press(prefs, "ScreenScraperLogInButton", () => Answered(prefs));
            Assert.Single(UserInfos);
            Assert.Equal(said, Said(prefs));
            Assert.False(File.Exists(MemberAccount.PathOf));
            Assert.Equal("Not signed in: runs use EmuSen's developer credentials alone.", TextOf(prefs, "ScreenScraperMemberText"));
            foreach (string secret in new[] { "FAKEMEMBERWRONG", Secret, FakeScreenScraper.DevPassword, FakeScreenScraper.DevId })
                Assert.DoesNotContain(secret, Said(prefs));
        });

        [Fact]
        public Task Without_the_developer_file_log_in_says_it_cannot_be_checked_here_and_sends_nothing() => OnUi(() =>
        {
            MainWindow window = Open(developer: false);
            PreferencesWindow prefs = Prefs(window);
            Type(prefs, User, Secret);
            Press(prefs, "ScreenScraperLogInButton", () => Answered(prefs));
            Assert.StartsWith("Signing in can't be checked or used on this computer: EmuSen's developer file is not here", Said(prefs));
            Assert.Empty(Server.Asked);
            Assert.False(File.Exists(MemberAccount.PathOf));
        });

        [Fact]
        public Task Typing_an_account_without_Log_In_keeps_nothing() => OnUi(() =>
        {
            MainWindow window = Open();
            PreferencesWindow prefs = Prefs(window);
            Type(prefs, User, Secret);
            Named<TextBox>(prefs, "ScreenScraperUserBox").Focus();
            Named<TextBox>(prefs, "ScreenScraperPasswordBox").Focus();
            prefs.Close();
            Pump(100);
            Assert.False(File.Exists(MemberAccount.PathOf));
            Assert.Empty(Server.Asked);
        });

        [Fact]
        public Task Log_Out_deletes_the_file_and_a_run_in_progress_stops_sending_the_member_from_its_next_request() => OnUi(() =>
        {
            string first = Game("A First (USA)", 10), second = Game("B Second (USA)", 20);
            Server.Games.Add(new FakeGame(77, "First", Md5(first)));
            Server.Games.Add(new FakeGame(78, "Second", Md5(second)));
            new MemberAccount(User, Secret) { Verified = DateTime.UtcNow }.Save();
            Server.Gate = new SemaphoreSlim(0);
            MainWindow window = Open(settings: a => a.OpenEmuFallback = false);
            PreferencesWindow prefs = Prefs(window);
            Assert.StartsWith($"Signed in as {User} · checked ", TextOf(prefs, "ScreenScraperMemberText"));
            Scrape(window, new ScrapeScope());
            WaitFor(() => Server.Asked.Count == 1, "the first lookup");
            Assert.Equal(User, FakeScreenScraper.Param(Server.Asked.Single(), "ssid"));

            Click(Named<Button>(prefs, "ScreenScraperLogOutButton"));
            Assert.Equal("Signed out: screenscraper.json was deleted. Runs now use EmuSen's developer credentials alone.", Said(prefs));
            Assert.False(File.Exists(MemberAccount.PathOf));
            Assert.Equal("Not signed in: runs use EmuSen's developer credentials alone.", TextOf(prefs, "ScreenScraperMemberText"));
            int before = Server.Asked.Count;
            Server.Gate.Release(1000);
            RunEnds(window);
            string[] after = Server.Asked.Skip(before).ToArray();
            Assert.Equal(4 + 1 + 4, after.Length);
            Assert.All(after, u => Assert.DoesNotContain("ssid=", u));
            Assert.All(after, u => Assert.DoesNotContain("sspassword=", u));
            window.ShowScrapeStatus();
            Assert.StartsWith("No member account", TextOf(window.ScrapeStatusShown!, "ScrapeStatusMember"));
        });

        [Fact]
        public Task An_account_from_the_old_two_boxes_shows_signed_in_not_checked_is_used_and_Check_verifies_it() => OnUi(() =>
        {
            Directory.CreateDirectory(ConfigStore.Directory);
            File.WriteAllText(MemberAccount.PathOf, $"{{\"ssid\":\"{User}\",\"sspassword\":\"{Secret}\"}}");
            File.SetUnixFileMode(MemberAccount.PathOf, UnixFileMode.UserRead | UnixFileMode.UserWrite);
            Server.Member = (User, Secret);
            MainWindow window = Open();
            PreferencesWindow prefs = Prefs(window);
            Assert.Equal($"Signed in as {User} · not checked: this account was typed before Log In checked accounts. Check asks ScreenScraper once.", TextOf(prefs, "ScreenScraperMemberText"));
            Assert.True(Named<Button>(prefs, "ScreenScraperCheckButton").IsVisible);
            Assert.True(Named<Button>(prefs, "ScreenScraperLogOutButton").IsVisible);
            Pump(300);
            Assert.Empty(Server.Asked);
            Assert.Null(MemberAccount.Load().Verified);
            Assert.Equal(User, MemberAccount.Load().User);

            window.ShowScrapeStatus();
            Assert.Equal($"Member: {User} (not checked; Check in Preferences ▸ Scraping)", TextOf(window.ScrapeStatusShown!, "ScrapeStatusMember"));

            Press(prefs, "ScreenScraperCheckButton", () => Answered(prefs));
            Assert.Single(UserInfos);
            Assert.Equal($"Signed in as {User}.", Said(prefs));
            Assert.NotNull(MemberAccount.Load().Verified);
            Assert.False(Named<Button>(prefs, "ScreenScraperCheckButton").IsVisible);
            Assert.Equal(UnixFileMode.UserRead | UnixFileMode.UserWrite, File.GetUnixFileMode(MemberAccount.PathOf));
        });

        [Fact]
        public Task A_check_that_fails_keeps_the_old_account_and_says_so() => OnUi(() =>
        {
            Directory.CreateDirectory(ConfigStore.Directory);
            File.WriteAllText(MemberAccount.PathOf, $"{{\"ssid\":\"{User}\",\"sspassword\":\"FAKEMEMBERWRONG\"}}");
            Server.Member = (User, Secret);
            MainWindow window = Open();
            PreferencesWindow prefs = Prefs(window);
            Press(prefs, "ScreenScraperCheckButton", () => Answered(prefs));
            Assert.Equal("ScreenScraper did not accept that name and password. The account is kept until you Log Out.", Said(prefs));
            Assert.Equal(("FAKEMEMBERWRONG", (DateTime?)null), (MemberAccount.Load().Password, MemberAccount.Load().Verified));
        });

        [Fact]
        public Task An_old_account_is_sent_by_a_run_before_it_is_checked() => OnUi(() =>
        {
            string game = Game("A First (USA)", 10);
            Directory.CreateDirectory(ConfigStore.Directory);
            File.WriteAllText(MemberAccount.PathOf, $"{{\"ssid\":\"{User}\",\"sspassword\":\"{Secret}\"}}");
            MainWindow window = Open(settings: a => a.OpenEmuFallback = false);
            Scrape(window, ScrapeScope.ThisGame(game));
            RunEnds(window);
            Assert.Equal(User, FakeScreenScraper.Param(Server.JeuInfos.Single(), "ssid"));
        });

        // On the sheet in a big-screen session: the boxes are typed on the on-screen keyboard, and Log In and Log Out are reached and pressed by the pad.
        [Fact]
        public Task Log_In_and_Log_Out_are_reached_and_used_by_the_pad_on_the_sheet() => OnUi(() =>
        {
            Server.Member = ("padplayer", "padsecret");
            MainWindow window = Open(bigScreen: true);
            var pad = new PadDriver(window);
            pad.Start();
            List<EmuSen.Mistress.Input.PadMenuEntry> entries = PadMenu(window);
            int at = entries.FindIndex(e => e.Text() == "Scrape Games...");
            pad.Down(at);
            pad.A();
            var prefs = Assert.IsType<PreferencesWindow>(Sheets(window).Current);
            Control sheet = RootOf(prefs);
            window.UpdateLayout();

            HashSet<InputElement> reached = PadAudit.Reachable(sheet, pad);
            Assert.Contains(reached, e => e is Button { Name: "ScreenScraperLogInButton" });
            Assert.Contains(reached, e => e is TextBox { Name: "ScreenScraperPasswordBox" });
            Assert.Empty(PadAudit.Operable(sheet).Where(c => !reached.Contains(c)).Select(PadAudit.Describe));

            foreach ((string box, string text) in new[] { ("ScreenScraperUserBox", "padplayer"), ("ScreenScraperPasswordBox", "padsecret") })
            {
                PadAudit.Reach(sheet, pad, e => e is TextBox t && t.Name == box);
                pad.A();
                OnScreenKeyboard keyboard = OnScreenKeyboard.OpenOver(window) ?? throw new InvalidOperationException("no keyboard");
                PadCheatsTests.TypeByPad(pad, keyboard, text);
                pad.Start();
                Assert.Null(OnScreenKeyboard.OpenOver(window));
            }
            Assert.Equal('•', Named<TextBox>(prefs, "ScreenScraperPasswordBox").PasswordChar);
            Assert.Empty(Server.Asked);

            PadAudit.Reach(sheet, pad, e => e is Button { Name: "ScreenScraperLogInButton" });
            pad.A();
            WaitFor(() => Answered(prefs), "the log in");
            Assert.Equal("Signed in as padplayer.", Said(prefs));
            Assert.Single(Server.Asked);
            // Log In hides once signed in; the focus is handed to Log Out rather than lost.
            Assert.Equal("ScreenScraperLogOutButton", (window.FocusManager!.GetFocusedElement() as Control)?.Name);

            window.UpdateLayout();
            reached = PadAudit.Reachable(sheet, pad);
            Assert.Contains(reached, e => e is Button { Name: "ScreenScraperLogOutButton" });
            PadAudit.Reach(sheet, pad, e => e is Button { Name: "ScreenScraperLogOutButton" });
            pad.A();
            Assert.False(File.Exists(MemberAccount.PathOf));
            Assert.StartsWith("Signed out", Said(prefs));
            Assert.Equal("ScreenScraperUserBox", (window.FocusManager!.GetFocusedElement() as Control)?.Name);
        });

        [Fact]
        public void A_sign_in_answer_never_carries_a_credential()
        {
            ScrapeRedactor.Register("FAKESIGNINPW");
            SignInAnswer failed = SignInAnswer.From(ScrapeStatus.Failed, "connect to https://api.screenscraper.fr/api2/ssuserInfos.php?devid=FAKEX&devpassword=FAKEY&ssid=someone&sspassword=FAKESIGNINPW refused");
            Assert.Equal(SignInResult.Unreachable, failed.Result);
            Assert.DoesNotContain("FAKEY", failed.Message);
            Assert.DoesNotContain("FAKESIGNINPW", failed.Message);
            Assert.Contains("sspassword=***", failed.Message);

            Assert.Equal(SignInResult.WrongAccount, SignInAnswer.From(ScrapeStatus.BadCredentials, "Erreur de login : Vérifier les identifiants utilisateurs !").Result);
            Assert.Equal(SignInResult.DeveloperRefused, SignInAnswer.From(ScrapeStatus.BadCredentials, "Erreur de login : Vérifier vos identifiants développeur !").Result);
            Assert.Equal(SignInResult.WrongAccount, SignInAnswer.From(ScrapeStatus.Found, "{\"header\":{},\"response\":{\"ssuser\":null}}").Result);
            Assert.Equal(SignInResult.Unreadable, SignInAnswer.From(ScrapeStatus.Found, "not json").Result);
        }
    }

    // The run's own numbers: the pace behind the estimate, pauses left out of it, skipped games counted apart, the recent list's cap, and redaction.
    public class ScrapeProgressTests
    {
        private static readonly DateTimeOffset T0 = new(2026, 9, 26, 12, 0, 0, TimeSpan.Zero);

        private static ScrapeResult Result(string name, ScrapeOutcome outcome, bool skipped = false, string detail = "", params string[] written) =>
            new($"/roms/{name}.sfc", "snes", outcome, outcome == ScrapeOutcome.Found, written, detail, Skipped: skipped);

        [Fact]
        public void The_estimate_is_the_run_s_own_pace_over_the_games_left()
        {
            var run = new ScrapeProgress(10, T0);
            Assert.Null(run.EstimateLeft(T0.AddSeconds(5)));
            run.Count(Result("a", ScrapeOutcome.Found));
            run.Count(Result("b", ScrapeOutcome.Unknown));
            Assert.Equal(TimeSpan.FromSeconds(80), run.EstimateLeft(T0.AddSeconds(20)));
            Assert.Equal(TimeSpan.FromSeconds(160), run.EstimateLeft(T0.AddSeconds(40)));
        }

        [Fact]
        public void Paused_time_is_not_work_and_skipped_games_are_not_pace()
        {
            var run = new ScrapeProgress(10, T0);
            run.Count(Result("a", ScrapeOutcome.Found));
            run.SetPaused(true, T0.AddSeconds(10));
            Assert.True(run.IsPaused);
            Assert.Equal(TimeSpan.FromSeconds(10), run.Working(T0.AddSeconds(100)));
            run.SetPaused(false, T0.AddSeconds(100));
            Assert.Equal(TimeSpan.FromSeconds(20), run.Working(T0.AddSeconds(110)));
            Assert.Equal(TimeSpan.FromSeconds(110), run.Elapsed(T0.AddSeconds(110)));
            run.Count(Result("b", ScrapeOutcome.Found, skipped: true, detail: "already scraped"));
            Assert.Equal(TimeSpan.FromSeconds(160), run.EstimateLeft(T0.AddSeconds(110)));
        }

        [Fact]
        public void Each_outcome_has_its_own_tally_and_the_failure_its_reason()
        {
            var run = new ScrapeProgress(6, T0);
            run.Count(Result("a", ScrapeOutcome.Found, written: [Path.Combine("snes", "covers", "a.png"), Path.Combine("snes", "miximages", "a.png")]));
            run.Count(Result("b", ScrapeOutcome.Unknown));
            run.Count(Result("c", ScrapeOutcome.Error, detail: "BadRequest: nope"));
            run.Count(Result("d", ScrapeOutcome.Found, skipped: true, detail: "already scraped"));
            run.Count(Result("e", ScrapeOutcome.Missing, skipped: true, detail: "the file is gone"));
            run.Count(Result("f", ScrapeOutcome.Retry, detail: "Failed: timeout"));
            Assert.Equal((5, 1, 1, 1, 2, 1), (run.Done, run.Found, run.Unknown, run.Failed, run.Skipped, run.Retrying));
            Assert.Equal("BadRequest: nope", run.LastFailure);
            Assert.Equal(["e", "d", "c", "b", "a"], run.Recent.Select(r => Path.GetFileNameWithoutExtension(r.Path)));
            Assert.Equal(["cover", "miximage"], run.Recent.Last().Kinds);
            run.FilledByFailover("/roms/b.sfc");
            Assert.True(run.Recent.Single(r => r.Path == "/roms/b.sfc").FromFailover);
        }

        [Fact]
        public void The_recent_list_keeps_the_newest_only()
        {
            var run = new ScrapeProgress(ScrapeProgress.RecentLimit + 50, T0);
            for (int i = 0; i < ScrapeProgress.RecentLimit + 50; i++) run.Count(Result($"g{i}", ScrapeOutcome.Unknown));
            Assert.Equal(ScrapeProgress.RecentLimit, run.Recent.Count);
            Assert.Equal($"g{ScrapeProgress.RecentLimit + 49}", Path.GetFileNameWithoutExtension(run.Recent[0].Path));
        }

        [Fact]
        public void Requests_are_the_lookups_and_the_pictures_a_worker_reported()
        {
            var run = new ScrapeProgress(1, T0);
            run.Note(new ScrapeActivity("/roms/a.sfc", "snes", ScrapeStep.LookingUp));
            run.Note(new ScrapeActivity("/roms/a.sfc", "snes", ScrapeStep.Downloading, "cover"));
            run.Note(new ScrapeActivity("/roms/a.sfc", "snes", ScrapeStep.Arrived, "cover", "/media/snes/covers/a.png"));
            Assert.Equal(2, run.Requests);
            Assert.Equal("/media/snes/covers/a.png", run.LastPicture);
            Assert.Equal(ScrapeStep.Arrived, run.Current!.Step);
        }

        [Fact]
        public void A_credential_in_a_failure_or_a_row_is_blanked()
        {
            var run = new ScrapeProgress(1, T0);
            run.Count(Result("a", ScrapeOutcome.Error, detail: "Failed: GET ?devid=FAKEROWID&devpassword=FAKEROWPW&sspassword=FAKEROWSS"));
            Assert.DoesNotContain("FAKEROWPW", run.LastFailure);
            Assert.DoesNotContain("FAKEROWSS", run.LastFailure);
            string row = ScrapeStatusWindow.Describe(new ScrapeRecent("/roms/a.sfc", "snes", ScrapeOutcome.Error, [], "devpassword=FAKEROWPW2", false));
            Assert.DoesNotContain("FAKEROWPW2", row);
            Assert.Contains("devpassword=***", row);
        }
    }
}
