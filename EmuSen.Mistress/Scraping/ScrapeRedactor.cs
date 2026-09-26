using System;
using System.Collections.Concurrent;
using System.Text.RegularExpressions;

namespace EmuSen.Mistress.Scraping
{
    // The one place a ScreenScraper URL or message is made safe to show or write: every credential parameter blanked - see EmuSen_BigPicture.md §17.
    public static partial class ScrapeRedactor
    {
        public const string Blank = "***";

        private static readonly ConcurrentDictionary<string, byte> Literals = new(StringComparer.Ordinal);

        [GeneratedRegex(@"(?i)(?<![A-Za-z0-9_])(devid|devpassword|ssid|sspassword)=([^&\s""'<>\\`)]*)")]
        private static partial Regex Credential();

        // A credential's own value, blanked wherever it turns up, not only after its parameter name; shorter values would blank ordinary words.
        public static void Register(string? secret)
        {
            if (secret is { Length: >= 4 }) Literals.TryAdd(secret, 0);
        }

        public static string Redact(string? text)
        {
            if (string.IsNullOrEmpty(text)) return "";
            string safe = Credential().Replace(text, m => m.Groups[1].Value + "=" + Blank);
            foreach (string secret in Literals.Keys) safe = safe.Replace(secret, Blank, StringComparison.Ordinal);
            return safe;
        }
    }
}
