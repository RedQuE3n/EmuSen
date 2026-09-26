using System;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Layout;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.Mistress.Scraping;

namespace EmuSen.Mistress.Views
{
    // What Preferences asks of the window that scrapes: whether it can, the quota, and a run started, resumed or cancelled by the player.
    public interface IScrapeHost
    {
        bool HasDeveloperCredentials { get; }
        string Status { get; }
        QuotaSnapshot? Quota { get; }
        event Action? ScrapeChanged;
        bool ScrapeRunning { get; }
        ScrapeProgress? Progress { get; }
        int Interrupted { get; }
        System.Threading.Tasks.Task<bool> ConfirmAndScrapeAsync(ScrapeScope scope);
        bool ResumeScrape();
        void CancelScrape();

        // The status window's - see EmuSen_Settings_Reference.md §4.57.
        void ShowScrapeStatus();
        bool ScrapePaused { get; }
        bool CanPauseScrape { get; }
        void SetScrapePaused(bool paused);
        System.Threading.Tasks.Task<bool> ConfirmCancelAsync(Window owner);
        DateTimeOffset ScrapeNow { get; }
        ScrapeQuota? ScrapeLimits { get; }

        // The member account's sign-in - see EmuSen_Settings_Reference.md §4.60.
        MemberAccount Member { get; }
        ScrapeMember? MemberChecked { get; }
        System.Threading.Tasks.Task<SignInAnswer> SignInAsync(string user, string password);
        System.Threading.Tasks.Task<SignInAnswer> CheckMemberAsync();
        string SignOut();
    }

    // Preferences' Scraping tab: ScreenScraper, the member account, what to fetch, region and language, the quota, and OpenEmu's failover - see EmuSen_Settings_Reference.md §4.60.
    public sealed class ScrapePreferencesPane
    {
        public static readonly (string Value, string Text)[] Regions =
        {
            (ScrapeRules.AutomaticRegion, "Automatic, from the file's name"), ("wor", "World"), ("us", "USA"), ("eu", "Europe"), ("jp", "Japan"),
            ("uk", "United Kingdom"), ("fr", "France"), ("de", "Germany"), ("sp", "Spain"), ("it", "Italy"), ("au", "Australia"), ("kr", "Korea"), ("br", "Brazil"),
        };

        public static readonly (string Value, string Text)[] Languages =
        {
            ("en", "English"), ("fr", "French"), ("de", "German"), ("es", "Spanish"), ("it", "Italian"), ("pt", "Portuguese"), ("ja", "Japanese"), ("nl", "Dutch"),
        };

        private readonly AppSettings _settings;
        private readonly IScrapeHost? _host;
        private readonly MeterRow _quota = new() { Name = "ScrapeQuotaMeter", Label = "Requests today" };
        private readonly TextBlock _status = new() { Name = "ScrapeStatusText", TextWrapping = Avalonia.Media.TextWrapping.Wrap };

        private readonly TextBox _user = new() { Name = "ScreenScraperUserBox", Watermark = "(none)", HorizontalAlignment = HorizontalAlignment.Stretch };
        private readonly TextBox _password = new() { Name = "ScreenScraperPasswordBox", PasswordChar = '•', Watermark = "(none)", HorizontalAlignment = HorizontalAlignment.Stretch };
        private readonly TextBlock _memberText = new() { Name = "ScreenScraperMemberText", TextWrapping = Avalonia.Media.TextWrapping.Wrap };
        private readonly HintText _signInMessage = new() { Name = "ScreenScraperSignInMessage" };
        private readonly Button _signIn = Ui.Button("Log In", () => { });
        private readonly Button _check = Ui.Button("Check", () => { });
        private readonly Button _signOut = Ui.Button("Log Out", () => { });
        private readonly Button _statusButton = Ui.Button("Status...", () => { });

        private readonly Dropdown _scopeShelf = new() { Name = "ScrapeShelfDropdown", HorizontalAlignment = HorizontalAlignment.Stretch };
        private readonly LunaSwitch _missingOnly = new() { Name = "ScrapeMissingArtSwitch", Label = "Only games with no cover", IsChecked = true };
        private readonly Button _start = Ui.Button("Scrape...", () => { });
        private readonly Button _resume = Ui.Button("Resume", () => { });
        private readonly Button _cancel = Ui.Button("Cancel Scraping", () => { });
        private readonly ProgressBar _progress = new() { Name = "ScrapeProgressBar", Minimum = 0, Maximum = 1, HorizontalAlignment = HorizontalAlignment.Stretch };

        // The shelves a scope can name, Game Boy Color as its own; the first entry is the whole library.
        public const string AllShelves = "Every console";

        public static string[] ShelfChoices => [AllShelves, .. EmuSen.Cores.CoreCatalog.ShelvesInReleaseOrder.Select(s => s.Name)];

        public ScrapeScope ChosenScope() => new(Shelf: _scopeShelf.SelectedItem is string s && s != AllShelves ? s : null, MissingArtOnly: _missingOnly.IsChecked == true);

        public ScrapePreferencesPane(AppSettings settings, IScrapeHost? host)
        {
            _settings = settings;
            _host = host;
        }

        public Control[] Rows()
        {
            _signIn.Name = "ScreenScraperLogInButton";
            _signIn.Click += async (_, _) => await SignInAsync(check: false);
            _check.Name = "ScreenScraperCheckButton";
            _check.Click += async (_, _) => await SignInAsync(check: true);
            _signOut.Name = "ScreenScraperLogOutButton";
            _signOut.Click += (_, _) => SignOut();
            _statusButton.Name = "ScrapeStatusButton";
            _statusButton.Click += (_, _) => _host?.ShowScrapeStatus();

            _scopeShelf.Fill(ShelfChoices, AllShelves);
            _start.Name = "ScrapeStartButton";
            _start.Click += async (_, _) => { if (_host is not null) await _host.ConfirmAndScrapeAsync(ChosenScope()); Show(); };
            _resume.Name = "ScrapeResumeButton";
            _resume.Click += (_, _) => { _host?.ResumeScrape(); Show(); };
            _cancel.Name = "ScrapeCancelButton";
            _cancel.Click += (_, _) => { _host?.CancelScrape(); Show(); };

            bool developer = _host?.HasDeveloperCredentials == true;
            return
            [
                new FieldRow
                {
                    Label = "ScreenScraper",
                    Hint = "Covers, screenshots, marquees, mix images and each game's description, developer, publisher, genre, players, rating and release date, from screenscraper.fr. "
                        + "For each game it sends the file's name, size and three hashes (MD5, CRC32, SHA-1), the console, EmuSen's developer credentials and your member account if one is below, so ScreenScraper sees which games you have. "
                        + "What comes back is written by ScreenScraper's contributors and the art belongs to its publishers; it is kept in home/Media and never shared. "
                        + (developer ? "EmuSen's developer file is on this computer." : "EmuSen's developer file is not on this computer, so ScreenScraper cannot be used here; builds do not carry it."),
                    Content = Switch("ScrapingSwitch", "Use ScreenScraper", _settings.Scraping, v => _settings.Scraping = v),
                },
                new FieldRow
                {
                    Label = "Member Account",
                    Hint = "Optional. Your own free account at screenscraper.fr; contributing or donating there raises its daily requests and threads. "
                        + "Log In checks it with one request to ScreenScraper and keeps it only if accepted, in its own file readable only by you, never in appsettings.json. Log Out deletes that file. "
                        + "The developer credentials are always EmuSen's: ScreenScraper issues those to software authors, not to players, so your own cannot be used.",
                    Content = Ui.Stack(6, _memberText, _user, _password, Ui.Row(8, _signIn, _check, _signOut), _signInMessage),
                },
                new FieldRow
                {
                    Label = "Fetch",
                    Hint = "Box art is ScreenScraper's box-2D, the marquee its wheel, the mix image its mixrbv2. A picture already in the cover art folder is never fetched again.",
                    Content = Ui.Stack(6,
                        Switch("ScrapeCoversSwitch", "Covers", _settings.ScrapeCovers, v => _settings.ScrapeCovers = v),
                        Switch("ScrapeScreenshotsSwitch", "Screenshots", _settings.ScrapeScreenshots, v => _settings.ScrapeScreenshots = v),
                        Switch("ScrapeMarqueesSwitch", "Marquees", _settings.ScrapeMarquees, v => _settings.ScrapeMarquees = v),
                        Switch("ScrapeMiximagesSwitch", "Mix images", _settings.ScrapeMiximages, v => _settings.ScrapeMiximages = v),
                        Switch("ScrapeTitleScreensSwitch", "Title screens", _settings.ScrapeTitleScreens, v => _settings.ScrapeTitleScreens = v)),
                },
                new FieldRow
                {
                    Label = "Region",
                    Hint = "Whose box and name are preferred. Automatic reads the file's own tag: (USA), (Europe), (Japan), (World).",
                    Content = Ui.Stack(6,
                        Choice("ScrapeRegionDropdown", Regions, _settings.ScrapeRegion, v => _settings.ScrapeRegion = v),
                        Switch("ScrapeRegionFallbackSwitch", "Else world, USA, Europe, Japan, then any", _settings.ScrapeRegionFallback, v => _settings.ScrapeRegionFallback = v)),
                },
                new FieldRow
                {
                    Label = "Language",
                    Hint = "The description's and genre's language; English when ScreenScraper has none in this one.",
                    Content = Choice("ScrapeLanguageDropdown", Languages, _settings.ScrapeLanguage, v => _settings.ScrapeLanguage = v),
                },
                new FieldRow
                {
                    Label = "Scrape",
                    Hint = "Nothing is sent to ScreenScraper or OpenEmu's sources until you start it here, from a game's menu (Scrape This Game), or from the pad menu. "
                        + "Choose a console or every console, and whether only games with no cover; the count and the requests it will cost are shown before it starts. "
                        + "A run that is cancelled or stopped by the quota can be resumed; it never resumes by itself.",
                    Content = Ui.Stack(6, _scopeShelf, _missingOnly, Ui.Row(8, _start, _resume, _cancel, _statusButton), _progress),
                },
                new FieldRow
                {
                    Label = "Today",
                    Hint = "ScreenScraper's own count, from its last answer. Mistress stops two percent short of the day's limit and resumes the next day where it stopped.",
                    Content = Ui.Stack(6, _quota, _status),
                },
                new FieldRow
                {
                    Label = "OpenEmu Failover",
                    Hint = "Only for a game ScreenScraper has no cover for, or while ScreenScraper cannot be used (no developer file, today's quota used up, the service closed or refusing this build). "
                        + "Mistress then downloads OpenVGDB, the game database OpenEmu uses (about 9 MB, from GitHub), and asks thumbnails.libretro.com for the game's box, then the address OpenVGDB gives (GameFAQs). "
                        + "Those servers see which of those games you have. OpenVGDB states no licence and the covers are other people's scans. They are kept in home/Media/openemu.",
                    Content = Switch("OpenEmuFallbackSwitch", "Use OpenEmu's sources (OpenVGDB and libretro thumbnails) when ScreenScraper has nothing", _settings.OpenEmuFallback, v => _settings.OpenEmuFallback = v),
                },
            ];
        }

        // The quota and status from the host, again whenever it says something changed; unsubscribed when the sheet closes.
        public void Attach(Window window)
        {
            Show();
            if (_host is null) return;
            _host.ScrapeChanged += Show;
            window.Closed += (_, _) => _host.ScrapeChanged -= Show;
        }

        // Log In sends what was typed and keeps it only when ScreenScraper accepts it; Check asks about the stored account - see EmuSen_Settings_Reference.md §4.60.
        private async System.Threading.Tasks.Task SignInAsync(bool check)
        {
            if (_host is null) return;
            _signIn.IsEnabled = _check.IsEnabled = false;
            _signInMessage.Text = "Asking ScreenScraper...";
            SignInAnswer answer = check ? await _host.CheckMemberAsync() : await _host.SignInAsync(_user.Text ?? "", _password.Text ?? "");
            _signIn.IsEnabled = _check.IsEnabled = true;
            _signInMessage.Text = answer.Message + (check && !answer.SignedIn && _host.Member.IsSet ? " The account is kept until you Log Out." : "");
            if (answer.SignedIn) _password.Text = "";
            Show();
        }

        private void SignOut()
        {
            _signInMessage.Text = _host?.SignOut() ?? (MemberAccount.Delete() ? "Signed out: screenscraper.json was deleted." : "Signed out.");
            _user.Text = "";
            _password.Text = "";
            Show();
        }

        // Signed in, from this session's check or the file's date; an old file from the two boxes shows as not checked, with Check.
        private void ShowMember()
        {
            MemberAccount member = _host?.Member ?? MemberAccount.Load();
            ScrapeMember? seen = _host?.MemberChecked;
            bool signedIn = member.IsSet;
            _user.IsVisible = _password.IsVisible = _signIn.IsVisible = !signedIn;
            _signIn.IsEnabled = _host is not null;
            _signOut.IsVisible = signedIn;
            _check.IsVisible = signedIn && member.Verified is null && seen is null;
            if (!signedIn)
            {
                _memberText.Text = "Not signed in: runs use EmuSen's developer credentials alone.";
                return;
            }
            string text = $"Signed in as {member.User}";
            if (seen is not null)
            {
                if (seen.Level is { Length: > 0 } level) text += $" · level {level}";
                if (seen.Quota is { } q)
                {
                    if (q.MaxRequestsPerDay is int day) text += $" · {q.RequestsToday ?? 0:N0} of {day:N0} requests today";
                    if (q.MaxThreads is int threads) text += $" · {threads} thread{(threads == 1 ? "" : "s")}";
                    if (q.MaxDownloadKBps is int kb) text += $" · {kb:N0} KB/s";
                }
            }
            else if (member.Verified is DateTime at) text += $" · checked {at.ToLocalTime():d MMM yyyy}";
            else text += " · not checked: this account was typed before Log In checked accounts. Check asks ScreenScraper once.";
            _memberText.Text = text;
        }

        private void Show()
        {
            ShowMember();
            _status.Text = _host?.Status ?? "";
            bool running = _host?.ScrapeRunning == true;
            _start.IsEnabled = _host is not null && !running;
            _resume.IsEnabled = !running && (_host?.Interrupted ?? 0) > 0;
            _cancel.IsEnabled = running;
            ScrapeProgress? p = _host?.Progress;
            _progress.IsVisible = p is not null;
            if (p is not null) _progress.Value = p.Total == 0 ? 0 : (double)p.Done / p.Total;
            QuotaSnapshot? q = _host?.Quota;
            _quota.IsVisible = q is not null;
            if (q is null) return;
            _quota.Percent = q.MaxPerDay is int max && max > 0 ? Math.Min(100, 100.0 * q.RequestsToday / max) : 0;
            string perDay = q.MaxPerDay is int m ? $"{q.RequestsToday:N0} of {m:N0}" : $"{q.RequestsToday:N0}";
            string ko = q.MaxKoPerDay is int mk ? $"{q.KoToday:N0} of {mk:N0}" : $"{q.KoToday:N0}";
            _quota.ValueText = $"{perDay} · unrecognised {ko} · {q.Threads} thread{(q.Threads == 1 ? "" : "s")}";
        }

        private LunaSwitch Switch(string name, string label, bool value, Action<bool> set)
        {
            var toggle = new LunaSwitch { Name = name, Label = label, IsChecked = value };
            toggle.IsCheckedChanged += (_, _) => { set(toggle.IsChecked == true); _settings.Save(); };
            return toggle;
        }

        private Dropdown Choice(string name, (string Value, string Text)[] choices, string current, Action<string> set)
        {
            var dropdown = new Dropdown { Name = name, HorizontalAlignment = HorizontalAlignment.Stretch };
            string[] texts = choices.Select(c => c.Text).ToArray();
            dropdown.Fill(texts, choices.FirstOrDefault(c => c.Value == current).Text ?? texts[0]);
            dropdown.Chose += chosen =>
            {
                if (choices.FirstOrDefault(c => c.Text == chosen as string).Value is not string value) return;
                set(value);
                _settings.Save();
            };
            return dropdown;
        }
    }
}
