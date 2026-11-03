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
    // What Settings and the scraping windows ask of the window that scrapes: whether it can, the quota, and a run started, resumed or cancelled by the player.
    public interface IScrapeHost
    {
        bool HasDeveloperCredentials { get; }
        DeveloperOrigin? DeveloperOrigin => HasDeveloperCredentials ? Scraping.DeveloperOrigin.TreeFile : null;
        string Status { get; }
        QuotaSnapshot? Quota { get; }
        event Action? ScrapeChanged;
        bool ScrapeRunning { get; }
        ScrapeProgress? Progress { get; }
        int Interrupted { get; }
        System.Threading.Tasks.Task<bool> ConfirmAndScrapeAsync(ScrapeScope scope);
        bool ResumeScrape();
        void CancelScrape();

        // The Scraper window, where a wide run is chosen and started - see EmuSen_Settings_Reference.md §4.60.
        void ShowScraper() { }

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

        // OpenVGDB's download and its Remove (Q42) - see EmuSen_Settings_Reference.md §4.65.
        long? OpenVgdbBytes => null;
        bool OpenVgdbBusy => true;
        System.Threading.Tasks.Task<string?> RemoveOpenVgdbAsync() => System.Threading.Tasks.Task.FromResult<string?>(null);
        System.Threading.Tasks.Task<string> DownloadOpenVgdbAsync() => System.Threading.Tasks.Task.FromResult("");

        // Pass 8's cleanup of media whose game is gone - see EmuSen_Settings_Reference.md §4.76.
        System.Threading.Tasks.Task<string> CleanUpOrphansAsync() => System.Threading.Tasks.Task.FromResult("");
    }

    // A byte count as the scraping lines say it, in KB or MB.
    public static class ScrapeSizes
    {
        public static string Text(long bytes) => bytes >= 1 << 20 ? $"{bytes / 1048576.0:0.#} MB" : $"{Math.Max(1, bytes / 1024)} KB";
    }
}
