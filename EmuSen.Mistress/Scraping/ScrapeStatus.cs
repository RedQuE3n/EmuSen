using System;
using System.Collections.Generic;

namespace EmuSen.Mistress.Scraping
{
    // ScreenScraper's answers, by what the client must do about them - see EmuSen_BigPicture.md §5.1's table.
    public enum ScrapeStatus
    {
        Found,
        NotFound,          // 404: record as unknown
        BadRequest,        // 400: this request is wrong, and will be again
        ServerBusy,        // 401: closed to non-members while the server is loaded; wait five minutes
        BadCredentials,    // 403: stop
        ApiClosed,         // 423: stop
        Blacklisted,       // 426: stop; the software needs a new build
        TooManyRequests,   // 429: slow down
        DailyQuota,        // 430: stop until tomorrow
        DailyKoQuota,      // 431: stop until tomorrow
        Malformed,         // an answer that is not the documented shape
        Failed,            // no answer: the network, a timeout, another status
    }

    // A name search's answer: the games in ScreenScraper's order, and the quota it carried.
    public sealed record SearchAnswer(ScrapeStatus Status, IReadOnlyList<ScrapedGame> Games, ScrapeQuota? Quota, string Detail, TimeSpan Took);
}
