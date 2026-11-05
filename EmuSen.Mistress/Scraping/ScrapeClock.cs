using System;
using System.Threading;
using System.Threading.Tasks;

namespace EmuSen.Mistress.Scraping
{
    // Time for the pacer and the stop rules; a test moves it instead of waiting.
    public interface IScrapeClock
    {
        DateTimeOffset Now { get; }
        Task Delay(TimeSpan wait, CancellationToken stop);
    }

    public sealed class SystemScrapeClock : IScrapeClock
    {
        public static readonly SystemScrapeClock Instance = new();
        public DateTimeOffset Now => DateTimeOffset.UtcNow;
        public Task Delay(TimeSpan wait, CancellationToken stop) => wait <= TimeSpan.Zero ? Task.CompletedTask : Task.Delay(wait, stop);
    }

    // What the quota allows now: go, wait until a moment, or stopped (until a moment, or until the player acts).
    public enum QuotaGate { Go, Wait, Stopped }

    public sealed record QuotaSnapshot(string Day, int RequestsToday, int? MaxPerDay, int KoToday, int? MaxKoPerDay, int Threads,
        int? MaxRequestsPerMinute, TimeSpan Interval, QuotaGate Gate, DateTimeOffset? Until, string? Reason, bool FromServer);
}
