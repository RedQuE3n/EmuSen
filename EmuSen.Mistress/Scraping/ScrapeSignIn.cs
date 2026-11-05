using System;

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
            $"Signing in can't be checked or used here: {DeveloperCredentials.NoneHere}, so nothing can be sent to ScreenScraper.");
    }
}
