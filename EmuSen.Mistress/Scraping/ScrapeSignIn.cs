using System;
using System.Collections.Generic;
using System.Text.Json;

namespace EmuSen.Mistress.Scraping
{
    public enum SignInResult { SignedIn, WrongAccount, DeveloperRefused, NoDeveloperFile, Incomplete, Busy, Closed, Blocked, TooMany, Unreachable, Unreadable }

    // The member ssuserInfos describes: the login name, ScreenScraper's level for it, and the quota that applies.
    public sealed record ScrapeMember(string Name, string? Level, ScrapeQuota? Quota);

    // What a Log In or Check came to, with a sentence for the player that never holds a credential - see EmuSen_Settings_Reference.md §4.60.
    public sealed record SignInAnswer(SignInResult Result, ScrapeMember? Member, string Message)
    {
        public bool SignedIn => Result == SignInResult.SignedIn;

        public static SignInAnswer NoDeveloper() => new(SignInResult.NoDeveloperFile, null,
            "Signing in can't be checked or used on this computer: EmuSen's developer file is not here, so nothing can be sent to ScreenScraper.");

        // ScreenScraper answers a wrong member with 403 "Erreur de login : Vérifier les identifiants utilisateurs !" (measured, plan §17.9); a refused developer is 403 too, told apart by its text.
        public static SignInAnswer From(ScrapeStatus status, string body)
        {
            string said = ScrapeRedactor.Redact(body.Split('\n', 2)[0].Trim());
            if (said.Length > 160) said = said[..160];
            switch (status)
            {
                case ScrapeStatus.Found:
                    try
                    {
                        if (ScreenScraperJson.Member(body) is { } member)
                            return new(SignInResult.SignedIn, member, $"Signed in as {member.Name}.");
                        return new(SignInResult.WrongAccount, null, "ScreenScraper did not accept that name and password.");
                    }
                    catch (Exception ex) when (ex is JsonException or KeyNotFoundException or InvalidOperationException)
                    {
                        return new(SignInResult.Unreadable, null, "ScreenScraper's answer could not be read.");
                    }
                case ScrapeStatus.BadCredentials when body.Contains("velop", StringComparison.OrdinalIgnoreCase):
                    return new(SignInResult.DeveloperRefused, null, "ScreenScraper refused EmuSen's developer credentials (403), so no account can be checked with this build.");
                case ScrapeStatus.BadCredentials:
                    return new(SignInResult.WrongAccount, null, "ScreenScraper did not accept that name and password.");
                case ScrapeStatus.ServerBusy:
                    return new(SignInResult.Busy, null, "ScreenScraper is too busy to check an account now (401). Try again in a few minutes.");
                case ScrapeStatus.ApiClosed:
                    return new(SignInResult.Closed, null, "ScreenScraper's API is closed (423). Try again later.");
                case ScrapeStatus.Blacklisted:
                    return new(SignInResult.Blocked, null, "ScreenScraper has blocked this version of the software (426); a newer build is needed.");
                case ScrapeStatus.TooManyRequests or ScrapeStatus.DailyQuota or ScrapeStatus.DailyKoQuota:
                    return new(SignInResult.TooMany, null, "ScreenScraper is refusing more requests for now. Try again later.");
                case ScrapeStatus.Failed:
                    return new(SignInResult.Unreachable, null, ScrapeRedactor.Redact($"ScreenScraper did not answer normally: {said}"));
                default:
                    return new(SignInResult.Unreadable, null, ScrapeRedactor.Redact($"ScreenScraper's answer could not be read: {said}"));
            }
        }
    }
}
